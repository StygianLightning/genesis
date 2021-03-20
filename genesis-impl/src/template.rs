use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::input::*;

pub(crate) fn generate_code(input: &Input) -> TokenStream {
    let template_definition = generate_template_definition(input);

    let extra_attributes = input.attributes.iter().map(|attr| {
        let tokens = &attr.to_token_stream();
        quote! {
            #tokens
        }
    });

    quote! {
        #(#extra_attributes)*
        #template_definition
    }
}

fn generate_template_definition(input: &Input) -> TokenStream {
    let template_fields = input.components.iter().map(|c| {
        let ty = &c.field.ty;
        let name = &c.template_name;
        quote! {
            #name: ::std::option::Option<#ty>,
        }
    });

    let vis = &input.vis;
    let name = &input.template_name;

    quote! {
        #[derive(Default)]
        #vis struct #name {
            #(#template_fields)*
        }
    }
}
