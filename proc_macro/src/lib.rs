extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn dag(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(item as ItemFn);
    input_fn.block.stmts.push(syn::parse_quote!(parse_cli();));

    // Convert the modified function back to a TokenStream
    TokenStream::from(quote!(#input_fn))
}
