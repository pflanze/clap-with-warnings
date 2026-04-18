pub fn with_syntax_errors(
    f: impl FnOnce() -> syn::Result<proc_macro::TokenStream>,
) -> proc_macro::TokenStream {
    match f() {
        Ok(v) => v,
        Err(e) => e.into_compile_error().into(),
    }
}
