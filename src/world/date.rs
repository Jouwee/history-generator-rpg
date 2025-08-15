use std::ops::{Add, Sub};

use serde::{Deserialize, Serialize};

const DAYS_IN_MONTH: i32 = 28;
const MONTHS_IN_YEAR: i32 = 12;
const DAYS_IN_YEAR: i32 = DAYS_IN_MONTH * MONTHS_IN_YEAR;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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

}

impl Add for WorldDate {
    type Output = WorldDate;
    
    fn add(self, rhs: Self) -> Self::Output {
        return WorldDate {
            timestamp: self.timestamp + rhs.timestamp
        }
    }

}

impl Sub for WorldDate {
    type Output = WorldDate;
    
    fn sub(self, rhs: Self) -> Self::Output {
        return WorldDate {
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
        let date = WorldDate::new(200, 3, 17) + WorldDate::new(1, 2, 21);
        assert_eq!(date.year(), 201);
        assert_eq!(date.month(), 6);
        assert_eq!(date.day(), 10);
    }

}
