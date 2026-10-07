use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::parse::Parser;
use syn::{
    FnArg, GenericArgument, Ident, ImplItem, ImplItemFn, ItemImpl, LitStr, Meta, Pat, Path,
    PathArguments, ReturnType, Type,
};

struct ModuleArguments {
    namespace: String,
    requires: Option<Type>,
}

impl ModuleArguments {
    fn parse(arguments: TokenStream) -> syn::Result<Self> {
        let mut namespace = None;
        let mut requires = None;

        let parser = syn::meta::parser(|meta| {
            if meta.path.is_ident("namespace") {
                let value: LitStr = meta.value()?.parse()?;
                namespace = Some(value.value());
                return Ok(());
            }

            if meta.path.is_ident("requires") {
                requires = Some(meta.value()?.parse()?);
                return Ok(());
            }

            Err(meta.error("expected `namespace = \"...\"` or `requires = Type`"))
        });

        parser.parse2(arguments)?;

        let Some(namespace) = namespace else {
            return Err(syn::Error::new(
                Span::call_site(),
                "lua_module requires `namespace = \"...\"`",
            ));
        };

        Ok(ModuleArguments {
            namespace,
            requires,
        })
    }
}

struct FunctionAttributes {
    name: Option<String>,
    scope: Option<Path>,
}

impl FunctionAttributes {
    fn take(function: &mut ImplItemFn) -> syn::Result<Option<Self>> {
        let mut found = None;
        let mut kept = Vec::with_capacity(function.attrs.len());

        for attribute in function.attrs.drain(..) {
            if !attribute.path().is_ident("lua") {
                kept.push(attribute);
                continue;
            }

            let attributes = found.get_or_insert(FunctionAttributes {
                name: None,
                scope: None,
            });

            if matches!(attribute.meta, Meta::Path(_)) {
                continue;
            }

            attribute.parse_nested_meta(|meta| {
                if meta.path.is_ident("name") {
                    let value: LitStr = meta.value()?.parse()?;
                    attributes.name = Some(value.value());
                    return Ok(());
                }

                if meta.path.is_ident("scope") {
                    attributes.scope = Some(meta.value()?.parse()?);
                    return Ok(());
                }

                Err(meta.error("expected `name = \"...\"` or `scope = Resource::Action`"))
            })?;
        }

        function.attrs = kept;

        Ok(found)
    }
}

enum Argument {
    Context,
    Lua,
    Value { ident: Ident, ty: Box<Type> },
}

impl Argument {
    fn parse(argument: &FnArg) -> syn::Result<Self> {
        let FnArg::Typed(typed) = argument else {
            return Err(syn::Error::new_spanned(
                argument,
                "a lua function takes no `self`",
            ));
        };

        if let Type::Reference(reference) = typed.ty.as_ref()
            && let Type::Path(path) = reference.elem.as_ref()
            && let Some(segment) = path.path.segments.last()
        {
            if segment.ident == "LuaCallContext" {
                return Ok(Argument::Context);
            }

            if segment.ident == "Lua" {
                return Ok(Argument::Lua);
            }
        }

        let Pat::Ident(pattern) = typed.pat.as_ref() else {
            return Err(syn::Error::new_spanned(
                &typed.pat,
                "a lua function parameter must be a plain name",
            ));
        };

        Ok(Argument::Value {
            ident: pattern.ident.clone(),
            ty: typed.ty.clone(),
        })
    }
}

fn returned_type(function: &ImplItemFn) -> syn::Result<&Type> {
    let error = || {
        syn::Error::new_spanned(
            &function.sig,
            "a lua function must return `mlua::Result<T>`",
        )
    };

    let ReturnType::Type(_, ty) = &function.sig.output else {
        return Err(error());
    };

    let Type::Path(path) = ty.as_ref() else {
        return Err(error());
    };

    let Some(segment) = path.path.segments.last() else {
        return Err(error());
    };

    if segment.ident != "Result" {
        return Err(error());
    }

    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return Err(error());
    };

    match arguments.args.first() {
        Some(GenericArgument::Type(ty)) => Ok(ty),
        _ => Err(error()),
    }
}

fn scope_tokens(scope: &Path) -> syn::Result<TokenStream> {
    let segments: Vec<&Ident> = scope.segments.iter().map(|s| &s.ident).collect();

    let [resource, action] = segments.as_slice() else {
        return Err(syn::Error::new_spanned(
            scope,
            "expected `scope = Resource::Action`",
        ));
    };

    Ok(quote! {
        ::std::option::Option::Some(crate::auth::scope::Scope::new(
            crate::auth::scope::Resource::#resource,
            crate::auth::scope::Action::#action,
        ))
    })
}

struct Exposed {
    name_const: TokenStream,
    declaration: TokenStream,
    registration: TokenStream,
}

fn expose(
    namespace: &str,
    index: usize,
    function: &ImplItemFn,
    attributes: FunctionAttributes,
) -> syn::Result<Exposed> {
    let ident = &function.sig.ident;
    let rust_name = ident.to_string().trim_start_matches("r#").to_owned();
    let lua_name = attributes.name.unwrap_or_else(|| rust_name.clone());
    let qualified = format!("{namespace}.{lua_name}");
    let const_ident = format_ident!("{}", rust_name.to_uppercase(), span = ident.span());

    let returned = returned_type(function)?;

    let scope = match &attributes.scope {
        Some(scope) => scope_tokens(scope)?,
        None => quote!(::std::option::Option::None),
    };

    let arguments = function
        .sig
        .inputs
        .iter()
        .map(Argument::parse)
        .collect::<syn::Result<Vec<_>>>()?;

    let uses_context = arguments.iter().any(|a| matches!(a, Argument::Context));
    let is_async = function.sig.asyncness.is_some();

    let mut params = Vec::new();
    let mut names = Vec::new();
    let mut types = Vec::new();

    for argument in &arguments {
        if let Argument::Value { ident, ty } = argument {
            let name = ident.to_string().trim_start_matches("r#").to_owned();

            params.push(quote! {
                crate::lua::LuaParam {
                    name: #name,
                    ty: <#ty as crate::lua::LuaTyped>::TYPE,
                }
            });

            names.push(ident);
            types.push(ty);
        }
    }

    let pattern = match names.as_slice() {
        [] => quote!((): ()),
        [name] => {
            let ty = types[0];
            quote!(#name: #ty)
        }
        _ => quote!((#(#names),*): (#(#types),*)),
    };

    let lua_argument = if is_async {
        quote!(&__lua)
    } else {
        quote!(__lua)
    };

    let call_arguments = arguments.iter().map(|argument| match argument {
        Argument::Context => quote!(&cx),
        Argument::Lua => lua_argument.clone(),
        Argument::Value { ident, .. } => quote!(#ident),
    });

    let outer_clone = uses_context.then(|| quote!(let cx = cx.clone();));

    let closure = if is_async {
        let inner_clone = uses_context.then(|| quote!(let cx = cx.clone();));

        quote! {
            lua.create_async_function(move |__lua, #pattern| {
                #inner_clone

                async move { Self::#ident(#(#call_arguments),*).await }
            })
        }
    } else {
        quote! {
            lua.create_function(move |__lua, #pattern| Self::#ident(#(#call_arguments),*))
        }
    };

    Ok(Exposed {
        name_const: quote! {
            #[allow(dead_code)]
            pub const #const_ident: &'static str = #qualified;
        },
        declaration: quote! {
            crate::lua::LuaFunction {
                name: #lua_name,
                params: &[#(#params),*],
                returns: <#returned as crate::lua::LuaReturns>::RETURNS,
                scope: #scope,
            }
        },
        registration: quote! {
            cx.expose(table, &Self::FUNCTIONS[#index], || {
                #outer_clone

                #closure
            })?;
        },
    })
}

pub fn expand(arguments: TokenStream, mut input: ItemImpl) -> syn::Result<TokenStream> {
    let arguments = ModuleArguments::parse(arguments)?;

    if let Some((path, _)) = &input.trait_ {
        return Err(syn::Error::new_spanned(
            path,
            "lua_module goes on an inherent impl",
        ));
    }

    let namespace = arguments.namespace;
    let mut exposed = Vec::new();

    for item in &mut input.items {
        let ImplItem::Fn(function) = item else {
            continue;
        };

        let Some(attributes) = FunctionAttributes::take(function)? else {
            continue;
        };

        exposed.push(expose(&namespace, exposed.len(), function, attributes)?);
    }

    let self_ty = &input.self_ty;
    let (impl_generics, _, where_clause) = input.generics.split_for_impl();

    let name_consts = exposed.iter().map(|e| &e.name_const);
    let declarations = exposed.iter().map(|e| &e.declaration);
    let registrations = exposed.iter().map(|e| &e.registration);

    let gate = arguments.requires.map(|requires| {
        quote! {
            if !cx.state.handles.contains::<#requires>() {
                return Ok(());
            }
        }
    });

    Ok(quote! {
        #input

        impl #impl_generics #self_ty #where_clause {
            #(#name_consts)*

            pub const FUNCTIONS: &'static [crate::lua::LuaFunction] = &[#(#declarations),*];
        }

        impl #impl_generics crate::lua::LuaModule for #self_ty #where_clause {
            fn namespace(&self) -> &'static str {
                #namespace
            }

            fn functions(&self) -> &'static [crate::lua::LuaFunction] {
                Self::FUNCTIONS
            }

            fn register(
                &self,
                lua: &::mlua::Lua,
                table: &::mlua::Table,
                cx: &crate::lua::LuaCallContext,
            ) -> ::mlua::Result<()> {
                #gate

                #(#registrations)*

                Ok(())
            }
        }
    })
}
