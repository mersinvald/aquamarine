extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};

use quote::quote;
use syn::{parse_macro_input, Attribute};

mod attrs;
mod parse;

/// Example of user-defined [procedural macro attribute][1].
///
/// [1]: https://doc.rust-lang.org/reference/procedural-macros.html#attribute-macros

#[proc_macro_attribute]
#[proc_macro_error]
pub fn aquamarine(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as parse::Input);

    check_input_attrs(&input.attrs);

    let attrs = attrs::Attrs::from(input.attrs);
    let forward = input.rest;

    let tokens = quote! {
        #attrs
        #forward
    };

    tokens.into()
}

fn check_input_attrs(input: &[Attribute]) {
    for attr in input {
        if attr.path.is_ident("aquamarine") {
            abort!(
                attr,
                "multiple `aquamarine` attributes on one entity are illegal"
            );
        }
    }
}
