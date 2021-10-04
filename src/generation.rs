use crate::{config::Config, types::Serial, Frequency, Gpio, PWMInterface};
use syn::{parse_quote, parse_str, Expr, ExprUnsafe, Ident, Stmt, Type};

/// The Generator trait is used to determine the proper generation functions
/// It is just a meta trait that combines all special generation traits.
pub trait Generator:
    DeviceGeneration + GpioGeneration + SysGeneration + PWMGeneration + SerialGeneration
{
}
pub trait DeviceGeneration {
    /// Everything that should be used in the device init function with
    /// a ``use crate::pa::th`` statement.
    fn generate_imports(&self) -> Vec<Stmt>;
    /// Here you can add functions to prepare the general device
    /// and introduce variable names for later use
    /// For example the stm32f1xx boards need access to a peripheral
    /// singleton and initialized flash.
    fn generate_device_init(&self) -> Vec<Stmt>;
    /// In the stm32f1 hal, each pin channel ('A' to 'E' in the pin types PAX to PEX)
    /// has to be initialized to initialize the actual pins
    /// this is done with these statements.
    /// A function to get the channel name is included in the Pin trait.
    /// A function to get the pin is included in the Gpio trait.
    fn generate_channels(&self, gpios: &Vec<Box<dyn Gpio>>) -> Vec<Stmt>;
}
pub trait GpioGeneration {
    /// In this function all pins should be introduced with a let binding.
    /// The identifiers for the pins should be generated with the identifier
    /// function of the Gpio trait (or rather its Component trait bound).
    /// The identifiers will later be used to populate the global data statics.
    ///
    /// All other gpio dependent initializations (like gpio interrupts) should go
    /// here as well.
    fn generate_gpios(&self, gpios: &Vec<Box<dyn Gpio>>) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        for gpio in gpios {
            stmts.append(&mut gpio.generate());
        }
        stmts
    }
    /// This function should return all gpio interrupts that should be enabled.
    /// For the Stm32f1 boards this would be the appropriate Exti_X (External
    /// Interrupt) lines
    fn interrupts(&self, gpios: &Vec<Box<dyn Gpio>>) -> Vec<Stmt>;
}
pub trait PWMGeneration {
    fn generate_pwm_pins(&self, pwms: &Vec<&dyn PWMInterface>) -> Vec<Stmt> {
        let mut stmts = vec![];
        for pwm in pwms {
            stmts.append(&mut pwm.generate());
        }
        stmts
    }
}
pub trait SerialGeneration {
    fn generate_serials(&self, serials: &Vec<&dyn Serial>) -> Vec<Stmt> {
        let mut stmts = vec![];
        for serial in serials {
            stmts.append(&mut serial.generate())
        }
        stmts
    }
}
pub trait SysGeneration {
    /// With this function statements for board speed are generated
    /// These statements go right after the device init statements
    fn generate_clock(&self, sys_frequency: &Option<Frequency>) -> Vec<Stmt>;
}

//TODO: shorten
macro_rules! define_static {
    ($static_name:expr, $types:expr, $identifiers:expr) => {{
        let tys: &Vec<Type> = $types;
        let static_name = quote::format_ident!("{}", $static_name);
        let identifiers: &Vec<Ident> = $identifiers;
        let src: Vec<Stmt> = syn::parse_quote!(
            static mut #static_name: MaybeUninit<(#(#tys,)*)> = MaybeUninit::uninit();
            unsafe{#static_name.write((#(#identifiers,)*))};
        );
        src
    }};
}

pub(crate) fn component_statics(config: &Config) -> Vec<Stmt> {
    let mut stmts = vec![];
    stmts.append(&mut define_static!(
        "INPUT_PINS",
        &config.input_tys(),
        &config.input_idents()
    ));
    stmts.append(&mut define_static!(
        "OUTPUT_PINS",
        &config.output_tys(),
        &config.output_idents()
    ));
    stmts.append(&mut define_static!(
        "PWM_PINS",
        &config.pwm_tys(),
        &config.pwm_idents()
    ));
    stmts.append(&mut define_static!("CHANNELS", &vec![], &vec![]));
    // stmts.append(&mut define_static!(
    //     "SERIALS",
    //     "Serial",
    //     &config.serial_rx_tys(),
    //     &config.serial_tx_tys()
    // ));
    stmts.append(&mut define_static!("TIMERS", &vec![], &vec![]));
    stmts.into()
}

pub(crate) fn init_retutn_statement(config: &Config) -> ExprUnsafe {
    syn::parse_quote!(unsafe {
        (
            INPUT_PINS.assume_init(),
            OUTPUT_PINS.assume_init(),
            PWM_PINS.assume_init(),
            CHANNELS.assume_init(),
            TIMERS.assume_init(),
        )
    })
}
pub(crate) fn init_retutn_type(config: &Config) -> Type {
    let input_types = &config.input_tys();
    let output_types = &config.output_tys();
    let pwm_types = &config.pwm_tys();
    let channels_types: &Vec<Type> = &vec![];
    let timer_types: &Vec<Type> = &vec![];
    syn::parse_quote!((
        (#(#input_types, )*),
        (#(#output_types,)*),
        (#(#pwm_types,)*),
        (#(#channels_types,)*),
        (#(#timer_types,)*),
    ))
}