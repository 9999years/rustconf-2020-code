use std::fmt;

use chrono::prelude::*;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
#[serde(from = "i64")]
pub struct UnixUTC(DateTime<Utc>);

impl From<i64> for UnixUTC {
    fn from(time: i64) -> Self {
        Self(Utc.timestamp(time, 0))
    }
}

impl Into<i64> for UnixUTC {
    fn into(self) -> i64 {
        self.0.timestamp()
    }
}

impl fmt::Debug for UnixUTC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct OneCall {
    pub hourly: Vec<Hourly>,
    pub daily: Vec<Daily>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Hourly {
    pub dt: UnixUTC,
    pub temp: f64,
    pub feels_like: f64,
    pub humidity: f64,
    pub clouds: f64,
    pub rain: Option<Rain>,
    pub snow: Option<Snow>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Cloudiness {
    pub all: f64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Rain {
    #[serde(rename = "1h")]
    pub one_hour: f64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Snow {
    #[serde(rename = "1h")]
    pub one_hour: f64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Daily {
    pub dt: UnixUTC,
    pub sunrise: UnixUTC,
    pub sunset: UnixUTC,
    pub rain: Option<f64>,
    pub snow: Option<f64>,
    pub temp: DailyTemp,
    pub feels_like: DailyTempCommon,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DailyTempCommon {
    pub morn: f64,
    pub day: f64,
    pub eve: f64,
    pub night: f64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DailyTemp {
    #[serde(flatten)]
    pub common: DailyTempCommon,
    pub min: f64,
    pub max: f64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Historical {
    pub hourly: Vec<HistoricalHourly>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct HistoricalHourly {
    pub dt: UnixUTC,
    pub temp: f64,
    pub feels_like: f64,
    pub humidity: f64,
    pub clouds: f64,
    pub wind_speed: f64,
    pub wind_gust: Option<f64>,
    pub rain: Option<Rain>,
    pub snow: Option<Snow>,
}
