use proc_macro::TokenStream;

#[proc_macro]
pub fn impl_for_tuple(input: TokenStream) -> TokenStream {
    match internal::impl_for_tuple(input.into()) {
        Ok(output) => output.into(),
        Err(error) => error.to_compile_error().into(),
    }
}