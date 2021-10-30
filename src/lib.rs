use std::fs;

// mod config;
mod device;
use device::DeviceConfig;
// mod generation;
// mod types;
// use config::Config;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};
// use types::*;

#[proc_macro_attribute]
pub fn device_config(attr: TokenStream, item: TokenStream) -> TokenStream {
    let config = parse_yaml(&attr);
    // let item_struct = parse_macro_input!(item as ItemStruct);
    // let struct_name = item_struct.ident.clone();
    // let statics_initialization = generation::component_statics(&config);

    // let init_statements = config.init_statements();
    // let return_statement = generation::init_return_statement(&config);
    // let return_type = generation::init_return_type(&config);
    // let interrupt_unmasks = config.interrupt_unmasks();
    // quote!(
    //     #item_struct
    //     impl #struct_name{
    //         fn init() -> #return_type{
    //             use core::mem::MaybeUninit;
    //             #(#init_statements)*
    //             #(#statics_initialization)*
    //             #return_statement
    //         }
    //         #[inline]
    //         fn enable_interrupts() {
    //             unsafe{
    //                 #(#interrupt_unmasks)*
    //             }
    //         }
    //     }
    // )
    // .into()
    quote!().into()
}

pub(crate) fn parse_yaml(attributes: &TokenStream) {
    let mut path = project_root::get_project_root().expect("Unable to find project root");
    path.push("device.yaml");
    let contents =
        fs::read_to_string(path.clone()).expect("Unable to read device.yaml in project root");
    let parsed_yaml = match yaml_rust::YamlLoader::load_from_str(&contents) {
        Ok(yaml) => yaml,
        Err(e) => {
            let location = e.marker();
            let line = contents.lines().nth(location.line() - 1).unwrap();
            let mut message = String::new();
            // path/device.yaml:line:column
            message.push_str(&format!("--> {:?}:\n", path,));
            // line_nr | yaml
            message.push_str(&format!("{} | {}\n", location.line(), line));
            for _ in 0..location.col() + location.line().to_string().len() + 3 {
                message.push(' ');
            }
            message.push('^');
            panic!("Unable to parse yaml:\n{}\n{}", e, message);
        }
    };
    let device_config = DeviceConfig::from_yaml(&parsed_yaml[0]);
}
