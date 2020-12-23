extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro_error::{proc_macro_error, abort, ResultExt};

use syn::{parse_macro_input, Attribute};
use quote::quote;

mod parse;
mod attrs;

/// Example of user-defined [procedural macro attribute][1].
///
/// [1]: https://doc.rust-lang.org/reference/procedural-macros.html#attribute-macros

#[proc_macro_attribute]
#[proc_macro_error]
pub fn aquamarine(args: TokenStream, input: TokenStream) -> TokenStream {
    let _args = parse_macro_input!(args as parse::Args);
    let input = parse_macro_input!(input as parse::Input);

    check_attrs(&input.attrs);
    // TODO load diagram from proc macro attrs
    // Make methods like push_diagram, push_diagram_from_file, push_attributes
    let attrs = attrs::convert_attrs(input.attrs).unwrap_or_abort();
    let tokens = input.rest;
    let tokens = quote! {
        #attrs
        #tokens
    };

    tokens.into()
}

fn check_attrs(input: &[Attribute]) {
    for attr in input {
        // TODO: support multiple aquamarine entries
        if attr.path.is_ident("aquamarine") {
            abort!(attr, "multiple `aquamarine` attributes aren't supported -- use the doc comments instead");
        }
    }
}