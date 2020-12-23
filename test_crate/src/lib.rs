/// An example structure showcasing the `#[aquamarine]` attribute macro
///
/// Open the source code to see the details
pub struct Example;

impl Example {
    #[aquamarine::aquamarine(mermaid = r#"
        graph TB;
            A & B --> C & D;
    "#)]
    /// mermaid diagram can be defined as a string in the attribute macro parameter
    ///
    /// By default the diagrams are placed under the rest of the documentation
    pub fn parameter() {}

    // TODO: support include_str expression
    /*
    #[aquamarine::aquamarine(placement = "top", mermaid = include_str!("../data/example.mdd"))]
    /// A bigger Mermaid diagram definition can also be loaded from a file, and provided into the attribute macro parameter
    ///
    /// Placement of the diagram can be configured with the 'placement' parameter and can either be 'top' or 'bottom'
    pub fn parameter_with_include() {}
    */

    #[aquamarine::aquamarine]
    /// It's also possible to inline the diagrams as code blocks
    /// ```mermaid
    /// sequenceDiagram
    ///     participant Alice
    ///     participant Bob
    ///     Alice->>John: Hello John, how are you?
    ///     loop Healthcheck
    ///         John->>John: Fight against hypochondria
    ///     end
    ///     Note right of John: Rational thoughts <br/>prevail!
    ///     John-->>Alice: Great!
    ///     John->>Bob: How about you?
    ///     Bob-->>John: Jolly good!
    /// ```
    /// In this case, the diagram is going to be located in place of the code block,
    /// unless the 'placement' parameter is specified
    pub fn inline_code_block() {}
}