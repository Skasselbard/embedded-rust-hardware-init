use core::panic;
use syn::parse_quote;
use yaml_rust::Yaml;

mod generation;

use self::generation::DeviceInit;

use super::{Baud, DeviceConfig, Hertz};

#[derive(Debug)]
pub struct Stm32f1xxPeripherals {
    gpio: Gpios,
    timer: Vec<Timer>,
    pwm: Vec<PWM>,
    serial: Vec<Serial>,
}

impl Stm32f1xxPeripherals {
    pub fn from_yaml(yaml: &Yaml) -> Self {
        let peripherals = Self {
            gpio: Gpios {
                input: yaml["gpio"]["input"]
                    .as_vec()
                    .map(|ins| {
                        ins.iter()
                            .map(|yaml| Gpio::input_from_yaml(&yaml))
                            .collect()
                    })
                    .unwrap_or_default(),
                output: yaml["gpio"]["output"]
                    .as_vec()
                    .map(|outs| {
                        outs.iter()
                            .map(|yaml| Gpio::output_from_yaml(&yaml))
                            .collect()
                    })
                    .unwrap_or_default(),
            },
            timer: yaml["timer"]
                .as_vec()
                .map(|timers| timers.iter().map(|timer| Timer::from_yaml(timer)).collect())
                .unwrap_or_default(),
            pwm: yaml["pwm"]
                .as_vec()
                .map(|pwms| pwms.iter().map(|pwm| PWM::from_yaml(pwm)).collect())
                .unwrap_or_default(),
            serial: yaml["serial"]
                .as_vec()
                .map(|serials| {
                    serials
                        .iter()
                        .map(|serial| Serial::from_yaml(serial))
                        .collect()
                })
                .unwrap_or_default(),
        };
        peripherals.check();
        peripherals
    }
    fn check(&self) {
        // TODO: check that timers are captured only once
        // TODO: check gpio and peripheral combination is possible
    }
    fn used_gpios(&self) -> Vec<(Pin, Port)> {
        let mut inputs: Vec<(Pin, Port)> = self
            .gpio
            .input
            .iter()
            .map(|gpio| (gpio.pin, gpio.port))
            .collect::<Vec<(Pin, Port)>>();
        let outputs = self
            .gpio
            .output
            .iter()
            .map(|gpio| (gpio.pin, gpio.port))
            .collect::<Vec<(Pin, Port)>>();
        let pwm = self
            .pwm
            .iter()
            .map(|pwm| pwm.pins.clone())
            .flatten()
            .collect::<Vec<(Pin, Port)>>();
        let serial = self
            .serial
            .iter()
            .map(|serial| vec![serial.tx, serial.rx])
            .flatten()
            .collect::<Vec<(Pin, Port)>>();
        inputs.extend_from_slice(&outputs);
        inputs.extend_from_slice(&pwm);
        inputs.extend_from_slice(&serial);
        inputs
    }
}

#[derive(Debug)]
pub struct Serial {
    id: SerialID,
    rx: (Pin, Port),
    tx: (Pin, Port),
    baud_rate: Baud,
}
#[derive(Debug)]
enum SerialID {
    Usart1,
    Usart2,
    Usart3,
}

impl Serial {
    fn from_yaml(yaml: &Yaml) -> Self {
        let config = yaml.as_hash().expect("Unexpected input serial format");
        let mut serial_name = None;
        for entry in config {
            match entry {
                (Yaml::String(k), Yaml::Null) => match serial_name {
                    Some(_) => unreachable!(),
                    None => {
                        serial_name = Some(k);
                        break;
                    }
                },
                _ => {}
            }
        }
        Self {
            id: SerialID::from_str(&serial_name.expect("Unknown serial ID")),
            rx: Gpio::parse_pin(&Some(
                yaml["rx"].as_str().expect("Missing 'rx' gpio in serial"),
            )),
            tx: Gpio::parse_pin(&Some(
                yaml["tx"].as_str().expect("Missing 'tx' gpio in serial"),
            )),
            baud_rate: Baud::from_i64(
                yaml["baud"]
                    .as_i64()
                    .expect("Missing 'baud' rate in serial"),
            ),
        }
    }
}

impl SerialID {
    fn from_str(str: &str) -> Self {
        match str.to_lowercase().as_str() {
            "usart1" => Self::Usart1,
            "usart2" => Self::Usart2,
            "usart3" => Self::Usart3,
            other => panic!("Unknown serial name '{:?}'", other),
        }
    }
}

#[derive(Debug)]
pub struct PWM {
    timer: Timer,
    pins: Vec<(Pin, Port)>,
    frequency: Option<Hertz>,
}

impl PWM {
    fn from_yaml(yaml: &Yaml) -> Self {
        let config = yaml.as_hash().expect("Unexpected input pwm format");
        let mut timer_name = None;
        for entry in config {
            match entry {
                (k, Yaml::Null) => match timer_name {
                    Some(_) => unreachable!(),
                    None => {
                        timer_name = Some(k);
                        break;
                    }
                },
                _ => {}
            }
        }
        Self {
            timer: Timer::from_yaml(timer_name.expect("no timer found for pwm")),
            pins: yaml["pins"]
                .clone()
                .into_iter()
                .map(|pin| Gpio::parse_pin(&pin.as_str()))
                .collect(),
            frequency: yaml["freq"].as_str().map(|f| Hertz::from_str(f)),
        }
    }
}

#[derive(Debug)]
struct Timer {
    id: TimerID,
}
#[derive(Debug)]
enum TimerID {
    Tim1,
    Tim2,
    Tim3,
}

impl Timer {
    fn from_yaml(yaml: &Yaml) -> Self {
        let id = match yaml
            .as_str()
            .expect("Unable to parse timer")
            .to_lowercase()
            .as_str()
        {
            "tim1" => TimerID::Tim1,
            "tim2" => TimerID::Tim2,
            "tim3" => TimerID::Tim3,
            other => panic!("Unknown timer '{}'", other),
        };
        Self { id }
    }
}

#[derive(Debug)]
struct Gpios {
    input: Vec<Gpio>,
    output: Vec<Gpio>,
}
#[derive(Clone, Copy, Debug)]
pub struct Gpio {
    pin: Pin,
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
                    None => pin_name = Some(k.as_str()),
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
                        pin_name = Some(k.as_str());
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
    fn parse_pin(key: &Option<&str>) -> (Pin, Port) {
        let string = key.expect("could not parse pin name").to_lowercase();
        let string = match string.strip_prefix("p") {
            Some(s) => s,
            None => &string,
        };
        let (port, pin) = string.split_at(1);
        (
            Pin(pin.parse::<usize>().expect("Unable to parse pin number")),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pin(usize);

impl Pin {
    fn control_reg(&self) -> &str {
        if (self.0 % 16) < 8 {
            "crl"
        } else {
            "crh"
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Port {
    A,
    B,
    C,
    D,
    E,
}

impl Port {
    fn lower(&self) -> &str {
        match self {
            Port::A => "gpioa",
            Port::B => "gpiob",
            Port::C => "gpioc",
            Port::D => "gpiod",
            Port::E => "gpioe",
        }
    }
    fn upper(&self) -> &str {
        match self {
            Port::A => "GPIOA",
            Port::B => "GPIOB",
            Port::C => "GPIOC",
            Port::D => "GPIOD",
            Port::E => "GPIOE",
        }
    }
    fn short(&self) -> char {
        match self {
            Port::A => 'a',
            Port::B => 'b',
            Port::C => 'c',
            Port::D => 'd',
            Port::E => 'e',
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PinMode {
    InputFloating,
    InputPullUp,
    InputPullDown,
    OutputPushPull,
    OutputOpenDrain,
}

impl PinMode {
    fn init_function_name(&self) -> &str {
        match self {
            PinMode::InputFloating => "into_floating_input",
            PinMode::InputPullUp => "into_pull_up_input",
            PinMode::InputPullDown => "into_pull_down_input",
            PinMode::OutputPushPull => "into_open_drain_output",
            PinMode::OutputOpenDrain => "into_push_pull_output",
        }
    }
    fn direction_name(&self) -> &str {
        match self {
            PinMode::InputFloating => "Input",
            PinMode::InputPullUp => "Input",
            PinMode::InputPullDown => "Output",
            PinMode::OutputPushPull => "Output",
            PinMode::OutputOpenDrain => "Output",
        }
    }
    fn mode_name(&self) -> &str {
        match self {
            PinMode::InputFloating => "Floating",
            PinMode::InputPullUp => "PullUp",
            PinMode::InputPullDown => "PullDown",
            PinMode::OutputPushPull => "OpenDrain",
            PinMode::OutputOpenDrain => "PushPull",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterruptMode {
    None,
    Rising,
    Falling,
    RisingFalling,
}

impl InterruptMode {
    fn ident(&self) -> &str {
        match self {
            InterruptMode::None => {
                panic!("InterruptMode::None cannot be converted into an identifier")
            }
            InterruptMode::Rising => "RISING",
            InterruptMode::Falling => "FALLING",
            InterruptMode::RisingFalling => "RISING_FALLING",
        }
    }
}

pub(super) fn init_stmts_and_return_tys(
    config: &DeviceConfig,
) -> (Vec<syn::Stmt>, syn::Type) {
    DeviceInit::get_init_block(config)
}
