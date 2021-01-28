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
///     a -. inject mermaid.js .-> r
///     end
/// ```
///
/// The diagram is going to be located in place of the code snippet
///
/// Dark mode is automatically enabled if `dark` or `ayu` rustdoc theme is selected.
/// You might need to reload the page after the theme change in order to redraw the diagram with an updated theme.
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
/// To learn more, see the [Theming Section](https://mermaid-js.github.io/mermaid/#/theming) of the mermain.js book
pub fn example_with_styling() {}