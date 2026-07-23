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

    while let Some(c) = it.next() {
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
    use chrono::TimeDelta;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(delta: &TimeDelta, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&super::humanize(*delta))
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

pub mod option_time_delta_from_str {
    use super::parse_datetime_str_with_ms;
    use chrono::TimeDelta;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(delta: &Option<TimeDelta>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match delta {
            Some(delta) => serializer.serialize_some(&super::humanize(*delta)),
            None => serializer.serialize_none(),
        }
    }

    /// Deserialize an optional duration string (e.g. `"24h"`) into
    /// `Option<TimeDelta>`. Pairs with `#[serde(default)]` so the field can be
    /// omitted entirely.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<TimeDelta>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let Some(s) = Option::<String>::deserialize(deserializer)? else {
            return Ok(None);
        };
        let time_delta = parse_datetime_str_with_ms(&s).map_err(serde::de::Error::custom)?;
        Ok(Some(time_delta))
    }
}

pub fn humanize(delta: TimeDelta) -> String {
    let mut secs = delta.num_seconds();
    let sign = if secs < 0 { "-" } else { "" };
    secs = secs.abs();
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;

    let mut parts = Vec::new();
    if hours > 0 {
        parts.push(format!("{hours}h"));
    }
    if minutes > 0 {
        parts.push(format!("{minutes}m"));
    }
    if seconds > 0 || parts.is_empty() {
        parts.push(format!("{seconds}s"));
    }
    format!("{sign}{}", parts.join(" "))
}

pub mod signed_time_delta_from_str {
    use super::parse_datetime_str_with_ms;
    use chrono::TimeDelta;
    use serde::{self, Deserialize, Deserializer};

    /// Deserialize a signed duration string (e.g. `"30m"`, `"-15m"`) into a
    /// [`TimeDelta`]. A leading `-` negates the parsed magnitude. Pairs with
    /// `#[serde(default)]` (which yields a zero delta) so the field is optional.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<TimeDelta, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let (negative, rest) = match s.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, s.as_str()),
        };
        let delta = parse_datetime_str_with_ms(rest).map_err(serde::de::Error::custom)?;
        Ok(if negative { -delta } else { delta })
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
    #[case(TimeDelta::minutes(30), "30m")]
    #[case(TimeDelta::minutes(-30), "-30m")]
    #[case(TimeDelta::hours(2), "2h")]
    #[case(TimeDelta::seconds(5430), "1h 30m 30s")]
    #[case(TimeDelta::zero(), "0s")]
    fn test_humanize(#[case] delta: TimeDelta, #[case] expected: &str) {
        assert_eq!(humanize(delta), expected);
    }

    #[rstest]
    #[case("30m", TimeDelta::minutes(30))]
    #[case("-30m", TimeDelta::minutes(-30))]
    #[case("-1h 30m", TimeDelta::minutes(-90))]
    #[case("2h", TimeDelta::hours(2))]
    fn test_signed_offset_parses(#[case] s: &str, #[case] expected: TimeDelta) {
        use serde::de::IntoDeserializer;
        use serde::de::value::{Error, StrDeserializer};
        let de: StrDeserializer<Error> = s.into_deserializer();
        let result = signed_time_delta_from_str::deserialize(de).unwrap();
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
