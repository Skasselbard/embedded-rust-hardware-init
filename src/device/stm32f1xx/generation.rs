use std::{collections::HashSet, fmt::format};

use syn::parse_quote;

use crate::device::{
    stm32f1xx::{InterruptMode, PinMode},
    Hertz,
};

use super::{Gpio, Pin, Port};

type Identifier = String;

struct DeviceInit {
    init_block: Vec<syn::Stmt>,
    peripherals: Identifier,
    flash: Identifier,
    rcc: Option<Identifier>, // FIXME: should be a state machine to reflect consumption on 'freeze(self, ...)'
    cfgr: Option<Identifier>,
    afio: Option<Identifier>,
    clocks: Option<(Identifier, Hertz)>,
    gpios: Option<HashSet<Identifier>>,
}

fn gpio_ident(pin: Pin, port: Port) -> String {
    format!("p{}{}", port.short(), pin.0)
}

impl DeviceInit {
    fn new() -> Self {
        let peripherals = "peripherals";
        let flash = "flash";
        let init_block = parse_quote!(
            let #peripherals = stm32f1xx_hal::pac::Peripherals::take().unwrap();
            let mut #flash = #peripherals.FLASH.constrain();
        );
        Self {
            init_block,
            peripherals: peripherals.to_string(),
            flash: flash.to_string(),
            rcc: None,
            cfgr: None,
            afio: None,
            clocks: None,
            gpios: None,
        }
    }
    fn rcc(&mut self) -> Identifier {
        match &self.rcc {
            Some(ident) => ident,
            None => {
                let peripherals_ident = &self.peripherals;
                let rcc_ident = "rcc";
                self.init_block
                    .push(parse_quote!(let mut #rcc_ident = #peripherals_ident.RCC.constrain();));
                self.rcc = Some(rcc_ident.to_string());
                rcc_ident
            }
        }
        .to_string()
    }
    fn cfgr(&mut self) -> Identifier {
        match &self.cfgr {
            Some(ident) => ident,
            None => {
                if self.clocks.is_some() {
                    panic!("cfgr initialization after freeze")
                }
                let rcc_ident = self.rcc();
                let cfgr_ident = "cfgr";
                self.init_block.push(parse_quote!(
                  let #cfgr_ident = #rcc_ident.cfgr;
                ));
                self.cfgr = Some(cfgr_ident.to_string());
                cfgr_ident
            }
        }
        .to_string()
    }
    fn afio(&mut self) -> Identifier {
        match &self.afio {
            Some(ident) => ident,
            None => {
                let rcc_ident = self.rcc();
                let peripherals_ident = &self.peripherals;
                let afio_ident = "afio";
                self.init_block.push(parse_quote!(
                    let mut #afio_ident = #peripherals_ident.AFIO.constrain(&mut #rcc_ident.apb2);
                ));
                self.afio = Some(afio_ident.to_string());
                afio_ident
            }
        }
        .to_string()
    }
    fn clocks(&mut self, clock: Hertz) -> Identifier {
        match &self.clocks {
            Some((ident, _)) => ident,
            None => {
                let freq = clock.0;
                let cfgr_ident = self.cfgr();
                let flash_ident = &self.flash;
                let clocks_ident = "clocks";
                self.init_block.append(&mut parse_quote!(
                    let #cfgr_ident = #cfgr_ident.sysclk(#freq.hz());
                    let #clocks_ident = #cfgr_ident.freeze(&mut #flash_ident.acr);
                ));
                self.cfgr = None;
                self.clocks = Some((clocks_ident.to_string(), clock));
                clocks_ident
            }
        }
        .to_string()
    }
    fn gpios(&mut self, gpios: Vec<(Pin, Port)>) -> &HashSet<String> {
        if self.gpios.is_none() {
            let rcc_ident = self.rcc();
            let peripherals_ident = &self.peripherals;
            // First initialize the gpio ports
            let mut ports = gpios.iter().map(|(_, port)| port).collect::<Vec<&Port>>();
            ports.dedup();
            for port in ports {
                let port_lower = port.lower();
                let port_upper = port.upper();
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
            self.gpios = Some(gpios_idents);
        }
        self.gpios.as_ref().unwrap()
    }
    fn inputs(&mut self, inputs: Vec<Gpio>) -> HashSet<String> {
        let mut idents = HashSet::new();
        let gpio_pool = self
            .gpios
            .as_mut()
            .expect("Ports and gpios are not initialized");
        let peripherals_ident = &self.peripherals;
        for gpio in inputs {
            let port_ident = gpio.port.lower();
            let gpio_ident = gpio_pool
                .take(&gpio_ident(gpio.pin, gpio.port))
                .expect("Use of uninitialized gpio");
            let pin_name = &gpio_ident; // Its only equal because we name the identifiers equally
            let control_reg = gpio.pin.control_reg();
            let init_function_name = gpio.mode.init_function_name();
            self.init_block.push(parse_quote!(
                let mut #gpio_ident = #port_ident.#pin_name.#init_function_name(&mut #port_ident.#control_reg);
            ));
            match gpio.interrupt_mode {
                InterruptMode::None => {}
                other => {
                    let edge_ident = other.ident();
                    // expand:
                    // pin_pxy.make_interrupt_source(&mut afio);
                    // pin_pxy.trigger_on_edge(&peripherals.EXTI, Edge::EDGE_TYPE);
                    // pin_pxy.enable_interrupt(&peripherals.EXTI);
                    self.init_block.append(&mut parse_quote!(
                        #gpio_ident.make_interrupt_source(&mut afio);
                        #gpio_ident.trigger_on_edge(&#peripherals_ident.EXTI, Edge::#edge_ident);
                        #gpio_ident.enable_interrupt(&#peripherals_ident.EXTI);
                    ));
                }
            }
            idents.insert(gpio_ident);
        }
        idents
    }
    fn outputs(&mut self, inputs: Vec<Gpio>) -> HashSet<String> {
        let mut idents = HashSet::new();
        let gpio_pool = self
            .gpios
            .as_mut()
            .expect("Ports and gpios are not initialized");
        for gpio in inputs {
            let port_ident = gpio.port.lower();
            let gpio_ident = gpio_pool
                .take(&gpio_ident(gpio.pin, gpio.port))
                .expect("Use of uninitialized gpio");
            let pin_name = &gpio_ident; // Its only equal because we name the identifiers equally
            let control_reg = gpio.pin.control_reg();
            let init_function_name = gpio.mode.init_function_name();
            self.init_block.push(parse_quote!(
                let mut #gpio_ident = #port_ident.#pin_name.#init_function_name(&mut #port_ident.#control_reg);
            ));
            idents.insert(gpio_ident);
        }
        idents
    }
}
