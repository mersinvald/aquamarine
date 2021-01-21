# Aquamarine

[![GitHub](https://img.shields.io/github/license/mersinvald/aquamarine)](LICENSE)
[![crates.io](https://img.shields.io/crates/d/aquamarine)](https://crates.io/crates/aquamarine)
[![docs.rs](https://docs.rs/aquamarine/badge.svg)](https://docs.rs/aquamarine)


Aquamarine is a procedural macro extension for [rustdoc](https://github.com/mersinvald/dotfiles), 
that aims to improve the visual component of Rust documentation through use of the [mermaid.js](https://mermaid-js.github.io/mermaid/#/) diagrams.

`#[aquamarine]` macro works through embedding the [mermaid.js](https://github.com/mermaid-js/mermaid) into the generated rustdoc HTML page, modifying the doc comment attributes.

To inline a diagram into the documentation, use the `mermaid` snippet in a doc-string:

```rust 
# use aquamarine::aquamarine
#[aquamarine]
/// ```mermaid
/// graph LR
///     s([Source]) --> a[[aquamarine]]
///     r[[rustdoc]] --> f([Docs w/ Mermaid!])
///     subgraph rustc[Rust Compiler]
///     a -. inject mermaid.js .-> r
///     end
/// ```
pub fn example() {}
``` 
The diagram will appear in place of the `mermaid` code block, preserving all the comments around it. You can even add multiple diagrams!

To see it in action, go to the [demo crate](https://docs.rs/aquamarine-demo-crate) docs.rs page.

You can learn more about `mermaid.js` and what it can do in the mermaid's [documentation MdBook](https://mermaid-js.github.io/mermaid/#/)

