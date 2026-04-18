mod debug_util;
mod syntax_result;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Field, Fields, FieldsNamed, Ident, Meta, Path};

use crate::{debug_util::DisplayDebugMultiple, syntax_result::with_syntax_errors};

fn is_crate(path: &Path, crate_name: &str) -> bool {
    path.segments
        .first()
        .map(|s| s.ident == crate_name)
        .unwrap_or(false)
}

fn field_defs_from_named_keeping_clap(
    named: &FieldsNamed,
    keep_clap_attributes: bool,
) -> impl Iterator<Item = proc_macro2::TokenStream> + use<'_> {
    named
        .named
        .iter()
        .map(move |f| -> proc_macro2::TokenStream {
            // `attrs` are the attributes on a field definition
            fn field_def<'t>(
                f: &'t Field,
                attrs: impl Iterator<Item = &'t Attribute>,
            ) -> proc_macro2::TokenStream {
                let vis = &f.vis;
                let ident = &f.ident;
                let ty = &f.ty;
                quote! {
                    #(#attrs)*
                    #vis #ident: #ty
                }
            }
            if keep_clap_attributes {
                field_def(f, f.attrs.iter())
            } else {
                let nonclap_attrs = f.attrs.iter().filter(|a| !a.path().is_ident("clap"));
                field_def(f, nonclap_attrs)
            }
        })
}

const DEBUG: bool = false;

#[proc_macro_attribute]
pub fn clap_with_warnings(attr: TokenStream, input: TokenStream) -> TokenStream {
    with_syntax_errors(move || -> syn::Result<proc_macro::TokenStream> {
        if DEBUG {
            eprintln!("attr = {attr}");
            eprintln!("input = {input}");
        }

        let input = syn::parse::<DeriveInput>(input)?;

        let error_only_named_fields = || {
            Err(syn::Error::new_spanned(
                &input,
                "ClapWithWarnings only supports structs with named fields",
            ))
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
                    let field_defs = field_defs_from_named_keeping_clap(named, true);

                    quote! {
                        #[derive(#(#base_derives),*, #(#clap_derives),*)]
                        #(#clap_attrs)*
                        struct #without_warnings_ident {
                            #(#field_defs),*
                        }
                    }
                }
                Fields::Unnamed(_unnamed) => return error_only_named_fields(),
                Fields::Unit => return error_only_named_fields(),
            },
            _ => return error_only_named_fields(),
        };

        // Rebuild the struct with all clap traces removed, but with the
        // original name
        let derived = match &input.data {
            Data::Struct(s) => match &s.fields {
                Fields::Named(named) => {
                    let field_defs = field_defs_from_named_keeping_clap(named, false);

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
                Fields::Unnamed(_unnamed) => return error_only_named_fields(),
                Fields::Unit => return error_only_named_fields(),
            },
            _ => return error_only_named_fields(),
        };

        let all: proc_macro2::TokenStream = quote! {
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

        Ok(all.into())
    })
}
