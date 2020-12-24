# Aquamarine

Aquamarine is a proc-macro that aims to add the visual component to the Rust documentation.

In a nutshell, aquamarine parses the documentation comments and injects the [mermaid.js](https://github.com/mermaid-js/mermaid) in place of the `mermaid` code snippets inside your Rust documentation strings.

```rust
#[aquamarine::aquamarine]
/// With aquamarine you can embed Mermaid diagrams into your Rust documentation using the code snippets
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
```

To see it in action, go to the [demo crate](https://docs.rs/aquamarine-demo-crate/) page on docs.rs

To learn more about Mermaid, see the [documentation MdBook](https://mermaid-js.github.io/mermaid/#/)

