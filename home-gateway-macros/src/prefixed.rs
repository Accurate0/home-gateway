use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{FnArg, ImplItem, ItemImpl, LitStr, Pat, ReturnType, Type, parse_quote};

/// Emits the impl unchanged plus a `<Type>V2` twin whose resolvers are renamed to
/// `<prefix>_<field>` and delegate to the original. Every attribute is copied, so guards,
/// docs and argument defaults stay identical. Place it above `#[Object]`/`#[Subscription]`.
pub fn expand(arguments: TokenStream, input: ItemImpl) -> syn::Result<TokenStream> {
    let prefix: LitStr = syn::parse2(arguments)?;
    let prefix = prefix.value();

    let Type::Path(self_ty) = input.self_ty.as_ref() else {
        return Err(syn::Error::new_spanned(
            &input.self_ty,
            "expected a type path",
        ));
    };
    let ident = &self_ty.path.segments.last().expect("type path").ident;
    let twin = format_ident!("{ident}V2");

    let mut methods = Vec::new();
    for item in &input.items {
        let ImplItem::Fn(method) = item else {
            return Err(syn::Error::new_spanned(
                item,
                "only resolvers are supported",
            ));
        };

        let mut sig = method.sig.clone();
        sig.ident = format_ident!("{prefix}_{}", method.sig.ident);

        let mut has_context = false;
        let mut arguments = Vec::new();
        for input in &method.sig.inputs {
            let FnArg::Typed(typed) = input else { continue };
            let Pat::Ident(pat) = typed.pat.as_ref() else {
                return Err(syn::Error::new_spanned(
                    &typed.pat,
                    "expected a named argument",
                ));
            };
            has_context |= is_context(&typed.ty);
            arguments.push(pat.ident.clone());
        }

        // `#[Object]` adds the context to resolvers without one and wraps their output in a Result.
        if !has_context {
            let context: FnArg = parse_quote!(ctx: &async_graphql::Context<'_>);
            sig.inputs.insert(1, context);
            arguments.insert(0, format_ident!("ctx"));

            if let ReturnType::Type(_, output) = &sig.output
                && !is_result(output)
            {
                sig.output = parse_quote!(-> async_graphql::Result<#output>);
            }
        }

        let attrs = &method.attrs;
        let name = &method.sig.ident;
        methods.push(quote! {
            #(#attrs)*
            #sig {
                <#ident as ::core::default::Default>::default().#name(#(#arguments),*).await
            }
        });
    }

    let attrs = &input.attrs;
    Ok(quote! {
        #input

        #[derive(Default)]
        pub struct #twin;

        #(#attrs)*
        impl #twin {
            #(#methods)*
        }
    })
}

fn is_context(ty: &Type) -> bool {
    let Type::Reference(reference) = ty else {
        return false;
    };
    let Type::Path(path) = reference.elem.as_ref() else {
        return false;
    };
    path.path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "Context")
}

fn is_result(ty: &Type) -> bool {
    let Type::Path(path) = ty else {
        return false;
    };
    path.path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "Result")
}
