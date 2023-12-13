//! A demo crate for [aquamarine](https://docs.rs/aquamarine)

#[cfg_attr(doc, aquamarine::aquamarine)]
/// A function showcasing aquamarine defaults
///
/// With aquamarine it's possible to embed Mermaid diagrams into your Rust documentation using the code snippets
/// 
/// ```mermaid
/// graph LR
///     s([Source]) --> a[[aquamarine]]
///     r[[rustdoc]] --> f([Docs w/ Mermaid!])
///     subgraph rustc[Rust Compiler]
///     a -. "inject mermaid.js" .-> r
///     end
/// ```
///
/// The diagram is going to be located in place of the code snippet
///
/// Dark mode is automatically enabled if `dark` or `ayu` rustdoc theme is selected.
///
/// You might need to reload the page to redraw the diagrams after changing the theme.
pub fn example() {}

#[cfg_attr(doc, aquamarine::aquamarine)]
/// You can apply custom themes on per-diagram basis using the %%init%% annotation
///
/// ```mermaid
/// %%{init: {
///     'theme': 'base',
///     'themeVariables': {
///            'primaryColor': '#ffcccc', 
///            'edgeLabelBackground':'#ccccff', 
///            'tertiaryColor': '#fff0f0' }}}%%
/// graph TD
///      A(Diagram needs to be drawn) --> B{Does it have 'init' annotation?}
///      B -->|No| C(Apply default theme)
///      B -->|Yes| D(Apply customized theme)
/// ```
///
/// To learn more, see the [Theming Section](https://mermaid-js.github.io/mermaid/#/theming) of the mermaid.js book
pub fn example_with_styling() {}

#[cfg_attr(doc, aquamarine::aquamarine)]
/// A diagram can be loaded from a file as well!
/// 
/// include_mmd!("diagram_0.mmd")
/// 
/// Reduce clutter in your doc comments, when a diagram is big enough.
/// 
/// You can include multiple diagrams in a single doc comment, using the macro-like syntax `include_mmd!("/path/to/diagram.mmd")`
/// 
/// include_mmd!(diagram_1.mmd)
///
/// **Note:** `indlude_mmd!` syntax is only supported inside doc comments
pub fn example_load_from_file() {}
