#![no_std]
#![no_main]
#![feature(custom_attribute, asm)]

extern crate panic_semihosting;
mod controls;
mod system;
mod timer;

use system::System;
use timer::Timer;

use cortex_m_rt::entry;
use dotstar::{ColorRgb, DemoLightShows};

use dotstar::embedded_hal::digital::ToggleableOutputPin;

#[entry]
fn main() -> ! {
    let mut system = System::new();
    let mut shows = DemoLightShows::new();
    let mut lights = [ColorRgb { r: 0, g: 0, b: 0 }; 90];

    let mut timer = Timer::new();
    timer.force_done(&system);
    loop {
        // Sleep until an interrupt happens! Probably it will be the systick interrupt that fires every 1ms.
        unsafe { asm!("wfi") };

        let mut needs_redisplay = false;
        if let Some(mode) = system.mode_selector.changed() {
            shows.set_mode(mode);
            timer.force_done(&system);
        }

        for i in 0..System::num_encoders() {
            if let Some(clicks) = system.encoder_moved(i) {
                shows.knob_turned(&mut lights, i, clicks);
                needs_redisplay = true;
            }
        }

        for i in 0..System::num_buttons() {
            if system.button_pressed(i) {
                shows.button_pressed(&mut lights, i);
                needs_redisplay = true;
            }
        }

        if timer.is_done(&system) {
            let duration = shows.next_lights(&mut lights);
            timer.restart(&system, &duration);
            needs_redisplay = true;
        }

        if needs_redisplay {
            system.onboard_led.toggle();
            system.send(&lights);
        }
    }
}
