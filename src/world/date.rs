use std::ops::{Add, Sub};

use serde::{Deserialize, Serialize};

const DAYS_IN_MONTH: i32 = 28;
const MONTHS_IN_YEAR: i32 = 12;
const DAYS_IN_YEAR: i32 = DAYS_IN_MONTH * MONTHS_IN_YEAR;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub(crate) struct WorldDate {
    /// Timestamp - Number of days since 1-1-1
    timestamp: i32,
}

impl WorldDate {

    pub(crate) fn new(year: i32, month: i32, day: i32) -> WorldDate {
        return WorldDate { timestamp: (year * DAYS_IN_YEAR) + (month * DAYS_IN_MONTH) + day }
    }

    pub(crate) fn year(&self) -> i32 {
        return self.timestamp / DAYS_IN_YEAR;
    }

    pub(crate) fn month(&self) -> i32 {
        return self.timestamp % DAYS_IN_YEAR / DAYS_IN_MONTH;
    }

    pub(crate) fn day(&self) -> i32 {
        return self.timestamp % DAYS_IN_MONTH;
    }

    pub(crate) fn fmt_long(&self) -> String {
        // TODO: Localize
        let day = match self.day() {
            1 => String::from("1st"),
            2 => String::from("2nd"),
            3 => String::from("3rd"),
            21 => String::from("21st"),
            22 => String::from("22nd"),
            23 => String::from("23rd"),
            n => format!("{n}th")
        };

        let month = match self.month() {
            1 => String::from("Jan."),
            2 => String::from("Feb."),
            3 => String::from("Mar."),
            4 => String::from("Apr."),
            5 => String::from("May."),
            6 => String::from("June"),
            7 => String::from("July"),
            8 => String::from("Aug."),
            9 => String::from("Sep."),
            10 => String::from("Oct."),
            11 => String::from("Nov."),
            _ => String::from("Dec."),
        };

        let year = self.year().to_string();

        return format!("{day} of {month}, {year}");
    }

}

impl Add<Duration> for WorldDate {
    type Output = WorldDate;
    
    fn add(self, rhs: Duration) -> Self::Output {
        return WorldDate {
            timestamp: self.timestamp + rhs.timestamp
        }
    }

}

impl Sub for WorldDate {
    type Output = Duration;
    
    fn sub(self, rhs: Self) -> Self::Output {
        return Duration {
            timestamp: self.timestamp - rhs.timestamp
        }
    }

}



impl Sub<Duration> for WorldDate {
    type Output = Self;

    fn sub(self, rhs: Duration) -> Self::Output {
        Self {
            timestamp: self.timestamp - rhs.timestamp
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub(crate) struct Duration {
    /// Timestamp - Number of days
    timestamp: i32,
}

impl Duration {
    pub(crate) fn days(days: i32) -> Self {
        return Self { timestamp: days }
    }

    pub(crate) fn months(months: i32) -> Self {
        return Self { timestamp: months * DAYS_IN_MONTH }
    }

    pub(crate) fn years(years: i32) -> Self {
        return Self { timestamp: years * DAYS_IN_YEAR }
    }

    pub(crate) fn get_years(&self) -> i32 {
        return self.timestamp / DAYS_IN_YEAR;
    }

    pub(crate) fn percentage_of_year(&self) -> f32 {
        return self.timestamp as f32 / DAYS_IN_YEAR as f32
    }

}

impl Sub for Duration {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            timestamp: self.timestamp - rhs.timestamp
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub(crate) fn base() {
        let date = WorldDate::new(200, 3, 17);
        assert_eq!(date.year(), 200);
        assert_eq!(date.month(), 3);
        assert_eq!(date.day(), 17);
    }

    #[test]
    pub(crate) fn add() {
        let date = WorldDate::new(200, 3, 17) + Duration::days(3);
        assert_eq!(date.year(), 200);
        assert_eq!(date.month(), 3);
        assert_eq!(date.day(), 20);

        let date = WorldDate::new(9, 12, 28) + Duration::days(1);
        assert_eq!(date.year(), 10);
        assert_eq!(date.month(), 1);
        assert_eq!(date.day(), 1);

    }

}
