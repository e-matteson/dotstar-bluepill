use core::cell::RefCell;
use core::sync::atomic::{AtomicUsize, Ordering};

use cortex_m_rt::exception;

use cortex_m::interrupt::{self, Mutex};
use cortex_m::peripheral::syst::SystClkSource;

use stm32f1xx_hal::{
    gpio::{
        gpioa::{PA0, PA1, PA2, PA5, PA6, PA7},
        Alternate, Floating, Input, PullUp, PushPull,
    },
    prelude::*,
    qei::Qei,
    spi::{Mode, Phase, Polarity, Spi},
    stm32::{self, SPI1, TIM2},
};

use dotstar::{ColorRgb, DotstarStrip};

use crate::button::Button;

type DotstarSPI = Spi<
    SPI1,
    (
        PA5<Alternate<PushPull>>,
        PA6<Input<Floating>>,
        PA7<Alternate<PushPull>>,
    ),
>;

static GLOBAL_MILLIS: AtomicUsize = AtomicUsize::new(0);
static BUTTON_PIN: Mutex<RefCell<Option<Button<PA2<Input<PullUp>>>>>> =
    Mutex::new(RefCell::new(None));

pub struct System {
    strip: DotstarStrip<DotstarSPI>,
    knob_state: u16,
    encoder: Qei<TIM2, (PA0<Input<Floating>>, PA1<Input<Floating>>)>,
}

pub enum EncoderEvent {
    ButtonPress,
    KnobRight,
    KnobLeft,
}

use self::EncoderEvent::*;

impl System {
    pub fn poll_event(&mut self) -> Option<EncoderEvent> {
        // Poll the button
        if self.was_pressed() {
            return Some(ButtonPress);
        }
        // Poll the knob (each tick increments the encoder by 4, so round it).
        let new_knob_state = self.encoder.count();
        let diff = new_knob_state.wrapping_sub(self.knob_state) as i16;
        if diff >= 4 {
            self.knob_state = self.knob_state.wrapping_add(4);
            return Some(KnobRight);
        } else if diff <= -4 {
            self.knob_state = self.knob_state.wrapping_sub(4);
            return Some(KnobLeft);
        }
        // Or maybe nothing's happened.
        None
    }

    pub fn new() -> System {
        // Get access to peripherals
        let cp = cortex_m::Peripherals::take().unwrap();
        let dp = stm32::Peripherals::take().unwrap();
        let mut rcc = dp.RCC.constrain();
        let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

        // Configure clocks
        let mut flash = dp.FLASH.constrain();
        let clocks = rcc.cfgr.freeze(&mut flash.acr);

        // Get SPI pins
        let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
        let sck = gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl);
        let miso = gpioa.pa6;
        let mosi = gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl);

        // Get quadrature encoder pins
        let c1 = gpioa.pa0;
        let c2 = gpioa.pa1;
        let encoder = Qei::tim2(dp.TIM2, (c1, c2), &mut afio.mapr, &mut rcc.apb1);

        // Create push-button
        let button = Button::new(gpioa.pa2.into_pull_up_input(&mut gpioa.crl));
        interrupt::free(|cs| BUTTON_PIN.borrow(cs).replace(Some(button)));

        // Configures the system timer to trigger a SysTick exception every 1 milliseceond
        let mut systick = cp.SYST;
        systick.set_clock_source(SystClkSource::Core);
        systick.set_reload(clocks.sysclk().0 / 1_000);
        systick.enable_counter();
        systick.enable_interrupt();

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
            clocks.pclk2(), // use max possible SPI rate
            clocks,
            &mut rcc.apb2,
        );

        System {
            strip: DotstarStrip::new(spi),
            encoder,
            knob_state: 0,
        }
    }

    pub fn get_millis(&self) -> u32 {
        GLOBAL_MILLIS.load(Ordering::Relaxed) as u32
    }

    pub fn was_pressed(&mut self) -> bool {
        interrupt::free(|cs| {
            BUTTON_PIN
                .borrow(cs)
                .borrow_mut()
                .as_mut()
                .expect("button pin must be set before use")
                .was_pressed()
        })
    }

    pub fn write_lights(&mut self, lights: &[ColorRgb]) {
        self.strip.send(lights).expect("Failed to send lights");
    }
}

#[exception]
fn SysTick() {
    GLOBAL_MILLIS.fetch_add(1, Ordering::Relaxed);

    interrupt::free(|cs| {
        BUTTON_PIN
            .borrow(cs)
            .borrow_mut()
            .as_mut()
            .expect("button pin must be set before interrupt is enabled")
            .sample();
    })
}
