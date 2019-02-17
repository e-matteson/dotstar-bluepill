#![no_std]
#![no_main]
#![feature(custom_attribute, asm)]

extern crate panic_semihosting;
mod button;
mod system;
mod timer;

use system::System;
use timer::Timer;

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use dotstar::{ColorRgb, FlashyShow, LightShow};

#[entry]
fn main() -> ! {
    hprintln!("Hello world!").unwrap();

    let mut system = System::new();

    // let mut settings = CircleShowSettings::default();
    // settings.brightness = 20;
    // let mut demo = CircleShow::new(&settings);
    let mut demo = FlashyShow::new(&());
    let mut lights = [ColorRgb { r: 0, g: 0, b: 0 }; 100];

    let mut timer = Timer::new();
    timer.force_ready(&system);
    loop {
        // Sleep until an interrupt happens! Probably it will be the systick interrupt that fires every 1ms.
        unsafe { asm!("wfi") };

        if system.was_pressed() {
            hprintln!("button down!").unwrap();
        }

        if timer.is_ready(&system) {
            timer.reset(&system, &demo.next(&mut lights));
            system.write_lights(&lights);
        }
    }
}
