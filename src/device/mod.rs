use core::panic;

use yaml_rust::Yaml;

use stm32f1xx::Stm32f1xxPeripherals;

mod stm32f1xx;

#[derive(Debug)]
pub(crate) struct DeviceConfig {
    kind: DeviceKind,
    clock: Hertz,
}

#[non_exhaustive]
#[derive(Debug)]
enum DeviceKind {
    Stm32f1xx(Stm32f1xxPeripherals),
}

#[derive(Debug, Copy, Clone)]
pub struct Baud(pub u32);
impl Baud {
    pub fn from_str(str: &str) -> Self {
        Self(str.parse::<u32>().expect("Unable to parse baud rate"))
    }
    pub fn from_i64(int: i64) -> Self {
        Self(int as u32)
    }
}
#[derive(Debug)]
struct Hertz(usize);

impl Hertz {
    pub fn from_str(str: &str) -> Self {
        let mut last_digit = 0;
        for char in str.chars() {
            if char.is_ascii_digit() {
                last_digit += 1;
            } else {
                break;
            }
        }
        let (amount, unit) = str.split_at(last_digit);
        let factor = match unit.to_lowercase().as_str() {
            "hz" => 1,
            "khz" => 1_000,
            "mhz" => 1_000_000,
            "ghz" => 1_000_000_000,
            _ => panic!("unknown frequency unit (unit is 'hz', 'khz', 'mhz' or 'ghz)"),
        };
        Self(amount.parse::<usize>().expect("Unable to parse frequency") * factor)
    }
}

impl DeviceConfig {
    pub(crate) fn from_yaml(yaml: &Yaml) -> Self {
        let kind = match yaml["kind"]
            .as_str()
            .expect("cannot parse device kind")
            .to_lowercase()
            .as_str()
        {
            "stm32f1xx" | "bluepill" => {
                DeviceKind::Stm32f1xx(Stm32f1xxPeripherals::from_yaml(&yaml))
            }
            other => panic!("Unknown device kind \"{}\"", other),
        };
        let clock = yaml["clock"].as_str().map(|c| Hertz::from_str(c));
        panic!(
            "{:?}",
            Self {
                kind,
                clock: clock.expect("Unable to parse clock"),
            }
        )
    }
}
