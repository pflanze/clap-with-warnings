mod debug_util;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Attribute, Data, DeriveInput, Fields, FieldsNamed, Ident, Meta, Path,
};

use crate::debug_util::DisplayDebugMultiple;

fn is_crate(path: &Path, crate_name: &str) -> bool {
    path.segments
        .first()
        .map(|s| s.ident == crate_name)
        .unwrap_or(false)
}

fn field_defs_from_named(
    named: &FieldsNamed,
    keep_clap_attributes: bool,
) -> impl Iterator<Item = proc_macro2::TokenStream> + use<'_> {
    named.named.iter().map(move |f| {
        let attrs = f
            .attrs
            .iter()
            .filter(|a| keep_clap_attributes || !a.path().is_ident("clap"));
        let vis = &f.vis;
        let ident = &f.ident;
        let ty = &f.ty;

        quote! {
            #(#attrs)*
            #vis #ident: #ty
        }
    })
}

const DEBUG: bool = false;

#[proc_macro_attribute]
pub fn clap_with_warnings(attr: TokenStream, input: TokenStream) -> TokenStream {
    if DEBUG {
        eprintln!("attr = {attr}");
        eprintln!("input = {input}");
    }

    let input = parse_macro_input!(input as DeriveInput);

    let syntax_error = || {
        syn::Error::new_spanned(
            &input,
            "ClapWithWarnings only supports structs with named fields",
        )
        .to_compile_error()
        .into()
    };

    let original_ident = &input.ident;
    let without_warnings_ident = Ident::new(
        &format!("{original_ident}WithoutWarnings"),
        original_ident.span(),
    );

    let mut base_derives: Vec<Path> = Vec::new();
    let mut clap_derives: Vec<Path> = Vec::new();

    let mut clap_attrs: Vec<Attribute> = Vec::new();
    let mut other_attrs: Vec<Attribute> = Vec::new();
    for attr in &input.attrs {
        if attr.path().is_ident("derive") {
            if let Meta::List(list) = &attr.meta {
                list.parse_nested_meta(|meta| {
                    let path = meta.path;
                    if path.is_ident("ClapWithWarnings") {
                        ()
                    } else if is_crate(&path, "clap") {
                        clap_derives.push(path);
                    } else {
                        base_derives.push(path);
                    }
                    Ok(())
                })
                .unwrap();
            }
        } else if attr.path().is_ident("clap") {
            clap_attrs.push(attr.clone());
        } else {
            other_attrs.push(attr.clone());
        }
    }

    if DEBUG {
        eprintln!(
            "base_derives={:?}",
            DisplayDebugMultiple::from(&base_derives)
        );
        eprintln!(
            "clap_derives={:?}",
            DisplayDebugMultiple::from(&clap_derives)
        );

        eprintln!("clap_attrs={:?}", DisplayDebugMultiple::from(&clap_attrs));
        eprintln!("other_attrs={:?}", DisplayDebugMultiple::from(&other_attrs));
    }

    // Rebuild the struct with clap::* derives and #[clap(...)]
    // attributes on struct and fields, i.e. everything the same as
    // `input`, except with different name. (XX could we instead
    // mutate?)
    let without_warnings_struct = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(named) => {
                let field_defs = field_defs_from_named(named, true);

                quote! {
                    #[derive(#(#base_derives),*, #(#clap_derives),*)]
                    #(#clap_attrs)*
                    struct #without_warnings_ident {
                        #(#field_defs),*
                    }
                }
            }
            Fields::Unnamed(_unnamed) => return syntax_error(),
            Fields::Unit => return syntax_error(),
        },
        _ => return syntax_error(),
    };

    // Rebuild the struct with all clap traces removed, but with the
    // original name
    let derived = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(named) => {
                let field_defs = field_defs_from_named(named, false);

                let field_names = named.named.iter().map(|f| {
                    let ident = &f.ident;
                    quote! {
                        #ident
                    }
                });

                let field_names2 = field_names.clone();

                quote! {
                    #[derive(#(#base_derives),*)]
                    struct #original_ident {
                        #(#field_defs),*
                    }

                    impl #without_warnings_ident {
                        pub fn with_warnings(self) -> #original_ident {
                            let #without_warnings_ident { #(#field_names),* } = self;
                            #original_ident { #(#field_names2),* }
                        }
                    }
                }
            }
            Fields::Unnamed(_unnamed) => return syntax_error(),
            Fields::Unit => return syntax_error(),
        },
        _ => return syntax_error(),
    };

    let all = quote! {
        #without_warnings_struct

        #derived

        impl #original_ident {
            pub fn parse() -> #original_ident {
                #without_warnings_ident :: parse().with_warnings()
            }
        }
    };

    if DEBUG {
        eprintln!("OK, returning:\n\n{all}\n");
    }

    all.into()
}
