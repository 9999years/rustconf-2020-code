use std::fs::File;
use std::path::PathBuf;

use eyre::WrapErr;
use reqwest::blocking::{Client, Response};
use serde::Deserialize;
use structopt::StructOpt;

fn main() -> eyre::Result<()> {
    let opt = Opt::from_args();
    let config_json = File::open(&opt.config)
        .wrap_err_with(|| format!("Failed to open config file {:?}", opt.config))?;
    let config: OpenWeatherConfig =
        serde_json::from_reader(&config_json).wrap_err("Failed to deserialize JSON")?;
    let res = get_weather(&config.api_key)?;
    println!("Response: {:#?}", res);
    let bytes = res.bytes()?;
    println!("Response text: {}", String::from_utf8_lossy(&*bytes));
    Ok(())
}

fn get_weather(api_key: &str) -> Result<Response, reqwest::Error> {
    let client = Client::new();
    client
        .get("https://api.openweathermap.org/data/2.5/weather")
        .query(&[("q", "Waltham,MA,US"), ("appid", api_key)])
        .send()
}

#[derive(Debug, Clone, Deserialize)]
struct OpenWeatherConfig {
    api_key: String,
}

/// A command-line interface to the openweathermap.org API.
#[derive(Debug, StructOpt)]
struct Opt {
    /// Config filename; a JSON file with an `api_key` field.
    #[structopt(
        short,
        long,
        parse(from_os_str),
        default_value = "openweather_api.json"
    )]
    config: PathBuf,
}
