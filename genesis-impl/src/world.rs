use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;

use syn::Ident;

use crate::input::*;

pub(crate) fn generate_code(input: &Input) -> TokenStream {
    let world = &input.world_name;

    let struct_definition = generate_struct_definition(input);
    let new_fn = generate_new(input);
    let spawn_fn = generate_spawn_fn(input);
    let despawn_fn = generate_despawn_fn(input);
    let clear_fn = generate_clear_fn(input);

    let register_impls = generate_register_impls(input);

    quote! {

        #struct_definition

        impl #world {
            #new_fn

            #spawn_fn

            #despawn_fn

            #clear_fn
        }

        #register_impls
    }
}

fn generate_struct_definition(input: &Input) -> TokenStream {
    let world_fields = input.components.iter().map(|c| {
        let name = &c.field.ident;
        let ty = &c.field.ty;
        let storage_type = Ident::new(c.storage_type.name(), Span::call_site());
        quote! {
            #name: ::genesis::#storage_type<#ty>,
        }
    });

    let world = &input.world_name;
    let vis = &input.vis;

    quote! {
        #vis struct #world {
            #vis entities: ::std::sync::Arc<::std::sync::RwLock<::genesis::Entities>>,
            #(#vis #world_fields)*
        }
    }
}

fn generate_new(input: &Input) -> TokenStream {
    let entities_arg = Ident::new("entities", Span::call_site());
    let capacity_arg = Ident::new("initial_capacity", Span::call_site());

    let storage_locals = input.components.iter().map(|c| {
        let name = &c.field.ident;
        let storage_type_name = Ident::new(c.storage_type.name(), Span::call_site());
        match c.storage_type {
            ComponentStorageType::Vec => quote! {
                let #name = ::genesis::#storage_type_name::new(::std::sync::Arc::clone(&#entities_arg), #capacity_arg);
            },
            ComponentStorageType::Map => quote! {
                let #name = ::genesis::#storage_type_name::new(::std::sync::Arc::clone(&#entities_arg));
            },
        }
    });

    let storage_names = input.components.iter().map(|c| {
        let name = &c.field.ident;
        quote! { #name, }
    });

    let vis = &input.vis;
    quote! {
        #vis fn new(#capacity_arg: u32) -> Self {
            let entities = ::std::sync::Arc::new(::std::sync::RwLock::new(::genesis::Entities::new(#capacity_arg)));

            #(#storage_locals)*

            Self {
                entities,
                #(#storage_names)*
            }
        }
    }
}

fn generate_spawn_fn(input: &Input) -> TokenStream {
    let vis = &input.vis;
    quote! {
        #vis fn spawn(&mut self) -> ::genesis::Entity {
            self.entities.write().unwrap().spawn()
        }
    }
}

fn generate_despawn_fn(input: &Input) -> TokenStream {
    let vis = &input.vis;

    let remove_unchecked_calls = input.components.iter().map(|c| {
        let name = &c.field.ident;
        quote! {
            self.#name.remove_unchecked(entity);
        }
    });

    quote! {
        #vis fn despawn(&mut self, entity: ::genesis::Entity) -> ::std::result::Result<(), ::genesis::NoSuchEntity> {
            let mut write = self.entities.write().unwrap();
            write.despawn(entity)?;
            #(#remove_unchecked_calls)*
            Ok(())
        }
    }
}

fn generate_clear_fn(input: &Input) -> TokenStream {
    let vis = &input.vis;

    let clear_calls = input.components.iter().map(|c| {
        let name = c.field.ident.as_ref().unwrap();
        quote! {
            self.#name.clear();
        }
    });

    quote! {
        #vis fn clear(&mut self) {
            let mut write = self.entities.write().unwrap();
            write.clear();
            #(#clear_calls)*
        }
    }
}

fn generate_register_impls(input: &Input) -> TokenStream {
    let world = &input.world_name;
    let register_impls = input.components.iter().map(|c| {
        let ty = &c.field.ty;
        let component_storage_name = c.field.ident.as_ref().unwrap();
        quote! {
            impl ::genesis::Register<#ty> for #world {
                fn register(&mut self, entity: ::genesis::Entity, component: #ty)
                    -> ::std::result::Result<std::option::Option<#ty>, ::genesis::NoSuchEntity> {
                    self.#component_storage_name.set(entity, component)
                }
            }
        }
    });
    let component_enum_register_impl = {
        let component_enum = &input.component_enum_name;
        let component_enum_match_impl_register = input.components.iter().map(|c| {
            let ty = &c.field.ty;

            quote! {
                #component_enum::#ty(c) => self.register(entity, c)?.map(|c| c.into()),
            }
        });

        quote! {
            impl ::genesis::Register<#component_enum> for #world {
                fn register(&mut self, entity: ::genesis::Entity, component: #component_enum)
                -> ::std::result::Result<::std::option::Option::<#component_enum>, ::genesis::NoSuchEntity> {
                Ok(match component {
                #(#component_enum_match_impl_register)*
                })
                }
            }
        }
    };

    quote! {
        #(#register_impls)*

        #component_enum_register_impl
    }
}
