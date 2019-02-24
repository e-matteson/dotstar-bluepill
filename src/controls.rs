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

pub struct Selector<P0, P1, P2, P3, P4, P5, P6, P7>
where
    P0: InputPin,
    P1: InputPin,
    P2: InputPin,
    P3: InputPin,
    P4: InputPin,
    P5: InputPin,
    P6: InputPin,
    P7: InputPin,
{
    pin0: P0,
    pin1: P1,
    pin2: P2,
    pin3: P3,
    pin4: P4,
    pin5: P5,
    pin6: P6,
    pin7: P7,
    previous: Option<u8>,
}

impl<P0, P1, P2, P3, P4, P5, P6, P7> Selector<P0, P1, P2, P3, P4, P5, P6, P7>
where
    P0: InputPin,
    P1: InputPin,
    P2: InputPin,
    P3: InputPin,
    P4: InputPin,
    P5: InputPin,
    P6: InputPin,
    P7: InputPin,
{
    pub fn new(
        pin0: P0,
        pin1: P1,
        pin2: P2,
        pin3: P3,
        pin4: P4,
        pin5: P5,
        pin6: P6,
        pin7: P7,
    ) -> Self {
        Self {
            pin0,
            pin1,
            pin2,
            pin3,
            pin4,
            pin5,
            pin6,
            pin7,
            previous: None,
        }
    }

    pub fn changed(&mut self) -> Option<u8> {
        let current = self.selection()?;
        if let Some(prev) = self.previous {
            if prev == current {
                return None;
            }
        }
        self.previous = Some(current);
        self.previous.clone()
    }

    pub fn selection(&self) -> Option<u8> {
        Some(if self.pin0.is_low() {
            0
        } else if self.pin1.is_low() {
            1
        } else if self.pin2.is_low() {
            2
        } else if self.pin3.is_low() {
            3
        } else if self.pin4.is_low() {
            4
        } else if self.pin5.is_low() {
            5
        } else if self.pin6.is_low() {
            6
        } else if self.pin7.is_low() {
            7
        } else {
            // Disconnected, or currently rotating between positions
            return None;
        })
    }
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

    pub fn clicks_moved(&mut self) -> Option<i16> {
        const COUNTS_PER_CLICK: i16 = 4;
        let new_count: u16 = self.qei.count().into();
        let diff = new_count.wrapping_sub(self.prev_count) as i16;
        let clicks = diff / COUNTS_PER_CLICK; // floor to the nearest whole click.
        if clicks == 0 {
            return None;
        } else if clicks < 0 {
            self.prev_count = self
                .prev_count
                .wrapping_sub((clicks.abs() * COUNTS_PER_CLICK) as u16);
        } else {
            self.prev_count = self
                .prev_count
                .wrapping_add((clicks * COUNTS_PER_CLICK) as u16);
        }
        Some(clicks)
    }
}
