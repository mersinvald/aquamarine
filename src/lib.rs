//! Aquamarine is a procedural macro extension for [rustdoc](https://doc.rust-lang.org/rustdoc/what-is-rustdoc.html), 
//! that aims to improve the visual component of Rust documentation through use of the [mermaid.js](https://mermaid-js.github.io/mermaid/#/) diagrams.
//!
//! `#[aquamarine]` macro works through embedding the [mermaid.js](https://github.com/mermaid-js/mermaid) into the generated rustdoc HTML page, modifying the doc comment attributes.
//!
//! To inline a diagram into the documentation, use the `mermaid` snippet in a doc-string:
//!
//! ```rust 
//! # use aquamarine::aquamarine
//! #[aquamarine]
//! /// ```mermaid
//! /// graph LR
//! ///     s([Source]) --> a[[aquamarine]]
//! ///     r[[rustdoc]] --> f([Docs w/ Mermaid!])
//! ///     subgraph rustc[Rust Compiler]
//! ///     a -. inject mermaid.js .-> r
//! ///     end
//! /// ```
//! pub fn example() {}
//! ``` 
//! The diagram will appear in place of the `mermaid` code block, preserving all the comments around it.
//!
//! You can even add multiple diagrams!
//!
//! To see it in action, go to the [demo crate](https://docs.rs/aquamarine-demo-crate/0.1.2/aquamarine_demo_crate/fn.example.html) docs.rs page.

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
