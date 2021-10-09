use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::Attribute;
use syn::Token;
use syn::{Data, DataStruct, DeriveInput, Field, Ident, Result, Visibility};

pub(crate) struct Input {
    pub world_name: Ident,
    pub component_enum_name: Ident,
    pub template_name: Ident,
    pub components: Vec<WorldComponent>,
    pub vis: Visibility,
    pub attributes: Vec<Attribute>,
}

pub struct InputArgs {
    pub component_name: Ident,
    pub template_name: Ident,
}

impl Parse for InputArgs {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let component_name = input.parse::<Ident>()?;
        let _separator = input.parse::<Token![,]>()?;
        let template_name = input.parse::<Ident>()?;
        Ok(Self {
            component_name,
            template_name,
        })
    }
}

pub(crate) struct WorldComponent {
    pub field: Field,
    pub template_name: Ident,
    pub storage_type: ComponentStorageType,
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum ComponentStorageType {
    Vec,
    Map,
}

impl ComponentStorageType {
    pub(crate) fn name(self) -> &'static str {
        match self {
            ComponentStorageType::Vec => "VecStorage",
            ComponentStorageType::Map => "MapStorage",
        }
    }
}

impl Default for ComponentStorageType {
    fn default() -> Self {
        ComponentStorageType::Vec
    }
}

impl Parse for ComponentStorageType {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let inner;
        syn::parenthesized!(inner in input);

        let ident = inner.parse::<Ident>()?.to_string();

        if ident == "map" {
            Ok(Self::Map)
        } else {
            Ok(Self::Vec)
        }
    }
}

pub(crate) struct TemplateName {
    pub ident: Ident,
}

impl Parse for TemplateName {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let inner;
        syn::parenthesized!(inner in input);

        let ident = inner.parse::<Ident>()?;

        Ok(Self { ident })
    }
}

const EXPECTED_NAMED_STRUCT_FIELDS: &str = "Only structs with named fields are supported.";

impl Input {
    pub(crate) fn new(args: InputArgs, input: &DeriveInput) -> Result<Self> {
        match &input.data {
            Data::Struct(DataStruct {
                fields: syn::Fields::Named(fields_named),
                ..
            }) => {
                let fields = fields_named
                    .named
                    .iter()
                    .map(|f| {
                        let (ty, name) = get_field_storage_type_and_template_name(f);
                        WorldComponent {
                            field: f.clone(),
                            storage_type: ty,
                            template_name: name,
                        }
                    })
                    .collect();
                Ok(Self {
                    world_name: input.ident.clone(),
                    template_name: args.template_name,
                    component_enum_name: args.component_name,
                    components: fields,
                    vis: input.vis.clone(),
                    attributes: input.attrs.clone(),
                })
            }
            Data::Struct(data_struct) => {
                if let syn::Fields::Unnamed(fields_unnamed) = &data_struct.fields {
                    Err(syn::Error::new(
                        fields_unnamed.span(),
                        EXPECTED_NAMED_STRUCT_FIELDS,
                    ))
                } else {
                    Err(syn::Error::new(input.span(), EXPECTED_NAMED_STRUCT_FIELDS))
                }
            }
            _ => Err(syn::Error::new(input.span(), EXPECTED_NAMED_STRUCT_FIELDS)),
        }
    }
}

fn get_field_storage_type_and_template_name(f: &Field) -> (ComponentStorageType, Ident) {
    let mut component_type = ComponentStorageType::Vec;
    let mut template_name = f.ident.as_ref().unwrap().clone();
    for attr in f.attrs.iter() {
        let path_ident = attr.path.get_ident();
        if path_ident.is_some() && path_ident.unwrap() == "component" {
            let tokens = attr.tokens.clone();
            if let Ok(ty) = syn::parse2::<ComponentStorageType>(tokens) {
                component_type = ty;
            }
        } else if path_ident.is_some() && path_ident.unwrap() == "template_name" {
            let tokens = attr.tokens.clone();
            if let Ok(name) = syn::parse2::<TemplateName>(tokens) {
                template_name = name.ident;
            }
        }
    }
    (component_type, template_name)
}
