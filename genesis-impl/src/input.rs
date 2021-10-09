use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::Attribute;
use syn::Token;
use syn::{
    AngleBracketedGenericArguments, Data, DataStruct, DeriveInput, Field, GenericArgument, Ident,
    Path, PathArguments, Result, Type, TypePath, Visibility,
};

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

#[derive(Debug)]
pub(crate) struct WorldComponent {
    pub template_name: Ident,
    pub storage_type: ComponentStorageType,
    pub component_type: Type,
    pub field_name: Ident,
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
                    .map(|f| world_component(f))
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

fn world_component(f: &Field) -> WorldComponent {
    let mut template_name = f.ident.as_ref().unwrap().clone();
    for attr in f.attrs.iter() {
        let path_ident = attr.path.get_ident();
        if path_ident.is_some() && path_ident.unwrap() == "template_name" {
            let tokens = attr.tokens.clone();
            if let Ok(name) = syn::parse2::<TemplateName>(tokens) {
                template_name = name.ident;
            }
        }
    }
    let (component_type, storage_type) = get_inner_type(f, "VecStorage")
        .map(|t| (t.clone(), ComponentStorageType::Vec))
        .or_else(|| get_inner_type(f, "MapStorage").map(|t| (t.clone(), ComponentStorageType::Map)))
        .expect("World components must be wrapped in VecStorage or MapStorage");

    WorldComponent {
        field_name: f.ident.clone().unwrap(),
        storage_type,
        template_name,
        component_type,
    }
}

fn get_inner_type<'a, 'b>(field: &'a Field, name: &'b str) -> Option<&'a Type> {
    match &field.ty {
        Type::Path(TypePath {
            qself: None,
            path: Path { segments, .. },
        }) if segments.first().is_some() => {
            let first_segment = segments.first().unwrap();
            if first_segment.ident == name {
                if let PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                    args, ..
                }) = &first_segment.arguments
                {
                    if let Some(GenericArgument::Type(inner_type)) = args.first() {
                        return Some(inner_type);
                    }
                }
            }

            None
        }
        _ => None,
    }
}
