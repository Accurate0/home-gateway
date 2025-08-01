use anyhow::Context;
use chrono::TimeDelta;
use phf::phf_map;

type TimeDeltaFn = fn(i64) -> Option<TimeDelta>;
const TIME_DELTA_FN_MAP: phf::Map<&'static str, TimeDeltaFn> = phf_map! {
    "h" => TimeDelta::try_hours,
    "m" => TimeDelta::try_minutes,
    "s" => TimeDelta::try_seconds,
    "ms" => TimeDelta::try_milliseconds
};

pub fn parse_datetime_str_with_ms(s: &str) -> anyhow::Result<TimeDelta> {
    let mut final_time_delta = TimeDelta::zero();

    let mut it = s.chars().peekable();
    let mut number = vec![];

    loop {
        let Some(c) = it.next() else {
            break;
        };

        if c.is_ascii_alphabetic() {
            let is_milliseconds = c == 'm' && it.peek().is_some_and(|c| *c == 's');
            match c {
                'h' | 'm' | 's' => {
                    if number.is_empty() {
                        anyhow::bail!("missing number before h, m, s or ms")
                    }

                    let time_value = String::from_utf8(number.clone())?.parse()?;
                    let time_delta_fn = if is_milliseconds {
                        TIME_DELTA_FN_MAP.get("ms").expect("must find ms")
                    } else {
                        TIME_DELTA_FN_MAP
                            .get(&String::from(c))
                            .context("invalid time operator")?
                    };

                    final_time_delta = final_time_delta
                        .checked_add(&time_delta_fn(time_value).context("invalid time")?)
                        .context("time too large...")?;

                    number.clear();
                    if is_milliseconds {
                        it.next();
                    }

                    continue;
                }
                _ => {}
            }
        }

        match c {
            '0'..='9' => {
                number.push(c as u8);
            }

            c if c.is_whitespace() => continue,
            c => anyhow::bail!("unexpected char: {}, example: 1h 32m 2s", c),
        }
    }

    if !number.is_empty() {
        anyhow::bail!("invalid format, example: 1h 23m")
    }

    Ok(final_time_delta)
}

pub mod time_delta_from_str {
    use super::parse_datetime_str_with_ms;
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
        let time_delta = parse_datetime_str_with_ms(&s).map_err(serde::de::Error::custom)?;
        Ok(time_delta)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    #[rstest]
    #[case("3h", TimeDelta::hours(3))]
    #[case("2h 2s", TimeDelta::hours(2) + TimeDelta::seconds(2))]
    #[case("20m 2h 2s", TimeDelta::minutes(20) + TimeDelta::hours(2) + TimeDelta::seconds(2))]
    #[case("20m 2s", TimeDelta::minutes(20) + TimeDelta::seconds(2))]
    #[case("21243s", TimeDelta::seconds(21243))]
    #[case("2332m 2h 2s", TimeDelta::minutes(2332) + TimeDelta::hours(2) + TimeDelta::seconds(2))]
    #[case("20m", TimeDelta::minutes(20))]
    #[case("20ms", TimeDelta::milliseconds(20))]
    #[case("1s 200ms", TimeDelta::seconds(1) + TimeDelta::milliseconds(200))]
    fn test_parse_datetime_str_with_ms(#[case] s: &str, #[case] expected: TimeDelta) {
        let result = parse_datetime_str_with_ms(s).unwrap();
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case("3x")]
    #[case("0")]
    #[case("h")]
    #[case("h3")]
    #[case("2hr")]
    fn test_parse_datetime_str_fail(#[case] s: &str) {
        let result = parse_datetime_str_with_ms(s);
        assert!(result.is_err())
    }
}
