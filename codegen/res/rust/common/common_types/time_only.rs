/* This file will be copied into the folder for generated code. */

use std::ops::{Deref, DerefMut};

use chrono::{TimeZone, Utc};
use serde::{de, ser, Deserialize, Deserializer, Serialize, Serializer};
use time::{self, format_description::well_known::Rfc3339};

#[derive(Clone, Debug)]
pub struct Time(time::Time);

impl Deref for Time {
    type Target = time::Time;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Time {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Serialize for Time {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_str(time_to_rfc3339(self).map_err(ser::Error::custom)?.as_str())
    }
}

impl<'de> Deserialize<'de> for Time {
    fn deserialize<D>(deserializer: D) -> Result<Time, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        rfc3339_to_time(&s).map_err(de::Error::custom)
    }
}

fn time_to_rfc3339(time: &Time) -> Result<String, &'static str> {
    let date_time = Utc
        .with_ymd_and_hms(
            0,
            1,
            1,
            u32::from(time.hour()),
            u32::from(time.minute()),
            u32::from(time.second()),
        )
        .unwrap();
    let date_time_string = date_time.to_rfc3339();
    let t_ix = date_time_string
        .find('T')
        .ok_or("error serializing Time to RFC 3339 format")?;
    let plus_ix = date_time_string
        .find('+')
        .ok_or("error serializing Time to RFC 3339 format")?;
    let time_str = &date_time_string[t_ix + 1..plus_ix];
    let mut time_string = time_str.to_string();
    time_string.push('Z');
    Ok(time_string)
}

fn rfc3339_to_time(time_str: &str) -> Result<Time, &'static str> {
    Ok(Time(
        time::Time::parse(format!("0000-01-01T{time_str}").as_str(), &Rfc3339)
            .map_err(|_| "error deserializing Time from RFC 3339 format")?,
    ))
}
