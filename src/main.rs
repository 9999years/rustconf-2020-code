use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::fs::File;
use std::path::PathBuf;

use chrono::{prelude::*, Duration};
use eyre::WrapErr;
use reqwest::blocking::{Client, Response};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use structopt::StructOpt;
use thiserror::Error;

mod openweather;
use openweather::*;

fn main() -> eyre::Result<()> {
    let opt = Opt::from_args();
    let config_json =
        File::open(&opt.config).wrap_err_with(|| {
            format!(
                "Failed to open config file {:?}",
                opt.config
            )
        })?;
    let config: OpenWeather = serde_json::from_reader(
        &config_json,
    )
    .wrap_err("Failed to deserialize configuration JSON")?;

    let onecall: OneCall = config
        .onecall()
        .wrap_err("Failed to deserialize hourly weather data")?;

    let historical = config
        .historical_day(Utc::today().and_hms(0, 0, 0) - Duration::days(1))
        .wrap_err("Failed to deserialize historical hourly weather data")?;

    let yesterday =
        Stats::from(historical.iter().map(|h| h.feels_like));
    let today = Stats::from(
        onecall.hourly.iter().map(|h| h.feels_like).take(24),
    );

    let diff = TempDifference::from(yesterday.avg, today.avg);

    print!(
        "Good morning! Today will be about {:.2}°F ",
        today.avg
    );
    println!(
        "({min} - {max}°F); that's {diff} {than} yesterday{end}",
        min = today.min,
        max = today.max,
        diff = diff,
        than = match diff {
            TempDifference::Same => "as",
            _ => "than",
        },
        end = if 60.0 <= today.avg && today.avg <= 80.0 {
            " :)"
        } else {
            "."
        }
    );

    Ok(())
}

#[derive(Deserialize, Debug, Clone)]
struct OpenWeather {
    api_key: String,

    lat: f64,
    lon: f64,

    #[serde(skip)]
    client: Client,
}

impl OpenWeather {
    fn get<Response: DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &[(&str, &str)],
    ) -> Result<Response, WeatherError> {
        let bytes = self
            .client
            .get(&format!(
                "https://api.openweathermap.org/data/2.5/{}",
                endpoint
            ))
            .query(params)
            .query(&[
                ("lat", &format!("{}", self.lat)),
                ("lon", &format!("{}", self.lon)),
                ("appid", &self.api_key),
            ])
            .send()?
            .bytes()?;
        // Attempt to deserialize as `Response`
        serde_json::from_reader(&*bytes).map_err(|err| {
            // If we fail, attempt to deserialize as `ClientError`
            (&*bytes)
                .try_into()
                // If we don't have a `ClientError`, fail with the original error.
                .unwrap_or_else(|_| {
                    WeatherError::Deserialize(
                        err,
                        String::from_utf8_lossy(&*bytes)
                            .to_string(),
                    )
                })
        })
    }

    fn onecall(&self) -> Result<OneCall, WeatherError> {
        self.get(
            "onecall",
            &[
                ("exclude", "currently,minutely"),
                ("units", "imperial"),
            ],
        )
    }

    fn historical_day(
        &self,
        date: DateTime<Utc>,
    ) -> Result<Vec<HistoricalHourly>, WeatherError> {
        let historical: Historical = self.get(
            "onecall/timemachine",
            &[
                ("units", "imperial"),
                ("dt", &date.timestamp().to_string()),
            ],
        )?;
        Ok(historical.hourly)
    }

    fn yesterday(
        &self,
    ) -> Result<Vec<HistoricalHourly>, WeatherError> {
        self.historical_day(Utc::now() - Duration::days(1))
    }
}

#[derive(Error, Debug)]
enum WeatherError {
    #[error("Request: {0}")]
    Request(#[from] reqwest::Error),
    #[error("{0} while deserializing JSON: {1}")]
    Deserialize(serde_json::Error, String),
    #[error("Client error ({}): {}", .0.code, .0.message)]
    Client(ClientError),
}

impl TryFrom<&[u8]> for WeatherError {
    type Error = serde_json::Error;
    fn try_from(response: &[u8]) -> Result<Self, Self::Error> {
        Ok(WeatherError::Client(serde_json::from_slice(
            response,
        )?))
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ClientError {
    /// HTTP response code.
    #[serde(rename = "cod")]
    code: u16,
    message: String,
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

#[derive(Debug, PartialEq)]
enum TempDifference {
    MuchColder,
    Colder,
    Same,
    Warmer,
    MuchWarmer,
}

impl fmt::Display for TempDifference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TempDifference::MuchColder => "much colder",
                TempDifference::Colder => "colder",
                TempDifference::Same => "about the same",
                TempDifference::Warmer => "warmer",
                TempDifference::MuchWarmer => "much warmer",
            }
        )
    }
}

impl TempDifference {
    fn from(from: f64, to: f64) -> Self {
        let delta = to - from;
        match delta {
            _ if delta > 10.0 => TempDifference::MuchWarmer,
            _ if delta > 5.0 => TempDifference::Warmer,
            _ if delta < -10.0 => TempDifference::MuchColder,
            _ if delta < -5.0 => TempDifference::Colder,
            _ => TempDifference::Same,
        }
    }
}

struct Stats {
    min: f64,
    max: f64,
    avg: f64,
    count: usize,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
            avg: 0.0,
            count: 0,
        }
    }
}

impl Stats {
    fn from(itr: impl Iterator<Item = f64>) -> Self {
        let mut ret = Self::default();
        let mut sum = 0.0;

        for i in itr {
            if i < ret.min {
                ret.min = i;
            } else if i > ret.max {
                ret.max = i;
            }
            ret.count += 1;
            sum += i;
        }

        ret.avg = sum / ret.count as f64;
        ret
    }
}

mod test {
    use super::*;

    #[test]
    fn test_tempdiff() {
        assert_eq!(
            TempDifference::from(50.0, 69.0),
            TempDifference::MuchWarmer
        );
        assert_eq!(
            TempDifference::from(13.0, 19.0),
            TempDifference::Warmer
        );
        assert_eq!(
            TempDifference::from(50.0, 51.0),
            TempDifference::Same
        );
        assert_eq!(
            TempDifference::from(50.0, 49.0),
            TempDifference::Same
        );
        assert_eq!(
            TempDifference::from(19.0, 13.0),
            TempDifference::Colder
        );
        assert_eq!(
            TempDifference::from(19.0, 5.0),
            TempDifference::MuchColder
        );
    }
}
