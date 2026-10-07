use quote::quote;
use syn::{Attribute, Data, DeriveInput, Fields, LitStr};

struct ClassAttributes {
    name: Option<String>,
    input: bool,
    output: bool,
}

impl ClassAttributes {
    fn parse(attributes: &[Attribute]) -> syn::Result<Self> {
        let mut parsed = ClassAttributes {
            name: None,
            input: false,
            output: false,
        };

        for attribute in attributes.iter().filter(|a| a.path().is_ident("lua")) {
            attribute.parse_nested_meta(|meta| {
                if meta.path.is_ident("name") {
                    let value: LitStr = meta.value()?.parse()?;
                    parsed.name = Some(value.value());
                    return Ok(());
                }

                if meta.path.is_ident("input") {
                    parsed.input = true;
                    return Ok(());
                }

                if meta.path.is_ident("output") {
                    parsed.output = true;
                    return Ok(());
                }

                Err(meta.error("expected `name = \"...\"`, `input` or `output`"))
            })?;
        }

        if !parsed.input && !parsed.output {
            parsed.input = true;
            parsed.output = true;
        }

        Ok(parsed)
    }
}

fn field_name(attributes: &[Attribute]) -> syn::Result<Option<String>> {
    let mut rename = None;

    for attribute in attributes.iter().filter(|a| a.path().is_ident("lua")) {
        attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename") {
                let value: LitStr = meta.value()?.parse()?;
                rename = Some(value.value());
                return Ok(());
            }

            Err(meta.error("expected `rename = \"...\"`"))
        })?;
    }

    Ok(rename)
}

pub fn expand(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let Data::Struct(data) = &input.data else {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "LuaClass can only be derived for structs",
        ));
    };

    let Fields::Named(fields) = &data.fields else {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "LuaClass requires named fields",
        ));
    };

    let attributes = ClassAttributes::parse(&input.attrs)?;
    let ident = &input.ident;
    let class_name = attributes.name.unwrap_or_else(|| ident.to_string());

    let mut declarations = Vec::new();
    let mut setters = Vec::new();
    let mut getters = Vec::new();

    for field in &fields.named {
        let Some(field_ident) = field.ident.as_ref() else {
            return Err(syn::Error::new_spanned(field, "expected a named field"));
        };

        let ty = &field.ty;

        let name = field_name(&field.attrs)?
            .unwrap_or_else(|| field_ident.to_string().trim_start_matches("r#").to_owned());

        declarations.push(quote! {
            crate::lua::LuaField {
                name: #name,
                ty: <#ty as crate::lua::LuaTyped>::TYPE,
            }
        });

        setters.push(quote! {
            table.set(#name, self.#field_ident)?;
        });

        getters.push(quote! {
            #field_ident: table.get(#name)?,
        });
    }

    let into_lua = attributes.output.then(|| {
        quote! {
            impl ::mlua::IntoLua for #ident {
                fn into_lua(self, lua: &::mlua::Lua) -> ::mlua::Result<::mlua::Value> {
                    let table = lua.create_table()?;

                    #(#setters)*

                    Ok(::mlua::Value::Table(table))
                }
            }
        }
    });

    let from_lua = attributes.input.then(|| {
        quote! {
            impl ::mlua::FromLua for #ident {
                fn from_lua(value: ::mlua::Value, lua: &::mlua::Lua) -> ::mlua::Result<Self> {
                    let table = <::mlua::Table as ::mlua::FromLua>::from_lua(value, lua)?;

                    Ok(#ident {
                        #(#getters)*
                    })
                }
            }
        }
    });

    Ok(quote! {
        impl #ident {
            pub const CLASS: crate::lua::LuaClass = crate::lua::LuaClass {
                name: #class_name,
                fields: &[#(#declarations),*],
            };
        }

        impl crate::lua::LuaTyped for #ident {
            const TYPE: crate::lua::LuaType = crate::lua::LuaType::Class(&Self::CLASS);
        }

        #into_lua

        #from_lua
    })
}
