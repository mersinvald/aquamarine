/// An example structure showcasing the `#[aquamarine]` attribute macro
///
/// Open the source code to see the details
struct Example;

impl Example {
    #[aquamarine::aquamarine(mermade = r#"
        graph TD;
            A-->B;
            A-->C;
            B-->D;
            C-->D;
    "#)]
    /// Mermade diagram can be defined as a string in the attribute macro parameter
    /// By default the diagrams are placed under the rest of the documentation
    pub fn parameter() {}

    #[aquamarine::aquamarine(placement = "top", mermade = include_str!("../data/example.mdd"))]
    /// Mermade diagram can also be loaded form a file, and provided into the attribute macro parameter
    /// Placement of the diagram can be configured with the 'placement' parameter and can either be 'top' or 'bottom'
    pub fn parameter_with_include() {}

    #[aquamarine::aquamarine]
    /// It's also possible to inline the diagrams as code blocks
    /// ```mermade
    /// graph TD;
    ///     A-->B;
    ///     A-->C;
    ///     B-->D;
    ///     C-->D;
    /// ```
    /// In this case, the diagram is going to be located in place of the code block,
    /// unless the 'placement' parameter is specified
    pub fn inline_code_block() {}
}