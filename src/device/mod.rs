mod stm32f1xx;

use std::fmt;

use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_aux::prelude::*;

#[derive(Debug)]
struct DeviceConfig {
    kind: DeviceKind,
    clock: Hertz,
    gpio: Gpios,
    timer: Vec<Timer>,
    pwm: Vec<Pwm>,
    serial: Vec<Serial>,
}

#[derive(serde::Deserialize, Debug)]
#[non_exhaustive]
enum DeviceKind {
    stm32f1xx,
}

#[derive(serde::Deserialize, Debug)]
struct Gpios {
    input: (),
    output: (),
}

#[derive(serde::Deserialize, Debug)]
struct Hertz {}

impl DeviceConfig {
    fn new(
        kind: DeviceKind,
        clock: Hertz,
        gpio: Gpios,
        timer: Vec<Timer>,
        pwm: Vec<Pwm>,
        serial: Vec<Serial>,
    ) -> Self {
        Self {
            kind,
            clock,
            gpio,
            timer,
            pwm,
            serial,
        }
    }
}

impl<'de> Deserialize<'de> for DeviceConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            Kind,
            Clock,
            Gpio,
            Timer,
            Pwm,
            Serial,
        }
        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;
                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("'kind', 'clock', 'gpio', 'timer', 'pwm' or 'serial'")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value.to_lowercase().as_str() {
                            "kind" => Ok(Field::Kind),
                            "clock" => Ok(Field::Clock),
                            "gpio" => Ok(Field::Gpio),
                            "timer" => Ok(Field::Timer),
                            "pwm" => Ok(Field::Pwm),
                            "serial" => Ok(Field::Serial),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct DeviceConfigVisitor;

        impl<'de> Visitor<'de> for DeviceConfigVisitor {
            type Value = DeviceConfig;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Duration")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<DeviceConfig, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let kind = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let clock = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let gpio = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                let timer = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(3, &self))?;
                let pwm = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(4, &self))?;
                let serial = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(5, &self))?;
                Ok(DeviceConfig::new(kind, clock, gpio, timer, pwm, serial))
            }

            fn visit_map<V>(self, mut map: V) -> Result<DeviceConfig, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut kind = None;
                let mut clock = None;
                let mut gpio = None;
                let mut timer = None;
                let mut pwm = None;
                let mut serial = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Kind => {
                            if kind.is_some() {
                                return Err(de::Error::duplicate_field("kind"));
                            }
                            kind = Some(map.next_value()?);
                        }
                        Field::Clock => {
                            if clock.is_some() {
                                return Err(de::Error::duplicate_field("clock"));
                            }
                            clock = Some(map.next_value()?);
                        }
                        Field::Gpio => {
                            if gpio.is_some() {
                                return Err(de::Error::duplicate_field("gpio"));
                            }
                            gpio = Some(map.next_value()?);
                        }
                        Field::Timer => {
                            if timer.is_some() {
                                return Err(de::Error::duplicate_field("timer"));
                            }
                            timer = Some(map.next_value()?);
                        }
                        Field::Pwm => {
                            if pwm.is_some() {
                                return Err(de::Error::duplicate_field("pwm"));
                            }
                            pwm = Some(map.next_value()?);
                        }
                        Field::Serial => {
                            if serial.is_some() {
                                return Err(de::Error::duplicate_field("serial"));
                            }
                            serial = Some(map.next_value()?);
                        }
                    }
                }
                let kind = kind.ok_or_else(|| de::Error::missing_field("kind"))?;
                let clock = clock.ok_or_else(|| de::Error::missing_field("clock"))?;
                let gpio = gpio.ok_or_else(|| de::Error::missing_field("gpio"))?;
                let timer = timer.ok_or_else(|| de::Error::missing_field("timer"))?;
                let pwm = pwm.ok_or_else(|| de::Error::missing_field("pwm"))?;
                let serial = serial.ok_or_else(|| de::Error::missing_field("serial"))?;
                Ok(DeviceConfig::new(kind, clock, gpio, timer, pwm, serial))
            }
        }

        const FIELDS: &'static [&'static str] = &["secs", "nanos"];
        deserializer.deserialize_struct("DeviceConfig", FIELDS, DeviceConfigVisitor)
    }
}
