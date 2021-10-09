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

/// Generates an ECS (World) and a component enum containing all components.
///
/// Takes as input a struct with named fields.
/// The names of the fields will correspond to the names of the storage types in the generated World.
/// The storage type used can be specified with `#[component(vec)]` for `VecStorage<T>` (the default)
/// or `#[component(map)]` for `MapStorage<T>`.
///
/// The name of the generated ECS is passed to the `#[world]` macro directly, together with the name of
/// the component enum. The component enum is a generated enum with one variant per component type that
/// can be used to register any of the component types on the generated World as an alternative to
/// directly calling `.set()` on the corresponding storage field.
///
/// The generated ECS has a shared set of `Entities` that is also used by each storage to check if
/// an entity exists; it is available via the `.entities` field. To avoid concurrency hazards,
/// it is stored in an `Arc<RwLock<Entities>>`. The generated `World` has some utility methods for
/// spawning new entities; these are handy shortcuts to accessing the underlying `entities` directly.
/// When spawning entities in a batch, direct access is recommended to avoid re-acquiring the write
/// lock over and over.
///
/// In addition to the component enum, this macro generates a "template" for an entity;
/// this template has one public field of type `Option<T>` for every component and can be used
/// to set the corresponding components on an entity. The name of these fields defaults to the name of the
/// field in the World definition and can be customized via `#[template_name(name)]`.
///
/// Attribute macros like `#[derive(Debug)]` are applied to both the component enum and the
/// template struct. This can be very useful for debugging and provides a quick and simple way
/// to define entities in data files and using e.g. serde to deserialize them into the generated
/// Template struct.
///
/// # Example
/// ```ignore
/// #[derive(Clone, Debug, Eq, PartialEq)]
/// pub struct Position {
///    pub position: (u32, u32),
/// }
///
/// #[derive(Clone, Debug, Eq, PartialEq)]
/// pub struct NameComponent {
///     pub name: String,
/// }
///
/// #[derive(Clone, Debug, Eq, PartialEq)]
/// pub struct RareComponent {
///     pub data: u32,
/// }
///
/// #[world(MyComponent, Template)]
/// #[derive(Clone, Debug, Eq, PartialEq)]
/// pub struct World {
///     positions: VecStorage<Position>,
///     names: VecStorage<NameComponent>,
///     rare_data: MapStorage<RareComponent>,
/// }
/// ```
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
