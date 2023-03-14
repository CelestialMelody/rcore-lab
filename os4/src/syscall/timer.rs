use core::ops::{Add, Sub};

pub const TICKS_PER_SEC: usize = 100;
pub const MSEC_PER_SEC: usize = 1000;
pub const USEC_PER_SEC: usize = 1_000_000;

#[derive(Copy, Clone, Debug)]
/// TimeVal 用于保存时间戳
pub struct TimeVal {
    pub sec: usize,  // second
    pub usec: usize, // microsecond
}

impl Add for TimeVal {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let mut sec = self.sec + other.sec;
        let mut usec = self.usec + other.usec;

        sec += usec / USEC_PER_SEC;
        usec %= USEC_PER_SEC;

        Self { sec, usec }
    }
}

impl Sub for TimeVal {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        if self.sec < other.sec {
            Self { sec: 0, usec: 0 }
        } else if self.sec == other.sec {
            if self.usec < other.usec {
                Self { sec: 0, usec: 0 }
            } else {
                Self {
                    sec: 0,
                    usec: self.usec - other.usec,
                }
            }
        } else {
            let mut sec = self.sec - other.sec;
            let mut usec = self.usec - other.usec;
            if self.usec < other.usec {
                sec -= 1;
                usec = USEC_PER_SEC + self.usec - other.usec;
            }
            Self { sec, usec }
        }
    }
}

impl TimeVal {
    pub fn as_bytes(&self) -> &[u8] {
        let size = core::mem::size_of::<Self>();
        unsafe { core::slice::from_raw_parts(self as *const _ as usize as *const u8, size) }
    }
}
