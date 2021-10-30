use core::panic;

use yaml_rust::Yaml;

use super::DeviceConfig;

pub struct Stm32f1xxPeripherals {
    gpio: Gpios,
    timer: Vec<()>,
    pwm: Vec<()>,
    serial: Vec<()>,
}

impl Stm32f1xxPeripherals {
    pub fn from_yaml(yaml: &Yaml) -> Self {
        Self {
            gpio: Gpios {
                input: yaml["gpio"]["input"]
                    .clone()
                    .into_iter()
                    .map(|yaml| Gpio::input_from_yaml(&yaml))
                    .collect(),
                output: yaml["gpio"]["output"]
                    .clone()
                    .into_iter()
                    .map(|yaml| Gpio::output_from_yaml(&yaml))
                    .collect(),
            },
            timer: vec![],
            pwm: vec![],
            serial: vec![],
        }
    }
}

struct Gpios {
    input: Vec<Gpio>,
    output: Vec<Gpio>,
}
#[derive(Clone, Copy, Debug)]
pub struct Gpio {
    pin: usize,
    port: Port,
    mode: PinMode,
    interrupt_mode: InterruptMode,
}

impl Gpio {
    pub fn input_from_yaml(gpio_yaml: &Yaml) -> Self {
        let config = gpio_yaml.as_hash().expect("Unexpected input gpio format");
        let mut pin_name = None;
        for entry in config {
            match entry {
                (Yaml::String(k), Yaml::Null) => match pin_name {
                    Some(_) => unreachable!(),
                    None => pin_name = Some(k),
                },
                _ => {}
            }
        }
        let (pin, port) = Self::parse_pin(&pin_name);
        let mode = match gpio_yaml["mode"]
            .as_str()
            .expect("Missing key 'mode' in input pin")
        {
            "pull_up" => PinMode::InputPullUp,
            "pull_down" => PinMode::InputPullDown,
            "floating" => PinMode::InputFloating,
            other => panic!("Unable to parse mode {:?}", other),
        };
        let interrupt_mode = match gpio_yaml["interrupt"].as_str() {
            Some("rising") => InterruptMode::Rising,
            Some("falling") => InterruptMode::Falling,
            Some("rising_falling") => InterruptMode::RisingFalling,
            Some("none") => InterruptMode::None,
            None => InterruptMode::None,
            Some(other) => panic!("Unable to parse interrupt mode '{:?}'", other),
        };
        Gpio {
            pin,
            port,
            mode,
            interrupt_mode,
        }
    }
    pub fn output_from_yaml(gpio_yaml: &Yaml) -> Self {
        let config = match gpio_yaml {
            Yaml::Hash(hash) => hash,
            _ => panic!("Unexpected input gpio format"),
        };
        let mut pin_name = None;
        let mut pin_mode = None;
        for entry in config {
            match entry {
                (Yaml::String(k), Yaml::String(v)) => match pin_name {
                    Some(_) => panic!(
                        "Expected a single mode element for output gpio key (e.g. pb5: push_pull"
                    ),
                    None => {
                        pin_name = Some(k);
                        pin_mode = Some(v)
                    }
                },
                (k, v) => {
                    panic!("unknown input gpio config {:?}: {:?}", k, v)
                }
            }
        }
        let (pin, port) = Self::parse_pin(&pin_name);
        let mode = match pin_mode
            .expect("Unable to parse output pin mode")
            .to_lowercase()
            .as_str()
        {
            "push_pull" => PinMode::OutputPushPull,
            "open_drain" => PinMode::OutputOpenDrain,
            _ => panic!("Unable to parse output pin mode"),
        };
        Gpio {
            pin,
            port,
            mode,
            interrupt_mode: InterruptMode::None,
        }
    }
    fn parse_pin(key: &Option<&String>) -> (usize, Port) {
        let string = key.expect("could not parse pin name").to_lowercase();
        let string = match string.strip_prefix("p") {
            Some(s) => s,
            None => &string,
        };
        let (port, pin) = string.split_at(1);
        (
            pin.parse::<usize>().expect("Unable to parse pin number"),
            match port {
                "a" => Port::A,
                "b" => Port::B,
                "c" => Port::C,
                "d" => Port::D,
                "e" => Port::E,
                _ => panic!("unable to parse port"),
            },
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Port {
    A,
    B,
    C,
    D,
    E,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinMode {
    InputFloating,
    InputPullUp,
    InputPullDown,
    OutputPushPull,
    OutputOpenDrain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterruptMode {
    None,
    Rising,
    Falling,
    RisingFalling,
}

fn config_to_impl(config: &DeviceConfig) -> Vec<syn::Stmt> {
    todo!()
}
