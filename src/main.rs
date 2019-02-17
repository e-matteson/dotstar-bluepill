#![no_std]
#![no_main]

extern crate panic_semihosting;
extern crate stm32f1xx_hal as hal;

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;

use hal::{
    delay::Delay,
    prelude::*,
    spi::{Mode, Phase, Polarity, Spi},
    qei::Qei,
    stm32
};

use dotstar::{ColorRgb, CircleShow, CircleShowSettings, DotstarStrip, Duration, LightShow, FlashyShow};

#[entry]
fn main() -> ! {
    hprintln!("Hello world!").unwrap();

    // Get access to peripherals
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32::Peripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();
    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

    // Configure clocks
    let mut flash = dp.FLASH.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let mut delay = Delay::new(cp.SYST, clocks);

    // LED
    // let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);
    // let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    // led.set_high(); // high is off

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

    let spi = Spi::spi1(
        dp.SPI1,
        (sck, miso, mosi),
        &mut afio.mapr,
        Mode { // mode 0
            phase: Phase::CaptureOnFirstTransition,
            polarity: Polarity::IdleLow,
        },
        1.mhz(),
        clocks,
        &mut rcc.apb2,
    );

    // let mut settings = CircleShowSettings::default();
    // settings.brightness = 20;
    // let mut demo = CircleShow::new(&settings);
    let mut demo = FlashyShow::new(&());
    let mut strip = DotstarStrip::new(spi);
    let mut lights = [ColorRgb { r: 0, g: 0, b: 0 }; 100];

    let period = 10u32;
    let mut duration = Duration::Millis(0);
    loop {
        delay.delay_ms(500_u16);
        hprintln!("{}", encoder.count()).unwrap();
        if button.is_low() {
            hprintln!("button down!").unwrap();
        }

        if duration.is_zero() {
            // hprintln!("before").unwrap();
            duration = demo.next(&mut lights);
            // hprintln!("after").unwrap();
            strip.send(&lights).expect("failed to send");
        }
        delay.delay_ms(period);
        duration.subtract(period);
    }
}
