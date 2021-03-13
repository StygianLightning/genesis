#![deny(rust_2018_idioms)]
#![deny(clippy::all)]

mod component;
mod input;
mod template;
mod world;

use input::*;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Result};

#[proc_macro_attribute]
pub fn world(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let args = parse_macro_input!(args as InputArgs);
    generate_code(args, input).unwrap_or_else(|e| e.to_compile_error().into())
}

fn generate_code(args: InputArgs, input: DeriveInput) -> Result<TokenStream> {
    let input = Input::new(args, &input)?;
    let template_code = template::generate_code(&input);
    let component_code = component::generate_code(&input);
    let world_code = world::generate_code(&input);

    let output = quote! {
        #template_code
        #component_code
        #world_code
    };

    Ok(TokenStream::from(output))
}
