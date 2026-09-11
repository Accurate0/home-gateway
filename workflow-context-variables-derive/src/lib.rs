use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Field, Fields, LitStr, parse_macro_input};

#[proc_macro_derive(WorkflowContextVariables, attributes(variables))]
pub fn derive_workflow_context_variables(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match expand(&input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

struct FieldAttributes {
    rename: Option<String>,
    skip: bool,
}

impl FieldAttributes {
    fn parse(field: &Field) -> syn::Result<Self> {
        let mut attributes = FieldAttributes {
            rename: None,
            skip: false,
        };

        for attribute in field
            .attrs
            .iter()
            .filter(|a| a.path().is_ident("variables"))
        {
            attribute.parse_nested_meta(|meta| {
                if meta.path.is_ident("skip") {
                    attributes.skip = true;
                    return Ok(());
                }

                if meta.path.is_ident("rename") {
                    let value: LitStr = meta.value()?.parse()?;
                    attributes.rename = Some(value.value());
                    return Ok(());
                }

                Err(meta.error("expected `rename = \"...\"` or `skip`"))
            })?;
        }

        Ok(attributes)
    }
}

fn expand(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let Data::Struct(data) = &input.data else {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "WorkflowContextVariables can only be derived for structs",
        ));
    };

    let Fields::Named(fields) = &data.fields else {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "WorkflowContextVariables requires named fields",
        ));
    };

    let mut shapes = Vec::new();
    let mut nodes = Vec::new();

    for field in &fields.named {
        let attributes = FieldAttributes::parse(field)?;

        if attributes.skip {
            continue;
        }

        let Some(ident) = field.ident.as_ref() else {
            return Err(syn::Error::new_spanned(field, "expected a named field"));
        };

        let ty = &field.ty;
        let name = attributes.rename.unwrap_or_else(|| ident.to_string());

        shapes.push(quote! {
            fields.insert(
                #name.to_owned(),
                <#ty as crate::variables::VariableField>::field_shape(),
            );
        });

        nodes.push(quote! {
            fields.insert(
                #name.to_owned(),
                crate::variables::VariableField::field_node(&self.#ident),
            );
        });
    }

    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics crate::variables::WorkflowContextVariables for #ident #ty_generics #where_clause {
            fn shape() -> crate::variables::Shape {
                let mut fields = ::std::collections::BTreeMap::new();
                #(#shapes)*
                crate::variables::Shape::Object(fields)
            }

            fn to_node(&self) -> crate::variables::Node {
                let mut fields = ::std::collections::BTreeMap::new();
                #(#nodes)*
                crate::variables::Node::Object(fields)
            }
        }

        impl #impl_generics crate::variables::VariableField for #ident #ty_generics #where_clause {
            fn field_shape() -> crate::variables::Shape {
                <Self as crate::variables::WorkflowContextVariables>::shape()
            }

            fn field_node(&self) -> crate::variables::Node {
                crate::variables::WorkflowContextVariables::to_node(self)
            }
        }
    })
}
