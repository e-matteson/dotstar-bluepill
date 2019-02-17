use dotstar::embedded_hal::digital::InputPin;

pub struct Button<T: InputPin> {
    pin: T,
    history: u8,
    was_pressed: bool,
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
