extern crate proc_macro;

use darling::FromMeta;
use proc_macro::TokenStream;
use syn::{spanned::Spanned, AttributeArgs, ItemFn};

#[derive(Debug, FromMeta)]
struct Args {
    #[darling(default)]
    key: Option<String>,
}

#[proc_macro_attribute]
pub fn ui(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input: ItemFn = syn::parse(input).unwrap();

    let inner_block = input.block;
    input.block = syn::parse_quote! {{ cape::call(move || #inner_block) }};

    quote::quote_spanned!(input.span() =>
        #[track_caller]
        #input
    )
    .into()
}

#[proc_macro_attribute]
pub fn unique_ui(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = syn::parse_macro_input!(args as AttributeArgs);
    let mut input = syn::parse_macro_input!(input as ItemFn);

    let args = match Args::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };

    let key = args.key.unwrap_or_else(|| String::from("key"));
    let key = quote::format_ident!("{}", key);

    let inner_block = input.block;
    input.block = syn::parse_quote! {{ cape::call_unique(#key, move || #inner_block) }};

    quote::quote_spanned!(input.span() =>
        #[track_caller]
        #input
    )
    .into()
}
