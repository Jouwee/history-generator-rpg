use std::ops::{Add, Sub};

const DAYS_IN_MONTH: i32 = 28;
const MONTHS_IN_YEAR: i32 = 12;
const DAYS_IN_YEAR: i32 = DAYS_IN_MONTH * MONTHS_IN_YEAR;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct WorldDate {
    timestamp: i32,
}

impl WorldDate {

    pub fn new(year: i32, month: i32, day: i32) -> WorldDate {
        return WorldDate { timestamp: (year * DAYS_IN_YEAR) + (month * DAYS_IN_MONTH) + day }
    }

    pub fn year(&self) -> i32 {
        return self.timestamp / DAYS_IN_YEAR;
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