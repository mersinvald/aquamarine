#[aquamarine::aquamarine]
/// A function showcasing aquamarine
///
/// With aquamarine it's possible to embed Mermaid diagrams into your Rust documentation, using the code snippet syntax
/// ```mermaid
/// graph LR
///     Source --> aquamarine -- inject mermaid.js --> ms[Modified Source] --> rustdoc --> f[Docs w/ Mermaid!]
/// ```
/// The diagram is going to be located in place of the code snippet
pub fn example() {}
