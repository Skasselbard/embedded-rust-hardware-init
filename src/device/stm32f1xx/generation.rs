use std::{
    borrow::BorrowMut, cell::RefCell, collections::HashSet, fmt::format, ops::Deref, rc::Rc,
};

use quote::format_ident;
use syn::{parse_quote, parse_str, Ident, Stmt};

use crate::device::{
    stm32f1xx::{InterruptMode, PinMode},
    DeviceConfig, Hertz,
};

use super::{Gpio, Pin, Port, Stm32f1xxPeripherals};

pub trait InitializedComponent {
    fn ty(&self) -> syn::Type;
    fn identifier(&self) -> Ident;
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct InitializedGpio {
    pin: Pin,
    port: Port,
    mode: PinMode,
    id: Ident,
}

impl InitializedComponent for InitializedGpio {
    fn ty(&self) -> syn::Type {
        let port_name = self.port.lower();
        let pin_type = gpio_short_type(self.pin, self.port);
        let direction = self.mode.direction_name();
        let mode = self.mode.mode_name();
        parse_str(&format!(
            "stm32f1xx_hal::gpio::{}::{}<stm32f1xx_hal::gpio::{}<stm32f1xx_hal::gpio::{}>>",
            port_name, pin_type, direction, mode
        ))
        .unwrap()
    }
    fn identifier(&self) -> Ident {
        self.id.clone()
    }
}

// FIXME: delete
impl InitializedComponent for () {
    fn ty(&self) -> syn::Type {
        panic!("used '()' as component")
    }
    fn identifier(&self) -> Ident {
        panic!("used '()' as component")
    }
}

pub struct DeviceInit {
    init_block: Vec<syn::Stmt>,
    peripherals: Ident,
    flash: Ident,
    rcc: Option<Ident>, // FIXME: should be a state machine to reflect consumption on 'freeze(self, ...)'
    cfgr: Option<Ident>,
    afio: Option<Ident>,
    clocks: Option<(Ident, Hertz)>,
    gpios: Option<RefCell<HashSet<Ident>>>,
}

fn gpio_ident(pin: Pin, port: Port) -> Ident {
    format_ident!("p{}{}", port.short(), pin.0)
}
fn gpio_short_type(pin: Pin, port: Port) -> Ident {
    format_ident!("P{}{}", port.short().to_uppercase().to_string(), pin.0)
}

impl DeviceInit {
    fn new() -> Self {
        let peripherals = format_ident!("peripherals");
        let flash = format_ident!("flash");
        let init_block = parse_quote!(
            use stm32f1xx_hal::prelude::*;
            let #peripherals = stm32f1xx_hal::pac::Peripherals::take().unwrap();
            let mut #flash = #peripherals.FLASH.constrain();
        );
        Self {
            init_block,
            peripherals: peripherals,
            flash: flash,
            rcc: None,
            cfgr: None,
            afio: None,
            clocks: None,
            gpios: None,
        }
    }
    fn rcc(&mut self) -> Ident {
        if self.rcc.is_none() {
            let peripherals_ident = &self.peripherals;
            let rcc_ident = format_ident!("rcc");
            self.init_block
                .push(parse_quote!(let mut #rcc_ident = #peripherals_ident.RCC.constrain();));
            self.rcc = Some(rcc_ident);
        }
        self.rcc.as_ref().unwrap().clone()
    }
    fn cfgr(&mut self) -> Ident {
        if self.cfgr.is_none() {
            if self.clocks.is_some() {
                panic!("cfgr initialization after freeze")
            }
            let rcc_ident = self.rcc();
            let cfgr_ident = format_ident!("cfgr");
            self.init_block.push(parse_quote!(
              let #cfgr_ident = #rcc_ident.cfgr;
            ));
            self.cfgr = Some(cfgr_ident);
        }
        self.cfgr.as_ref().unwrap().clone()
    }
    fn afio(&mut self) -> Ident {
        if self.afio.is_none() {
            let rcc_ident = self.rcc();
            let peripherals_ident = &self.peripherals;
            let afio_ident = format_ident!("afio");
            self.init_block.push(parse_quote!(
                let mut #afio_ident = #peripherals_ident.AFIO.constrain(&mut #rcc_ident.apb2);
            ));
            self.afio = Some(afio_ident);
        }
        self.afio.as_ref().unwrap().clone()
    }
    fn clocks(&mut self, clock: Hertz) -> Ident {
        if self.clocks.is_none() {
            let freq = clock.0;
            let cfgr_ident = self.cfgr();
            let flash_ident = &self.flash;
            let clocks_ident = format_ident!("clocks");
            self.init_block.append(&mut parse_quote!(
                let #cfgr_ident = #cfgr_ident.sysclk(#freq.hz());
                let #clocks_ident = #cfgr_ident.freeze(&mut #flash_ident.acr);
            ));
            self.cfgr = None;
            self.clocks = Some((clocks_ident, clock));
        }
        self.clocks.as_ref().unwrap().0.clone()
    }
    fn gpios(&mut self, peripheral_config: &Stm32f1xxPeripherals) -> RefCell<HashSet<Ident>> {
        if self.gpios.is_none() {
            let gpios = peripheral_config.used_gpios();
            let rcc_ident = self.rcc();
            let peripherals_ident = &self.peripherals;
            // First initialize the gpio ports
            let mut ports = gpios.iter().map(|(_, port)| port).collect::<Vec<&Port>>();
            ports.dedup();
            for port in ports {
                let port_lower = format_ident!("{}", port.lower());
                let port_upper = format_ident!("{}", port.upper());
                // expand: let mut gpiox = peripherals.GPIOX.split(&mut rcc.apb2);
                // its always apb2 on this boards
                self.init_block.push(parse_quote!(
                let mut #port_lower = #peripherals_ident.#port_upper.split(&mut #rcc_ident.apb2);
            ))
            }
            // remember all gpios and check for duplicates
            let mut gpios_idents = HashSet::new();
            for (pin, port) in gpios {
                if pin.0 > 15 {
                    panic!("Gpio pins are numbered from 0 to 15")
                }
                if !gpios_idents.insert(gpio_ident(pin, port)) {
                    panic!("Gpio '{}' is used multiple times", gpio_ident(pin, port));
                }
            }
            self.gpios = Some(RefCell::new(gpios_idents));
        }
        self.gpios.as_mut().unwrap().clone()
    }
    fn inputs(&mut self, peripheral_config: &Stm32f1xxPeripherals) -> HashSet<InitializedGpio> {
        let mut idents = HashSet::new();
        let inputs = &peripheral_config.gpio.input;
        let peripherals_ident = self.peripherals.clone();
        let gpio_pool = self.gpios(&peripheral_config);
        for gpio in inputs {
            let port_ident = format_ident!("{}", gpio.port.lower());
            let gpio_ident = gpio_pool
                .borrow_mut()
                .take(&gpio_ident(gpio.pin, gpio.port))
                .expect("Use of uninitialized gpio");
            let pin_name = &gpio_ident; // Its only equal because we name the identifiers equally
            let control_reg = format_ident!("{}", gpio.pin.control_reg());
            let init_function_name = format_ident!("{}", gpio.mode.init_function_name());
            self.init_block.push(parse_quote!(
                let mut #gpio_ident = #port_ident.#pin_name.#init_function_name(&mut #port_ident.#control_reg);
            ));
            match gpio.interrupt_mode {
                InterruptMode::None => {}
                other => {
                    let edge_ident = format_ident!("{}", other.ident());
                    let afio_ident = self.afio();

                    // expand:
                    // pin_pxy.make_interrupt_source(&mut afio);
                    // pin_pxy.trigger_on_edge(&peripherals.EXTI, Edge::EDGE_TYPE);
                    // pin_pxy.enable_interrupt(&peripherals.EXTI);
                    self.init_block.append(&mut parse_quote!(
                        #gpio_ident.make_interrupt_source(&mut #afio_ident);
                        #gpio_ident.trigger_on_edge(&#peripherals_ident.EXTI, stm32f1xx_hal::gpio::Edge::#edge_ident);
                        #gpio_ident.enable_interrupt(&#peripherals_ident.EXTI);
                    ));
                }
            }
            idents.insert(InitializedGpio {
                pin: gpio.pin,
                port: gpio.port,
                id: gpio_ident,
                mode: gpio.mode,
            });
        }
        idents
    }
    fn outputs(&mut self, peripheral_config: &Stm32f1xxPeripherals) -> HashSet<InitializedGpio> {
        let mut idents = HashSet::new();
        let outputs = &peripheral_config.gpio.output;
        let gpio_pool = self.gpios(peripheral_config);
        for gpio in outputs {
            let port_ident = format_ident!("{}", gpio.port.lower());
            let gpio_ident = gpio_pool
                .borrow_mut()
                .take(&gpio_ident(gpio.pin, gpio.port))
                .expect("Use of uninitialized gpio");
            let pin_name = &gpio_ident; // Its only equal because we name the identifiers equally
            let control_reg = format_ident!("{}", gpio.pin.control_reg());
            let init_function_name = format_ident!("{}", gpio.mode.init_function_name());
            self.init_block.push(parse_quote!(
                let mut #gpio_ident = #port_ident.#pin_name.#init_function_name(&mut #port_ident.#control_reg);
            ));
            idents.insert(InitializedGpio {
                pin: gpio.pin,
                port: gpio.port,
                id: gpio_ident,
                mode: gpio.mode,
            });
        }
        idents
    }

    fn static_init_and_return(
        self,
        inputs: &HashSet<InitializedGpio>,
        outputs: &HashSet<InitializedGpio>,
        timer: &HashSet<()>,
        pwm: &HashSet<()>,
        serial: &HashSet<()>,
    ) -> (Vec<Stmt>, syn::Type) {
        const IN_IDENT: &str = "INPUT_PINS";
        const OUT_IDENT: &str = "OUTPUT_PINS";
        const TIMER_IDENT: &str = "TIMERS";
        const PWM_IDENT: &str = "PWM_PINS";
        const SERIAL_IDENT: &str = "SERIALS";
        fn static_init_stmt(static_name: &str, types: &Vec<syn::Type>) -> Stmt {
            let static_name = format_ident!("{}", static_name);
            parse_quote!(
                static mut #static_name: MaybeUninit<(#(#types,)*)> = MaybeUninit::uninit();
            )
        }
        fn write_static(static_name: &str, identifiers: &Vec<Ident>) -> syn::Expr {
            let static_name = format_ident!("{}", static_name);
            parse_quote!(
                #static_name.write((#(#identifiers,)*))
            )
        }
        let in_tys = inputs.iter().map(|gpio| gpio.ty()).collect();
        let out_tys = outputs.iter().map(|gpio| gpio.ty()).collect();
        let timer_tys = timer.iter().map(|timer| timer.ty()).collect();
        let pwm_tys = pwm.iter().map(|pwm| pwm.ty()).collect();
        let serial_tys = serial.iter().map(|serial| serial.ty()).collect();

        let in_ids = inputs.iter().map(|gpio| gpio.identifier()).collect();
        let out_ids = outputs.iter().map(|gpio| gpio.identifier()).collect();
        let timer_ids = timer.iter().map(|timer| timer.identifier()).collect();
        let pwm_ids = pwm.iter().map(|pwm| pwm.identifier()).collect();
        let serial_ids = serial.iter().map(|serial| serial.identifier()).collect();

        let mut stmts = self.init_block;

        stmts.push(static_init_stmt(IN_IDENT, &in_tys));
        stmts.push(static_init_stmt(OUT_IDENT, &out_tys));
        stmts.push(static_init_stmt(TIMER_IDENT, &timer_tys));
        stmts.push(static_init_stmt(PWM_IDENT, &pwm_tys));
        stmts.push(static_init_stmt(SERIAL_IDENT, &serial_tys));

        let init_ins = write_static(IN_IDENT, &in_ids);
        let init_outs = write_static(OUT_IDENT, &out_ids);
        let init_timer = write_static(TIMER_IDENT, &timer_ids);
        let init_pwm = write_static(PWM_IDENT, &pwm_ids);
        let init_serial = write_static(SERIAL_IDENT, &serial_ids);

        stmts.push(parse_quote!(
            unsafe{(
                (#init_ins),
                (#init_outs),
                (#init_timer),
                (#init_pwm),
                (#init_serial),
            )}
        ));
        let return_types = parse_quote!(
            (
                (#(#in_tys,)*),
                (#(#out_tys,)*),
                (#(#timer_tys,)*),
                (#(#pwm_tys,)*),
                (#(#serial_tys,)*),
            )
        );
        (stmts, return_types)
    }

    pub(crate) fn get_init_block(config: &DeviceConfig) -> (Vec<Stmt>, syn::Type) {
        let peripheral_config = match &config.kind {
            crate::device::DeviceKind::Stm32f1xx(pc) => pc,
            _ => panic!("Tried to build stm32f1xx config from other device kind"),
        };
        let mut device_init = DeviceInit::new();
        device_init.clocks(config.clock); // TODO: may change to take a config
        let inputs = device_init.inputs(&peripheral_config);
        let outputs = device_init.outputs(&peripheral_config);
        let timer = HashSet::new();
        let pwm = HashSet::new();
        let serial = HashSet::new();
        device_init.static_init_and_return(&inputs, &outputs, &timer, &pwm, &serial)
    }
}
