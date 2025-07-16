use chrono::{DateTime, FixedOffset, TimeDelta, Utc};
use std::ops::Mul;
use tokio::task::yield_now;

#[derive(Debug, Clone)]
pub struct CronlikeExpression {
    seconds_till_next: u64,
    now: fn() -> DateTime<Utc>,
}

impl CronlikeExpression {
    pub fn from_str(
        frequency: &str,
        now: fn() -> DateTime<Utc>,
    ) -> anyhow::Result<CronlikeExpression> {
        let mut it = frequency.split(" ").peekable();
        let mut starts_with_every = false;
        let mut number: Option<u64> = None;
        let mut seconds = 0u64;

        loop {
            let c = it.peek();
            if c.is_none() {
                break;
            }

            let s = c.unwrap();
            match *s {
                "every" => {
                    starts_with_every = true;
                }

                "weeks" if number.is_some() => seconds = number.unwrap().mul(10080 * 60),
                "week" if number.is_some_and(|n| n == 1) => {
                    seconds = number.unwrap().mul(10080 * 60)
                }

                "days" if number.is_some() => seconds = number.unwrap().mul(1440 * 60),
                "day" if number.is_some_and(|n| n == 1) => seconds = number.unwrap().mul(1440 * 60),

                "minutes" if number.is_some() => seconds = number.unwrap().mul(60),
                "minute" if number.is_some_and(|n| n == 1) => seconds = number.unwrap().mul(60),

                "seconds" if number.is_some() => seconds = number.unwrap(),
                "second" if number.is_some_and(|n| n == 1) => seconds = number.unwrap(),

                _ => {
                    if s.parse::<u64>().is_ok() && starts_with_every {
                        number = Some(s.parse::<u64>().unwrap())
                    } else {
                        return Err(anyhow::Error::msg(format!("invalid expression: {s}")));
                    }
                }
            }

            it.next();
        }

        Ok(CronlikeExpression {
            seconds_till_next: seconds,
            now,
        })
    }

    pub async fn next_trigger(&self, start_date: DateTime<FixedOffset>) -> TimeDelta {
        let time_delta = TimeDelta::seconds(self.seconds_till_next.try_into().unwrap());
        let now = self.now;
        let mut current_date = start_date;
        loop {
            if current_date >= now().fixed_offset() {
                break;
            }

            current_date = current_date.checked_add_signed(time_delta).unwrap();
            yield_now().await;
        }

        current_date - now().fixed_offset()
    }
}

pub mod cronlike_expression_from_str {
    use super::CronlikeExpression;
    use chrono::{DateTime, Utc};
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
    pub fn deserialize<'de, D>(deserializer: D) -> Result<CronlikeExpression, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let cronlike_expression =
            CronlikeExpression::from_str(&s, chrono::Utc::now).map_err(serde::de::Error::custom)?;
        Ok(cronlike_expression)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    #[rstest]
    #[case("every 2 weeks", 20160 * 60)]
    #[case("every 2 minutes", 2 * 60)]
    #[case("every 3 days", 3 * 1440 * 60)]
    fn test_parse_from_str(#[case] s: &str, #[case] expected: u64) {
        let result = CronlikeExpression::from_str(s, chrono::Utc::now).unwrap();
        assert_eq!(result.seconds_till_next, expected);
    }

    #[rstest]
    #[case("every 2 weeks", TimeDelta::days(13))]
    #[case("every 2 days", TimeDelta::days(1))]
    #[tokio::test]
    async fn test_parse_from_str_with_next(#[case] s: &str, #[case] next: TimeDelta) {
        let now = || "2025-07-15T09:00:00.0+08:00".parse().unwrap();
        let result = CronlikeExpression::from_str(s, now).unwrap();
        assert_eq!(
            result
                .next_trigger("2025-07-14T09:00:00.0+08:00".parse().unwrap())
                .await,
            next
        );
    }

    #[rstest]
    #[case("every 2 minutes", TimeDelta::seconds(58))]
    #[case("every 3 seconds", TimeDelta::seconds(1))]
    #[tokio::test]
    async fn test_parse_from_str_with_next_2(#[case] s: &str, #[case] next: TimeDelta) {
        let now = || "2025-07-15T09:01:02.0+08:00".parse().unwrap();
        let result = CronlikeExpression::from_str(s, now).unwrap();
        assert_eq!(
            result
                .next_trigger("2025-07-14T09:00:00.0+08:00".parse().unwrap())
                .await,
            next
        );
    }
}
