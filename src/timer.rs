use crate::system::System;
use dotstar::Duration;

pub struct Timer {
    start_time: u32,
    length: u32,
    is_disabled: bool,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            start_time: 0,
            length: 0,
            is_disabled: true,
        }
    }

    pub fn reset(&mut self, sys: &System, duration: &Duration) {
        match duration {
            Duration::Forever => {
                self.is_disabled = true;
            }
            Duration::Millis(ms) => {
                self.start_time = sys.get_millis();
                self.length = *ms;
                self.is_disabled = false;
            }
        }
    }

    pub fn force_ready(&mut self, sys: &System) {
        self.is_disabled = false;
        self.length = 0;
        self.start_time = sys.get_millis();
    }

    pub fn is_ready(&mut self, sys: &System) -> bool {
        if self.is_disabled {
            return false;
        }
        let elapsed = sys.get_millis() - self.start_time;
        if elapsed >= self.length {
            self.is_disabled = true;
            true
        } else {
            false
        }
    }
}
