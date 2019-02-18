use dotstar::embedded_hal::digital::InputPin;
use dotstar::embedded_hal::Qei;

pub struct Button<T: InputPin> {
    pin: T,
    history: u8,
    was_pressed: bool,
}

pub struct Encoder<T>
where
    T: Qei,
    T::Count: Into<u16>,
{
    qei: T,
    prev_count: u16,
}

impl<T: InputPin> Button<T> {
    pub fn new(pin: T) -> Self {
        Button {
            pin,
            history: 0,
            was_pressed: false,
        }
    }

    pub fn sample(&mut self) {
        self.history <<= 1;
        if self.pin.is_low() {
            self.history |= 1u8;
        }
        if self.history == 0b01111111 {
            self.was_pressed = true;
        }
    }

    pub fn was_pressed(&mut self) -> bool {
        // Check if there was a button press, resetting the flag to false.
        let ret = self.was_pressed;
        self.was_pressed = false;
        ret
    }
}

impl<T> Encoder<T>
where
    T: Qei,
    T::Count: Into<u16>,
{
    pub fn new(qei: T) -> Self {
        Self { qei, prev_count: 0 }
    }

    pub fn clicks_moved(&mut self) -> i16 {
        const COUNTS_PER_CLICK: i16 = 4;
        let new_count: u16 = self.qei.count().into();
        let diff = new_count.wrapping_sub(self.prev_count) as i16;
        let clicks = diff / COUNTS_PER_CLICK; // floor to the nearest whole click.
        if clicks < 0 {
            self.prev_count
                .wrapping_sub((clicks.abs() * COUNTS_PER_CLICK) as u16);
        } else {
            self.prev_count
                .wrapping_add((clicks * COUNTS_PER_CLICK) as u16);
        }
        clicks
    }
}
