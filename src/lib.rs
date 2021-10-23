use std::fs;

// mod config;
mod device;
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
    let parsed_yaml = match serde_yaml::from_str(&contents) {
        Ok(yaml) => yaml,
        Err(e) => {
            if let Some(location) = e.location() {
                let line = contents.lines().nth(location.line() - 1).unwrap();
                let mut message = String::new();
                // path/device.yaml:line:column
                message.push_str(&format!(
                    "{:?}:{}:{}\n",
                    path,
                    location.line(),
                    location.column()
                ));
                // line_nr | yaml
                message.push_str(&format!("{} | {}\n", location.line(), line));
                for _ in 0..location.column() + location.line().to_string().len() + 2 {
                    message.push(' ');
                }
                message.push('^');
                panic!("Unable to parse yaml:\n{}\n{}", e, message);
            } else {
                panic!("Unable to parse yaml: {}", e);
            }
        }
    };
    panic!("{:?}", parsed_yaml);
    parsed_yaml
    // let mut att_folded = Vec::new();
    // // extract all spans from the attribute token stream
    // attributes
    //     .clone()
    //     .into_iter()
    //     .for_each(|elem| att_folded.push(elem.span()));
    // // join all spans
    // let att_span = att_folded
    //     .iter()
    //     .fold(att_folded[0], |acc, elem| acc.join(*elem).unwrap());
    // let device_definition = att_span.source_text().expect("msg");
    // serde_yaml::from_str(&device_definition).expect("ParsingError")
}
