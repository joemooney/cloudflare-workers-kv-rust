
#[allow(unused_imports)]
// #[cfg(target_arch = "wasm32")]
use super::utils::{self, return_error, info};

// #[cfg(not(target_arch = "wasm32"))]
// use log::{info, warn};



// use chrono::offset::Utc;
//use chrono::{Duration, DateTime};
use chrono::{Duration};
// use std::sync::Arc;
// use std::time::UNIX_EPOCH;


enum TimeUnit {
    Minute,
    Hour,
    Second,
    Day,
    Week,
    Month,
    Year,
}

pub struct DurationHelper {
    spec: String,
    duration: Duration,
    current_unit: String,
    unit: TimeUnit,
    count: i64,
    // total: i64,
}

impl DurationHelper {
    /// 
    pub fn tally(&mut self) -> Result<(), String>{
        info!(">tally current_unit:{} count:{} duration:{:?}", self.current_unit, self.count, self.duration);
        if self.current_unit == "" {
            return Ok(())
        }
        self.check_unit()?;
        if self.count != 0 {
            let d = match self.unit {
                TimeUnit::Second => self.duration.checked_add(&Duration::seconds(self.count)),
                TimeUnit::Minute => self.duration.checked_add(&Duration::minutes(self.count)),
                TimeUnit::Hour => self.duration.checked_add(&Duration::hours(self.count)),
                TimeUnit::Day => self.duration.checked_add(&Duration::days(self.count)),
                TimeUnit::Week => self.duration.checked_add(&Duration::weeks(self.count)),
                TimeUnit::Month => self.duration.checked_add(&Duration::days(self.count*30)),
                TimeUnit::Year => self.duration.checked_add(&Duration::weeks(self.count*52)),
            };
            match d {
                Some(d) => self.duration = d,
                None => return Err(format!("Invalid duration {}", self.spec))
            }
        }
        info!("<tally current_unit:{} count:{} duration:{:?}", self.current_unit, self.count, self.duration);
        self.current_unit = "".to_owned();
        self.count = 0;
        Ok(())
    }

    fn invalid(&self) -> Result<(), String> {
        return Err(format!("Invalid duration spec:{}", self.spec))
    }

    fn check_unit(&mut self) -> Result<(), String> {
        info!("check_unit: <{}>", self.current_unit);
        if self.current_unit == "" {
            if self.count > 0 {
                self.invalid()?;
            }
        }
        self.unit = match self.current_unit.to_lowercase().as_str() {
            "h" | "hr" | "hrs" | "hours"  | "hour" => TimeUnit::Hour,
            "m" | "min"| "mins" | "minute" | "minutes" => TimeUnit::Minute,
            "s" | "sec"| "secs" | "second" | "seconds" => TimeUnit::Second,
            "d" | "day"| "days"  => TimeUnit::Day,
            "mon"| "months"  => TimeUnit::Month,
            "w" | "wk"| "wks" | "week" | "weeks"  => TimeUnit::Week,
            "y" | "yr"| "yrs" | "year" | "years"  => TimeUnit::Year,
            _ => return self.invalid(),
        };
        Ok(())
    }
    /// Convert a string into a number of seconds
    pub fn convert(duration: &str) -> Result<i64, String> {
        let mut t = DurationHelper{ duration: Duration::zero(), spec: duration.to_owned(), count: 0, unit: TimeUnit::Second, current_unit: "".to_owned() };
        t.calc()?;
        Ok(t.duration.num_seconds())
    }

    /// calculate number of seconds in provided duration
    /// 1day = 24*60*60
    fn calc(&mut self) -> Result<i64, String> {
        let chars = self.spec.chars().collect::<Vec<char>>();
        info!("parse: {:?}", chars);
        for c in chars {
            info!("char: {}", c);
            if c.is_whitespace() {
                self.tally()?;
            } else if c == '-' {
                self.tally()?;
                self.count = -1;
            } else if c.is_digit(10) {
                if self.current_unit != "" {
                    self.tally()?;
                }
                let d = (c.to_string()).parse::<i64>().unwrap(); 
                self.count *= 10;
                if self.count < 0 {
                    self.count -= d;
                } else {
                    self.count += d;
                }
            } else {
                if self.current_unit == "" {
                    self.tally()?;
                }
                self.current_unit.push(c)
            }
        }
        self.tally()?;
        Ok(self.count)
    }
}

#[cfg(test)]
mod tests {
    use super::DurationHelper;

    #[test]
    fn test1() {
        let result = DurationHelper::convert("1d").unwrap();
        assert_eq!(result, 24*60*60);
    }
}
