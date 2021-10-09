#![feature(prelude_import)]
#![no_main]
#![no_std]
#[prelude_import]
use core::prelude::rust_2018::*;
#[macro_use]
extern crate core;
extern crate alloc;
use core::mem::MaybeUninit;
use embedded_rust_h2al::{Component, ComponentsBuilder};
use embedded_rust_hardware_init::device_config;
struct BluePill;
impl BluePill {
    fn init() -> (
        &'static mut (
            stm32f1xx_hal::gpio::gpioa::PA0<
                stm32f1xx_hal::gpio::Input<stm32f1xx_hal::gpio::PullUp>,
            >,
        ),
        &'static mut (
            stm32f1xx_hal::gpio::gpioc::PC13<
                stm32f1xx_hal::gpio::Output<stm32f1xx_hal::gpio::PushPull>,
            >,
        ),
        &'static mut (),
    ) {
        use core::mem::MaybeUninit;
        use stm32f1xx_hal::prelude::*;
        use stm32f1xx_hal::gpio::{self, Edge, ExtiPin};
        use stm32f1xx_hal::timer::{self, Timer};
        use stm32f1xx_hal::pwm::{self, PwmChannel};
        use stm32f1xx_hal::pac;
        use stm32f1xx_hal::serial::{self, Config};
        let peripherals = stm32f1xx_hal::pac::Peripherals::take().unwrap();
        let mut flash = peripherals.FLASH.constrain();
        let mut rcc = peripherals.RCC.constrain();
        let cfgr = rcc.cfgr;
        let cfgr = cfgr.sysclk(36000000u32.hz());
        let clocks = cfgr.freeze(&mut flash.acr);
        let mut afio = peripherals.AFIO.constrain(&mut rcc.apb2);
        let mut gpioa = peripherals.GPIOA.split(&mut rcc.apb2);
        let mut gpioc = peripherals.GPIOC.split(&mut rcc.apb2);
        let mut pa0 = gpioa.pa0.into_pull_up_input(&mut gpioa.crl);
        pa0.make_interrupt_source(&mut afio);
        pa0.trigger_on_edge(&peripherals.EXTI, Edge::FALLING);
        pa0.enable_interrupt(&peripherals.EXTI);
        let mut pc13 = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
        static mut INPUT_PINS: MaybeUninit<(
            stm32f1xx_hal::gpio::gpioa::PA0<
                stm32f1xx_hal::gpio::Input<stm32f1xx_hal::gpio::PullUp>,
            >,
        )> = MaybeUninit::uninit();
        static mut OUTPUT_PINS: MaybeUninit<(
            stm32f1xx_hal::gpio::gpioc::PC13<
                stm32f1xx_hal::gpio::Output<stm32f1xx_hal::gpio::PushPull>,
            >,
        )> = MaybeUninit::uninit();
        static mut PWM_PINS: MaybeUninit<()> = MaybeUninit::uninit();
        static mut CHANNELS: MaybeUninit<()> = MaybeUninit::uninit();
        static mut TIMERS: MaybeUninit<()> = MaybeUninit::uninit();
        unsafe {
            (
                (INPUT_PINS.write((pa0,))),
                (OUTPUT_PINS.write((pc13,))),
                (PWM_PINS.write(())),
            )
        }
    }
    #[inline]
    fn enable_interrupts() {
        unsafe {
            stm32f1xx_hal::pac::NVIC::unmask(stm32f1xx_hal::pac::Interrupt::EXTI0);
        }
    }
}
fn main() -> ! {
    static mut ca: [MaybeUninit<Component>; 2] = ComponentsBuilder::allocate_array();
    let components = BluePill::init();
    let mut cb = ComponentsBuilder::new(unsafe { &mut ca });
    cb.add_input_pin(&mut components.0 .0);
    cb.add_output_pin(&mut components.1 .0);
    loop {}
}
pub async fn test_task() {
    loop {}
}
