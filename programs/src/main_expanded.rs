#![feature(prelude_import)]
#![no_main]
#![no_std]
#[prelude_import]
use core::prelude::v1::*;
#[macro_use]
extern crate core;
extern crate alloc;
use alloc::boxed::Box;
use futures::stream;
use futures::StreamExt;
use cortex_m_rt::entry;
use embedded_rust::Task;
use embedded_rust_macros::*;
struct BluePill;
impl BluePill {
    #[inline]
    fn init() {
        use stm32f1xx_hal::gpio::{Edge, ExtiPin};
        use stm32f1xx_hal::prelude::*;
        use embedded_rust::device::{InputPin, OutputPin, Pin, Port, Channel};
        use embedded_rust::resources::{Resource};
        use embedded_rust::Runtime;
        let peripherals = stm32f1xx_hal::pac::Peripherals::take().unwrap();
        let mut flash = peripherals.FLASH.constrain();
        let mut rcc = peripherals.RCC.constrain();
        let cfgr = rcc.cfgr;
        let cfgr = cfgr.sysclk(36000000u32.hz());
        let clocks = cfgr.freeze(&mut flash.acr);
        let mut afio = peripherals.AFIO.constrain(&mut rcc.apb2);
        let mut gpioa = peripherals.GPIOA.split(&mut rcc.apb2);
        let mut gpioc = peripherals.GPIOC.split(&mut rcc.apb2);
        let mut pin_pa0 = gpioa.pa0.into_pull_up_input(&mut gpioa.crl);
        pin_pa0.make_interrupt_source(&mut afio);
        pin_pa0.trigger_on_edge(&peripherals.EXTI, Edge::FALLING);
        pin_pa0.enable_interrupt(&peripherals.EXTI);
        let mut pin_pc13 = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
        let mut pin_pa2 = gpioa.pa2.into_pull_down_input(&mut gpioa.crl);
        pin_pa2.make_interrupt_source(&mut afio);
        pin_pa2.trigger_on_edge(&peripherals.EXTI, Edge::RISING);
        pin_pa2.enable_interrupt(&peripherals.EXTI);
        static mut SYS: Option<()> = None;
        static mut INPUT_PINS: Option<(
            InputPin<
                stm32f1xx_hal::gpio::gpioa::PA0<
                    stm32f1xx_hal::gpio::Input<stm32f1xx_hal::gpio::PullUp>,
                >,
            >,
            InputPin<
                stm32f1xx_hal::gpio::gpioa::PA2<
                    stm32f1xx_hal::gpio::Input<stm32f1xx_hal::gpio::PullDown>,
                >,
            >,
        )> = None;
        static mut OUTPUT_PINS: Option<(
            OutputPin<
                stm32f1xx_hal::gpio::gpioc::PC13<
                    stm32f1xx_hal::gpio::Output<stm32f1xx_hal::gpio::PushPull>,
                >,
            >,
        )> = None;
        static mut PWM: Option<()> = None;
        static mut CHANNELS: Option<()> = None;
        static mut SERIALS: Option<()> = None;
        static mut TIMERS: Option<()> = None;
        static mut SYS_ARRAY: Option<[&'static mut dyn Resource; 0usize]> = None;
        static mut INPUT_ARRAY: Option<[&'static mut dyn Resource; 2usize]> = None;
        static mut OUTPUT_ARRAY: Option<[&'static mut dyn Resource; 1usize]> = None;
        static mut PWM_ARRAY: Option<[&'static mut dyn Resource; 0usize]> = None;
        static mut CHANNEL_ARRAY: Option<[&'static mut dyn Resource; 0usize]> = None;
        static mut SERIAL_ARRAY: Option<[&'static mut dyn Resource; 0usize]> = None;
        static mut TIMER_ARRAY: Option<[&'static mut dyn Resource; 0usize]> = None;
        unsafe {
            SYS = Some(());
            INPUT_PINS = Some((
                InputPin::new(Pin::new(Channel::A, Port::P00), pin_pa0),
                InputPin::new(Pin::new(Channel::A, Port::P02), pin_pa2),
            ));
            OUTPUT_PINS = Some((OutputPin::new(Pin::new(Channel::C, Port::P13), pin_pc13),));
            PWM = Some(());
            CHANNELS = Some(());
            SERIALS = Some(());
            TIMERS = Some(());
            let sys = SYS.as_mut().unwrap();
            let input_pins = INPUT_PINS.as_mut().unwrap();
            let output_pins = OUTPUT_PINS.as_mut().unwrap();
            let pwm = PWM.as_mut().unwrap();
            let channels = CHANNELS.as_mut().unwrap();
            let serials = SERIALS.as_mut().unwrap();
            let timers = TIMERS.as_mut().unwrap();
            SYS_ARRAY = Some([]);
            INPUT_ARRAY = Some([&mut input_pins.0, &mut input_pins.1]);
            OUTPUT_ARRAY = Some([&mut output_pins.0]);
            PWM_ARRAY = Some([]);
            CHANNEL_ARRAY = Some([]);
            SERIAL_ARRAY = Some([]);
            TIMER_ARRAY = Some([]);
        }
        unsafe {
            Runtime::init(
                10240usize,
                SYS_ARRAY.as_mut().unwrap(),
                INPUT_ARRAY.as_mut().unwrap(),
                OUTPUT_ARRAY.as_mut().unwrap(),
                PWM_ARRAY.as_mut().unwrap(),
                CHANNEL_ARRAY.as_mut().unwrap(),
                SERIAL_ARRAY.as_mut().unwrap(),
                TIMER_ARRAY.as_mut().unwrap(),
            )
            .expect("Runtime initialization failed");
        }
    }
    #[inline]
    fn get_resource(
        uri: &str,
    ) -> Result<embedded_rust::resources::ResourceID, embedded_rust::RuntimeError> {
        embedded_rust::Runtime::get().get_resource(uri)
    }
    #[inline]
    fn run() -> ! {
        unsafe {
            stm32f1xx_hal::pac::NVIC::unmask(stm32f1xx_hal::pac::Interrupt::EXTI0);
            stm32f1xx_hal::pac::NVIC::unmask(stm32f1xx_hal::pac::Interrupt::EXTI2);
        }
        embedded_rust::Runtime::get().run()
    }
}
#[doc(hidden)]
#[export_name = "main"]
pub unsafe extern "C" fn __cortex_m_rt_main_trampoline() {
    __cortex_m_rt_main()
}
fn __cortex_m_rt_main() -> ! {
    BluePill::init();
    Task::new(test_task()).spawn();
    BluePill::run();
}
pub async fn test_task() {
    let mut button_events = BluePill::get_resource("event:gpio/pa0").unwrap();
    let mut led = BluePill::get_resource("digital:gpio/pc13").unwrap();
    let mut led_light_state = false;
    while let Some(_event) = button_events.read_stream().next().await {
        led_light_state = !led_light_state;
        led.write(Box::pin(stream::once(async { led_light_state as u8 })))
            .await
            .unwrap();
    }
}