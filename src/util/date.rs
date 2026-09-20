use anyhow::{Result, bail};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};
use std::fmt;
use std::str::FromStr;

/// `YYYY-MM-DD` の日付です。
///
/// 並べ替えは年、月、日の順に比べます。フィールドの順序がそのまま比較の順序です。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Date {
    year: u16,
    month: u8,
    day: u8,
}

impl Date {
    /// 存在する日付であれば返します。
    pub fn new(year: u16, month: u8, day: u8) -> Result<Self> {
        if year > 9999 {
            bail!("年は 4 桁までです : {year}");
        }
        if !(1..=12).contains(&month) {
            bail!("月は 1 から 12 までです : {month}");
        }
        if day < 1 || day > days_in_month(year, month) {
            bail!("{year} 年 {month} 月に {day} 日はありません");
        }

        Ok(Self { year, month, day })
    }
}

/// その月の日数を返します。
fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        2 if is_leap(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// うるう年であれば `true` を返します。
///
/// 乗算、論理積、比較の 3 つで判定します。Falk Hüffner が Z3 で探した定数で、
/// 0 年から 102499 年まで通常の規則と一致します。libstdc++ の
/// `std::chrono::year::is_leap` も同じ形です。
/// <https://hueffner.de/falk/blog/a-leap-year-check-in-three-instructions.html>
fn is_leap(year: u16) -> bool {
    // year は 4 桁までなので、一致する範囲に収まる
    let year = year as u32;
    (year.wrapping_mul(1_073_750_999) & 3_221_352_463) <= 126_976
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { year, month, day } = self;
        write!(f, "{year:04}-{month:02}-{day:02}")
    }
}

impl FromStr for Date {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let is_digits =
            |part: &str, len: usize| part.len() == len && part.bytes().all(|b| b.is_ascii_digit());

        let parts: Vec<_> = s.split('-').collect();
        let [year, month, day] = parts[..] else {
            bail!("日付は YYYY-MM-DD の形で書いてください : {s}");
        };
        if !is_digits(year, 4) || !is_digits(month, 2) || !is_digits(day, 2) {
            bail!("日付は YYYY-MM-DD の形で書いてください : {s}");
        }

        Self::new(year.parse()?, month.parse()?, day.parse()?)
    }
}

impl Serialize for Date {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Date {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(D::Error::custom)
    }
}

#[cfg(test)]
mod test {
    use super::{Date, is_leap};

    /// 3 命令の判定が、通常の規則と一致することを確かめます。
    #[test]
    fn matches_the_usual_leap_year_rule() {
        for year in 0..=9999u16 {
            let usual = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
            assert_eq!(is_leap(year), usual, "{year} 年");
        }
    }

    #[test]
    fn rejects_dates_that_do_not_exist() {
        assert!("2026-02-29".parse::<Date>().is_err());
        assert!("2024-02-29".parse::<Date>().is_ok());
        assert!("2026-13-01".parse::<Date>().is_err());
        assert!("2026-04-31".parse::<Date>().is_err());
        assert!("2026-01-00".parse::<Date>().is_err());
    }

    #[test]
    fn rejects_other_shapes() {
        assert!("2026-1-1".parse::<Date>().is_err());
        assert!("2026/01/01".parse::<Date>().is_err());
        assert!("2026-01-01 ".parse::<Date>().is_err());
        assert!("".parse::<Date>().is_err());
        assert!("2026-01-01<b>".parse::<Date>().is_err());
    }

    #[test]
    fn keeps_the_written_form() {
        let date: Date = "2026-01-02".parse().unwrap();
        assert_eq!(date.to_string(), "2026-01-02");
    }

    #[test]
    fn compares_by_year_then_month_then_day() {
        let d = |s: &str| s.parse::<Date>().unwrap();
        assert!(d("2025-12-31") < d("2026-01-01"));
        assert!(d("2026-01-31") < d("2026-02-01"));
        assert!(d("2026-01-01") < d("2026-01-02"));
    }
}
