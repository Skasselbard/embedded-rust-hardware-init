#![feature(proc_macro_span)]
mod config;

mod devices;
mod generation;
mod types;
use config::Config;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};
use types::*;

#[proc_macro_attribute]
pub fn device_config(attr: TokenStream, item: TokenStream) -> TokenStream {
    let config = parse_yaml(&attr);
    let item_struct = parse_macro_input!(item as ItemStruct);
    let struct_name = item_struct.ident.clone();
    let statics = generation::component_statics(&config);

    let init_statements = config.init_statements();
    let return_statement = generation::init_retutn_statement(&config);
    let return_type = generation::init_retutn_type(&config);
    let interrupt_unmasks = config.interrupt_unmasks();
    quote!(
        #item_struct
        impl #struct_name{
            fn init() -> #return_type{
                use core::mem::MaybeUninit;
                #(#init_statements)*
                #(#statics)*
                #return_statement
            }
            #[inline]
            fn enable_interrupts() {
                unsafe{
                    #(#interrupt_unmasks)*
                }
            }
        }
    )
    .into()
}

pub(crate) fn parse_yaml(attributes: &TokenStream) -> Config {
    let mut att_folded = Vec::new();
    // extract all spans from the attribute token stream
    attributes
        .clone()
        .into_iter()
        .for_each(|elem| att_folded.push(elem.span()));
    // join all spans
    let att_span = att_folded
        .iter()
        .fold(att_folded[0], |acc, elem| acc.join(*elem).unwrap());
    let device_definition = att_span.source_text().expect("msg");
    serde_yaml::from_str(&device_definition).expect("ParsingError")
}
