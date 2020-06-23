use serde::Deserialize;
use std::time::Instant;
// use rayon::prelude::*;

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
            x.url = [&x.url, "core/os/x86_64/"].join("");
            x.details = [&x.url, "core.db.tar.gz"].join("");
            x
        })
        .collect::<Vec<_>>();
 
    println!("took {} ms", get_response_time(&servers[15].details).await?);
    Ok(())
}

async fn get_response_time(url: &str) -> Result<u128, Box<dyn std::error::Error>> {
    let now = Instant::now();
    reqwest::get(url).await?;
    Ok(now.elapsed().as_millis())
}
