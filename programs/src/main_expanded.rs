#![feature(prelude_import)]
#![no_main]
#![no_std]
#[prelude_import]
use core::prelude::rust_2018::*;
#[macro_use]
extern crate core;
extern crate alloc;
use embedded_rust_hardware_init::device_config;
struct BluePill;
impl BluePill {
    fn init() -> (
        (
            stm32f1xx_hal::gpio::gpioa::PA0<
                stm32f1xx_hal::gpio::Input<stm32f1xx_hal::gpio::PullUp>,
            >,
        ),
        (
            stm32f1xx_hal::gpio::gpioc::PC13<
                stm32f1xx_hal::gpio::Output<stm32f1xx_hal::gpio::PushPull>,
            >,
        ),
        (),
        (),
        (),
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
        unsafe { INPUT_PINS.write((pa0,)) };
        static mut OUTPUT_PINS: MaybeUninit<(
            stm32f1xx_hal::gpio::gpioc::PC13<
                stm32f1xx_hal::gpio::Output<stm32f1xx_hal::gpio::PushPull>,
            >,
        )> = MaybeUninit::uninit();
        unsafe { OUTPUT_PINS.write((pc13,)) };
        static mut PWM_PINS: MaybeUninit<()> = MaybeUninit::uninit();
        unsafe { PWM_PINS.write(()) };
        static mut CHANNELS: MaybeUninit<()> = MaybeUninit::uninit();
        unsafe { CHANNELS.write(()) };
        static mut TIMERS: MaybeUninit<()> = MaybeUninit::uninit();
        unsafe { TIMERS.write(()) };
        unsafe {
            (
                INPUT_PINS.assume_init(),
                OUTPUT_PINS.assume_init(),
                PWM_PINS.assume_init(),
                CHANNELS.assume_init(),
                TIMERS.assume_init(),
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
    loop {}
}
pub async fn test_task() {
    loop {}
}
