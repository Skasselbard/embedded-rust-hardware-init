use core::panic;

use yaml_rust::Yaml;

use stm32f1xx::Stm32f1xxPeripherals;

mod stm32f1xx;

pub(crate) struct DeviceConfig {
    kind: DeviceKind,
    clock: Hertz,
}

#[non_exhaustive]
enum DeviceKind {
    Stm32f1xx(Stm32f1xxPeripherals),
}

struct Hertz {}

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
        let clock = Hertz {};
        Self { kind, clock }
    }
}
