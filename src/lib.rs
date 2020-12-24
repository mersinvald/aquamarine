extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};

use quote::quote;
use syn::{parse_macro_input, Attribute};

mod attrs;
mod parse;

/// Aquamarine is a proc-macro that adds [Mermaid](https://mermaid-js.github.io/mermaid/#/) diagrams to rustdoc
///
/// To inline a diagram into the documentation, use the `mermaid` snippet:
///
/// ```rust
/// # use aquamarine::aquamarine;
/// #[aquamarine]
/// /// ```mermaid
/// ///   --- here goes your mermaid diagram ---
/// /// ```
/// struct Foo;
/// ```
///
/// The demo crate [can be found here](https://docs.rs/aquamarine-demo-crate)
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
