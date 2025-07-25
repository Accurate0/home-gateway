use anyhow::Context;
use chrono::TimeDelta;
use phf::phf_map;

type TimeDeltaFn = fn(i64) -> Option<TimeDelta>;
const TIME_DELTA_FN_MAP: phf::Map<char, TimeDeltaFn> = phf_map! {
    'h' => TimeDelta::try_hours,
    'm' => TimeDelta::try_minutes,
    's' => TimeDelta::try_seconds
};

pub fn parse_datetime_str(s: &str) -> anyhow::Result<TimeDelta> {
    let mut final_time_delta = TimeDelta::zero();

    let mut it = s.chars().peekable();
    let mut number = vec![];

    loop {
        if it.peek().is_none() {
            break;
        }

        let c = *it.peek().unwrap();
        match c {
            '0'..='9' => loop {
                let c = it.peek();
                if c.is_some_and(|c| c.is_ascii_digit()) {
                    number.push(*c.unwrap() as u8);
                    it.next();
                } else {
                    break;
                }
            },
            'm' | 'h' | 's' => {
                if number.is_empty() {
                    anyhow::bail!("missing number before h, m, or s")
                }

                let time_value = String::from_utf8(number.clone())?.parse()?;
                let time_delta_fn = TIME_DELTA_FN_MAP.get(&c).context("invalid time operator")?;

                final_time_delta = final_time_delta
                    .checked_add(&time_delta_fn(time_value).context("invalid time")?)
                    .context("time too large...")?;

                it.next();
                number.clear();
            }
            c if c.is_whitespace() => {
                it.next();
            }
            c => anyhow::bail!("unexpected char: {}, example: 1h 32m 2s", c),
        }
    }

    if !number.is_empty() {
        anyhow::bail!("invalid format, example: 1h 23m")
    }

    Ok(final_time_delta)
}

pub mod time_delta_from_str {
    use super::parse_datetime_str;
    use chrono::{DateTime, TimeDelta, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    #[allow(unused)]
    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        todo!()
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<TimeDelta, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let time_delta = parse_datetime_str(&s).map_err(serde::de::Error::custom)?;
        Ok(time_delta)
    }
}
