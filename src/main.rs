#![no_std]
#![no_main]
#![feature(custom_attribute, asm)]

extern crate panic_semihosting;
mod button;
mod system;
mod timer;

use system::EncoderEvent::*;
use system::System;
use timer::Timer;

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use dotstar::{CircleShow, ColorRgb, Duration, FlashyShow, LightShow};

struct Shows {
    mode: usize,
    flashy_demo: FlashyShow,
    circle_demo: CircleShow,
}

impl Shows {
    fn new() -> Shows {
        Shows {
            mode: 0,
            flashy_demo: FlashyShow::new(),
            circle_demo: CircleShow::new(),
        }
    }

    fn switch_mode(&mut self) {
        self.mode = (self.mode + 1) % 2
    }

    fn knob_left(&mut self, lights: &mut [ColorRgb]) {
        match self.mode {
            0 => self.circle_demo.change_red(-5),
            1 => self.flashy_demo.change_brightness(-10),
            _ => panic!("Invalid mode"),
        }
        self.update(lights);
    }

    fn knob_right(&mut self, lights: &mut [ColorRgb]) {
        match self.mode {
            0 => self.circle_demo.change_red(5),
            1 => self.flashy_demo.change_brightness(10),
            _ => panic!("Invalid mode"),
        }
        self.update(lights);
    }

    fn next(&mut self, lights: &mut [ColorRgb]) -> Duration {
        match self.mode {
            0 => self.circle_demo.next(lights),
            1 => self.flashy_demo.next(lights),
            _ => panic!("Invalid mode"),
        }
    }

    fn update(&mut self, lights: &mut [ColorRgb]) {
        match self.mode {
            0 => self.circle_demo.update(lights),
            1 => self.flashy_demo.update(lights),
            _ => panic!("Invalid mode"),
        }
    }
}

#[entry]
fn main() -> ! {
    hprintln!("Hello world!").unwrap();

    let mut system = System::new();
    let mut shows = Shows::new();
    let mut lights = [ColorRgb { r: 0, g: 0, b: 0 }; 100];

    let mut timer = Timer::new();
    timer.force_ready(&system);
    loop {
        // Sleep until an interrupt happens! Probably it will be the systick interrupt that fires every 1ms.
        unsafe { asm!("wfi") };

        match system.poll_event() {
            Some(ButtonPress) => {
                shows.switch_mode();
                system.write_lights(&lights);
            }
            Some(KnobLeft) => {
                shows.knob_left(&mut lights);
                system.write_lights(&lights);
            }
            Some(KnobRight) => {
                shows.knob_right(&mut lights);
                system.write_lights(&lights);
            }
            None => (),
        }

        if timer.is_ready(&system) {
            timer.reset(&system, &shows.next(&mut lights));
            system.write_lights(&lights);
        }
    }
}
