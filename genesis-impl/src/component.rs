use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::input::*;

pub(crate) fn generate_code(input: &Input) -> TokenStream {
    let enum_definition = generate_enum_definition(input);
    let from_impls = generate_from_impls(input);

    let extra_attributes = input.attributes.iter().map(|attr| {
        let tokens = &attr.to_token_stream();
        quote! {
            #tokens
        }
    });

    quote! {
        #(#extra_attributes)*
        #enum_definition
        #from_impls
    }
}

fn generate_enum_definition(input: &Input) -> TokenStream {
    let component_fields = input.components.iter().map(|c| {
        let ty = &c.field.ty;
        quote! {
            #ty(#ty),
        }
    });

    let vis = &input.vis;
    let name = &input.component_enum_name;

    quote! {
        #vis enum #name {
            #(#component_fields)*
        }
    }
}

fn generate_from_impls(input: &Input) -> TokenStream {
    let component_enum = &input.component_enum_name;
    let from_impls = input.components.iter().map(|c| {
        let ty = &c.field.ty;
        quote! {
            impl From<#ty> for #component_enum {
                fn from(component: #ty) -> Self {
                    Self::#ty(component)
                }
            }
        }
    });

    quote! {
        #(#from_impls)*
    }
}
