//! A demo crate for [aquamarine](https://docs.rs/aquamarine)

#[cfg_attr(doc, aquamarine::aquamarine)]
/// A function showcasing aquamarine
///
/// With aquamarine it's possible to embed Mermaid diagrams into your Rust documentation using the code snippets
/// ```mermaid
/// graph LR
///     s([Source]) --> a[[aquamarine]]
///     r[[rustdoc]] --> f([Docs w/ Mermaid!])
///     subgraph rustc[Rust Compiler]
///     a -. inject mermaid.js .-> r
///     end
/// ```
/// The diagram is going to be located in place of the code snippet
pub fn example() {}
