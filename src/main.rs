#![no_std]
#![no_main]

extern crate panic_semihosting;

mod system;

use system::System;

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use dotstar::{ColorRgb, Duration, FlashyShow, LightShow};

#[entry]
fn main() -> ! {
    hprintln!("Hello world!").unwrap();

    let mut system = System::new();

    // let mut settings = CircleShowSettings::default();
    // settings.brightness = 20;
    // let mut demo = CircleShow::new(&settings);
    let mut demo = FlashyShow::new(&());
    let mut lights = [ColorRgb { r: 0, g: 0, b: 0 }; 100];

    let period = 10u32;
    let mut duration = Duration::Millis(0);
    loop {
        system.delay_ms(500_u32);
        hprintln!("{}", system.read_encoder()).unwrap();
        if system.read_button() {
            hprintln!("button down!").unwrap();
        }

        if duration.is_zero() {
            // hprintln!("before").unwrap();
            duration = demo.next(&mut lights);
            // hprintln!("after").unwrap();
            system.write_lights(&lights);
        }
        system.delay_ms(period);
        duration.subtract(period);
    }
}
