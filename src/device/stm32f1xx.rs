use core::panic;

use yaml_rust::Yaml;

use super::{DeviceConfig, Hertz};

#[derive(Debug)]
pub struct Stm32f1xxPeripherals {
    gpio: Gpios,
    timer: Vec<Timer>,
    pwm: Vec<PWM>,
    serial: Vec<()>,
}

impl Stm32f1xxPeripherals {
    pub fn from_yaml(yaml: &Yaml) -> Self {
        Self {
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
            serial: vec![],
        }
    }
}

#[derive(Debug)]
pub struct PWM {
    timer: Timer,
    pins: Vec<(usize, Port)>,
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
    fn parse_pin(key: &Option<&str>) -> (usize, Port) {
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
