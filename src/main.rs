use serde::Deserialize;
use std::time::{Instant, Duration};
use futures::future::join_all;

#[derive(Deserialize, Debug)]
struct Mirror {
    url: String,
    protocol: String,
    last_sync: Option<String>,
    completion_pct: Option<f64>,
    delay: Option<u64>,
    duration_avg: Option<f64>,
    duration_stddev: Option<f64>,
    score: Option<f64>,
    active: bool,
    country: String,
    country_code: String,
    isos: bool,
    ipv4: bool,
    ipv6: bool,
    details: String,
}

#[derive(Deserialize, Debug)]
struct StatusData {
    cutoff: u32,
    last_check: String,
    num_checks: u16,
    check_frequency: u32,
    urls: Vec<Mirror>,
    version: u16
}

#[derive(Debug)]
struct Ranked {
    url: String,
    score: f64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let status_req = reqwest::get("https://www.archlinux.org/mirrors/status/json/")
        .await?
        .text()
        .await?;


    let mut status_data: StatusData = serde_json::from_str(&status_req)?;

    let servers = status_data.urls.iter_mut()
        .filter(|x| x.protocol == "https" && x.ipv4 && x.active)
        .filter(|x| match x.score {
            Some(_) => true,
            None => false,
        })
        .map(|x| {
            //x.url = [&x.url, "core/os/x86_64/"].join("");
            x.details = [&x.url, "core/os/x86_64/core.db.tar.gz"].join("");
            x
        })
        .collect::<Vec<_>>();
 
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(15))
        .build()?;

    let waiting = servers.iter()
        .map(|x| {
            get_response_time(&client, &x.details)
        })
        .collect::<Vec<_>>();

    
    let times = join_all(waiting).await;

    let mut urls = (0..servers.len()).into_iter()
        .filter_map(|i| {
            if let Ok(time) = times[i] {
                let url = ["Server = ", &servers[i].url, "$repo/os/$arch"].join("");
                let score = weighted_score(servers[i].score?, (time as f64) / 1000.);
                Some(Ranked { url, score })
            } else {
                None
            }
        })
        .filter(|x| x.score > 0.5)
        .collect::<Vec<_>>();

    urls.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    println!("{:?}", urls);

    Ok(())
}

async fn get_response_time(client: &reqwest::Client, url: &str) -> Result<u128, Box<dyn std::error::Error>> {
    println!("creating");
    let now = Instant::now();
    client.get(url).send().await?;
    println!("done {}", now.elapsed().as_millis());
    Ok(now.elapsed().as_millis())
}

fn weighted_score(score: f64, time: f64) -> f64 {
    (-(time * time) / 100.).exp() * 0.5 +
    (-(score * score) / 100.).exp() * 0.5
}
