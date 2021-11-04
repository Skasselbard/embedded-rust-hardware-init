use std::{fs, path::PathBuf};

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
    let mut path = project_root::get_project_root().expect("Unable to find project root");
    path.push("device.yaml");

    let config = parse_yaml(&path);
    let item_struct = parse_macro_input!(item as ItemStruct);
    let struct_name = item_struct.ident.clone();

    let (init_statements, return_type) = config.get_init_fn();
    quote!(
        #item_struct
        impl #struct_name{
            fn init() -> #return_type{
                use core::mem::MaybeUninit;
                #(#init_statements)*
                //#return_statement
            }
            // #[inline]
            // fn enable_interrupts() {
            //     unsafe{
            //         #(#interrupt_unmasks)*
            //     }
            // }
        }
    )
    .into()
}

pub(crate) fn parse_yaml(path: &PathBuf) -> DeviceConfig {
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
    DeviceConfig::from_yaml(&parsed_yaml[0])
}

#[test]
fn generate_test() {
    let mut path = project_root::get_project_root().expect("Unable to find project root");
    path.push("notes/yamlLayouts.yaml");
    let config = parse_yaml(&path);
    let (init_statements, return_type) = config.get_init_fn("test_struct");
}
