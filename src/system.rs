use core::cell::RefCell;
use core::sync::atomic::{AtomicUsize, Ordering};

use cortex_m_rt::exception;

use cortex_m::interrupt::{self, Mutex};
use cortex_m::peripheral::syst::SystClkSource;

use stm32f1xx_hal::{
    gpio::{
        gpioa::{PA0, PA1, PA10, PA6, PA7, PA8, PA9},
        gpiob::{PB10, PB11, PB12, PB13, PB14, PB15, PB3, PB4, PB5, PB6, PB7, PB8, PB9},
        Alternate, Floating, Input, PullUp, PushPull,
    },
    prelude::*,
    qei::Qei,
    spi::{Mode, Phase, Polarity, Spi},
    stm32::{self, SPI1, TIM2, TIM3, TIM4},
};

use dotstar::{ColorRgb, DotstarStrip};

use crate::controls::{Button, Encoder, Selector};

type DotstarSPI = Spi<
    SPI1,
    (
        PB3<Alternate<PushPull>>,
        PB4<Input<Floating>>,
        PB5<Alternate<PushPull>>,
    ),
>;

type ModeSelector = Selector<
    PB8<Input<PullUp>>,
    PB9<Input<PullUp>>,
    PB10<Input<PullUp>>,
    PB11<Input<PullUp>>,
    PB12<Input<PullUp>>,
    PB13<Input<PullUp>>,
    PB14<Input<PullUp>>,
    PB15<Input<PullUp>>,
>;

static GLOBAL_MILLIS: AtomicUsize = AtomicUsize::new(0);
static BUTTON0: Mutex<RefCell<Option<Button<PA8<Input<PullUp>>>>>> = Mutex::new(RefCell::new(None));
// TODO check button pins
static BUTTON1: Mutex<RefCell<Option<Button<PA9<Input<PullUp>>>>>> = Mutex::new(RefCell::new(None));
static BUTTON2: Mutex<RefCell<Option<Button<PA10<Input<PullUp>>>>>> =
    Mutex::new(RefCell::new(None));

pub struct System {
    strip: DotstarStrip<DotstarSPI>,
    pub encoder0: Encoder<Qei<TIM2, (PA0<Input<Floating>>, PA1<Input<Floating>>)>>,
    pub encoder1: Encoder<Qei<TIM3, (PA6<Input<Floating>>, PA7<Input<Floating>>)>>,
    pub encoder2: Encoder<Qei<TIM4, (PB6<Input<Floating>>, PB7<Input<Floating>>)>>,
    pub mode_selector: ModeSelector,
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

        // Get SPI pins
        let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);
        let sck = gpiob.pb3.into_alternate_push_pull(&mut gpiob.crl);
        let miso = gpiob.pb4;
        let mosi = gpiob.pb5.into_alternate_push_pull(&mut gpiob.crl);

        // Get quadrature encoder pins
        let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
        let encoder0 = Encoder::new(Qei::tim2(
            dp.TIM2,
            (gpioa.pa0, gpioa.pa1),
            &mut afio.mapr,
            &mut rcc.apb1,
        ));

        let encoder1 = Encoder::new(Qei::tim3(
            dp.TIM3,
            (gpioa.pa6, gpioa.pa7),
            &mut afio.mapr,
            &mut rcc.apb1,
        ));

        let encoder2 = Encoder::new(Qei::tim4(
            dp.TIM4,
            (gpiob.pb6, gpiob.pb7),
            &mut afio.mapr,
            &mut rcc.apb1,
        ));

        let mode_selector = ModeSelector::new(
            gpiob.pb8.into_pull_up_input(&mut gpiob.crh),
            gpiob.pb9.into_pull_up_input(&mut gpiob.crh),
            gpiob.pb10.into_pull_up_input(&mut gpiob.crh),
            gpiob.pb11.into_pull_up_input(&mut gpiob.crh),
            gpiob.pb12.into_pull_up_input(&mut gpiob.crh),
            gpiob.pb13.into_pull_up_input(&mut gpiob.crh),
            gpiob.pb14.into_pull_up_input(&mut gpiob.crh),
            gpiob.pb15.into_pull_up_input(&mut gpiob.crh),
        );

        // Create push-button
        let button0 = Button::new(gpioa.pa8.into_pull_up_input(&mut gpioa.crh));
        let button1 = Button::new(gpioa.pa9.into_pull_up_input(&mut gpioa.crh));
        let button2 = Button::new(gpioa.pa10.into_pull_up_input(&mut gpioa.crh));
        interrupt::free(|cs| {
            BUTTON0.borrow(cs).replace(Some(button0));
            BUTTON1.borrow(cs).replace(Some(button1));
            BUTTON2.borrow(cs).replace(Some(button2));
        });

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
            encoder0,
            encoder1,
            encoder2,
            mode_selector,
        }
    }
    pub const fn num_encoders() -> usize {
        3
    }
    pub const fn num_buttons() -> usize {
        3
    }
    pub fn encoder_moved(&mut self, encoder_num: usize) -> Option<i16> {
        // Poll the button
        match encoder_num {
            0 => self.encoder0.clicks_moved(),
            1 => self.encoder1.clicks_moved(),
            2 => self.encoder2.clicks_moved(),
            _ => panic!("Invalid encoder"),
        }
    }

    pub fn button_pressed(&mut self, which_button: usize) -> bool {
        interrupt::free(|cs| {
            match which_button {
                0 => BUTTON0
                    .borrow(cs)
                    .borrow_mut()
                    .as_mut()
                    .expect("button pin must be set before use")
                    .was_pressed(),
                1 => BUTTON1
                    .borrow(cs)
                    .borrow_mut()
                    .as_mut()
                    .expect("button pin must be set before use")
                    .was_pressed(),
                2 => BUTTON2
                    .borrow(cs)
                    .borrow_mut()
                    .as_mut()
                    .expect("button pin must be set before use")
                    .was_pressed(),
                // TODO is it bad to panic in an interrupt-free context?
                _ => panic!("Invalid button"),
            }
        })
    }

    pub fn get_millis(&self) -> u32 {
        GLOBAL_MILLIS.load(Ordering::Relaxed) as u32
    }

    pub fn send(&mut self, lights: &[ColorRgb]) {
        self.strip.send(lights).expect("Failed to send lights");
    }
}

#[exception]
fn SysTick() {
    GLOBAL_MILLIS.fetch_add(1, Ordering::Relaxed);

    interrupt::free(|cs| {
        BUTTON0
            .borrow(cs)
            .borrow_mut()
            .as_mut()
            .expect("button pin must be set before interrupt is enabled")
            .sample();
        BUTTON1
            .borrow(cs)
            .borrow_mut()
            .as_mut()
            .expect("button pin must be set before interrupt is enabled")
            .sample();
        BUTTON2
            .borrow(cs)
            .borrow_mut()
            .as_mut()
            .expect("button pin must be set before interrupt is enabled")
            .sample();
    })
}
