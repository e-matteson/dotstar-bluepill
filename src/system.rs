extern crate panic_semihosting;
extern crate stm32f1xx_hal as hal;

use dotstar::{ColorRgb, DotstarStrip};
use hal::{
    delay::Delay,
    gpio::{
        gpioa::{PA0, PA1, PA2, PA5, PA6, PA7},
        Alternate, Floating, Input, PullUp, PushPull,
    },
    prelude::*,
    qei::Qei,
    spi::{Mode, Phase, Polarity, Spi},
    stm32::{self, SPI1, TIM2},
};

type DotstarSPI = Spi<
    SPI1,
    (
        PA5<Alternate<PushPull>>,
        PA6<Input<Floating>>,
        PA7<Alternate<PushPull>>,
    ),
>;

pub struct System {
    pub strip: DotstarStrip<DotstarSPI>,
    pub delay: Delay,
    pub encoder: Qei<TIM2, (PA0<Input<Floating>>, PA1<Input<Floating>>)>,
    pub button: PA2<Input<PullUp>>,
}

impl System {
    pub fn new() -> System {
        // Get access to peripherals
        let cp = cortex_m::Peripherals::take().unwrap();
        let dp = stm32::Peripherals::take().unwrap();
        let mut rcc = dp.RCC.constrain();
        let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

        // Configure clocks
        let mut flash = dp.FLASH.constrain();
        let clocks = rcc.cfgr.freeze(&mut flash.acr);
        let delay = Delay::new(cp.SYST, clocks);

        // Get SPI pins
        let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
        let sck = gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl);
        let miso = gpioa.pa6;
        let mosi = gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl);

        // Get quadrature encoder pins
        let c1 = gpioa.pa0;
        let c2 = gpioa.pa1;
        let encoder = Qei::tim2(dp.TIM2, (c1, c2), &mut afio.mapr, &mut rcc.apb1);
        let button = gpioa.pa2.into_pull_up_input(&mut gpioa.crl);

        // Onboard LED
        // let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);
        // let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
        // led.set_high(); // high is off

        // Setup SPI
        let spi = Spi::spi1(
            dp.SPI1,
            (sck, miso, mosi),
            &mut afio.mapr,
            Mode {
                // mode 0
                phase: Phase::CaptureOnFirstTransition,
                polarity: Polarity::IdleLow,
            },
            1.mhz(),
            clocks,
            &mut rcc.apb2,
        );

        System {
            strip: DotstarStrip::new(spi),
            delay,
            encoder,
            button,
        }
    }

    pub fn delay_ms(&mut self, ms: u32) {
        self.delay.delay_ms(ms);
    }

    pub fn read_encoder(&mut self) -> u16 {
        self.encoder.count()
    }

    pub fn read_button(&mut self) -> bool {
        self.button.is_low()
    }

    pub fn write_lights(&mut self, lights: &[ColorRgb]) {
        self.strip.send(lights).expect("Failed to send lights");
    }
}
