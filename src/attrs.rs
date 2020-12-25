use itertools::Itertools;
use proc_macro2::TokenStream;
use proc_macro_error::{abort, emit_call_site_warning};
use quote::quote;
use std::iter;
use syn::{Attribute, Ident, MetaNameValue};

const UNEXPECTED_ATTR_ERROR: &str =
    "unexpected attribute inside a diagram definition: only #[doc] is allowed";

#[derive(Clone, Default)]
pub struct Attrs(Vec<Attr>);

#[derive(Clone)]
pub enum Attr {
    /// Attribute that is to be forwarded as-is
    Forward(Attribute),
    /// Doc comment that cannot be forwarded as-is
    DocComment(Ident, String),
    /// Diagram start token
    DiagramStart(Ident),
    /// Diagram entry (line)
    DiagramEntry(Ident, String),
    /// Diagram end token
    DiagramEnd(Ident),
}

impl Attr {
    pub fn as_ident(&self) -> Option<&Ident> {
        match self {
            Attr::Forward(attr) => attr.path.get_ident(),
            Attr::DocComment(ident, _) => Some(ident),
            Attr::DiagramStart(ident) => Some(ident),
            Attr::DiagramEntry(ident, _) => Some(ident),
            Attr::DiagramEnd(ident) => Some(ident),
        }
    }

    pub fn is_diagram_end(&self) -> bool {
        matches!(self, Attr::DiagramEnd(_))
    }

    pub fn expect_diagram_entry_text(&self) -> &str {
        match self {
            Attr::DiagramEntry(_, body) => body.as_str(),
            _ => abort!(self.as_ident(), UNEXPECTED_ATTR_ERROR),
        }
    }
}

impl From<Vec<Attribute>> for Attrs {
    fn from(attrs: Vec<Attribute>) -> Self {
        let mut out = Attrs::default();
        out.push_attrs(attrs);
        out
    }
}

impl quote::ToTokens for Attrs {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut attrs = self.0.iter();
        while let Some(attr) = attrs.next() {
            match attr {
                Attr::Forward(attr) => attr.to_tokens(tokens),
                Attr::DocComment(_, comment) => tokens.extend(quote! {
                    #[doc = #comment]
                }),
                Attr::DiagramStart(_) => {
                    let diagram = attrs
                        .by_ref()
                        .take_while(|x| !x.is_diagram_end())
                        .map(Attr::expect_diagram_entry_text);

                    tokens.extend(generate_diagram_rustdoc(diagram));
                }
                // If that happens, then the parsing stage is faulty: doc comments outside of
                // in between Start and End tokens are to be emitted as Attr::Forward or Attr::DocComment
                Attr::DiagramEntry(_, body) => {
                    emit_call_site_warning!("encountered an unexpected attribute that's going to be ignored, this is a bug! ({})", body);
                }
                Attr::DiagramEnd(_) => (),
            }
        }
    }
}

fn generate_diagram_rustdoc<'a>(parts: impl Iterator<Item = &'a str>) -> TokenStream {
    let preamble = iter::once(r#"<div class="mermaid">"#);
    let postamble = iter::once("</div>");

    let body = preamble.chain(parts).chain(postamble).join("\n");

    quote! {
        #[doc = r#"<script src="https://cdn.jsdelivr.net/npm/mermaid/dist/mermaid.min.js"></script>"#]
        #[doc = r#"<script>window.mermaid == null && mermaid.initialize({startOnLoad:true});</script>"#]
        #[doc = #body]
    }
}

impl Attrs {
    pub fn push_attrs(&mut self, attrs: Vec<Attribute>) {
        use syn::Lit::*;
        use syn::Meta::*;

        let mut current_location = Location::OutsideDiagram;
        let mut diagram_start_ident = None;

        for attr in attrs {
            match attr.parse_meta() {
                Ok(NameValue(MetaNameValue {
                    lit: Str(s), path, ..
                })) if path.is_ident("doc") => {
                    let ident = path.get_ident().unwrap();
                    for attr in split_attr_body(ident, &s.value(), &mut current_location) {
                        if matches!(attr, Attr::DiagramStart(_)) {
                            diagram_start_ident.replace(ident.clone());
                        }
                        self.0.push(attr);
                    }
                }
                _ => {
                    if let Location::InsideDiagram = current_location {
                        abort!(attr, UNEXPECTED_ATTR_ERROR)
                    } else {
                        self.0.push(Attr::Forward(attr))
                    }
                }
            }
        }

        if current_location.is_inside() {
            abort!(diagram_start_ident, "diagram code block is not terminated");
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Location {
    OutsideDiagram,
    InsideDiagram,
}

impl Location {
    fn is_inside(self) -> bool {
        matches!(self, Location::InsideDiagram)
    }
}

fn split_attr_body(ident: &Ident, input: &str, loc: &mut Location) -> Vec<Attr> {
    use DocToken::*;
    use Location::*;

    const MERMAID: &str = "mermaid";

    let mut tokens = tokenize_doc_str(input).peekable();

    // Special case: empty strings outside the diagram span should be still generated
    if tokens.peek().is_none() && !loc.is_inside() {
        return vec![Attr::DocComment(ident.clone(), String::new())];
    };

    // To aid rustc with type inference in closures
    #[derive(Default)]
    struct Ctx<'a> {
        attrs: Vec<Attr>,
        buffer: Vec<&'a str>,
    }

    let mut ctx = Default::default();

    let flush_buffer_as_doc_comment = |ctx: &mut Ctx| {
        if !ctx.buffer.is_empty() {
            ctx.attrs.push(Attr::DocComment(
                ident.clone(),
                ctx.buffer.drain(..).join(" "),
            ));
        }
    };

    let flush_buffer_as_diagram_entry = |ctx: &mut Ctx| {
        let s = ctx.buffer.drain(..).join(" ");
        if !s.trim().is_empty() {
            ctx.attrs.push(Attr::DiagramEntry(ident.clone(), s));
        }
    };

    while let Some(token) = tokens.next() {
        match (*loc, &token, tokens.peek()) {
            // Flush the buffer, then open the diagram code block
            (OutsideDiagram, Ticks, Some(Word(MERMAID))) => {
                tokens.next();
                *loc = InsideDiagram;
                flush_buffer_as_doc_comment(&mut ctx);
                ctx.attrs.push(Attr::DiagramStart(ident.clone()));
            }
            // Flush the buffer, close the code block
            (InsideDiagram, Ticks, _) => {
                *loc = OutsideDiagram;
                flush_buffer_as_diagram_entry(&mut ctx);
                ctx.attrs.push(Attr::DiagramEnd(ident.clone()))
            }
            _ => ctx.buffer.push(token.as_str()),
        }
    }

    if !ctx.buffer.is_empty() {
        if loc.is_inside() {
            flush_buffer_as_diagram_entry(&mut ctx);
        } else {
            flush_buffer_as_doc_comment(&mut ctx);
        };
    }

    ctx.attrs
}

enum DocToken<'a> {
    Ticks,
    Word(&'a str),
}

impl<'a> DocToken<'a> {
    fn as_str(&self) -> &'a str {
        match self {
            DocToken::Ticks => "```",
            DocToken::Word(s) => s,
        }
    }
}

fn tokenize_doc_str(input: &str) -> impl Iterator<Item = DocToken> {
    const TICKS: &str = "```";
    split_inclusive(input, TICKS)
        .flat_map(|token| {
            // not str::split_whitespace because we don't wanna filter-out the whitespace tokens
            token.split(' ')
        })
        .map(|token| match token {
            TICKS => DocToken::Ticks,
            other => DocToken::Word(other),
        })
}

// TODO: remove once str::split_inclusive is stable
fn split_inclusive<'a, 'b: 'a>(input: &'a str, delim: &'b str) -> impl Iterator<Item = &'a str> {
    let mut tokens = vec![];
    let mut prev = 0;

    for (idx, matches) in input.match_indices(delim) {
        if prev != idx {
            tokens.push(&input[prev..idx]);
        }

        prev = idx + matches.len();

        tokens.push(matches);
    }

    if prev < input.len() {
        tokens.push(&input[prev..]);
    }

    tokens.into_iter()
}

#[cfg(test)]
mod tests {
    use super::{split_inclusive, Attr};
    use std::fmt;

    #[cfg(test)]
    impl fmt::Debug for Attr {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Attr::Forward(..) => f.write_str("Attr::Forward"),
                Attr::DocComment(_, body) => write!(f, "Attr::DocComment({:?})", body),
                Attr::DiagramStart(..) => f.write_str("Attr::DiagramStart"),
                Attr::DiagramEntry(_, body) => write!(f, "Attr::DiagramEntry({:?})", body),
                Attr::DiagramEnd(..) => f.write_str("Attr::DiagramEnd"),
            }
        }
    }

    #[cfg(test)]
    impl Eq for Attr {}

    #[cfg(test)]
    impl PartialEq for Attr {
        fn eq(&self, other: &Self) -> bool {
            use std::mem::discriminant;
            use Attr::*;
            match (self, other) {
                (DocComment(_, a), DocComment(_, b)) => a == b,
                (DiagramEntry(_, a), DiagramEntry(_, b)) => a == b,
                (a, b) => discriminant(a) == discriminant(b),
            }
        }
    }

    #[test]
    fn temp_split_inclusive() {
        let src = "```";
        let out: Vec<_> = split_inclusive(src, "```").collect();
        assert_eq!(&out, &["```",]);

        let src = "```abcd```";
        let out: Vec<_> = split_inclusive(src, "```").collect();
        assert_eq!(&out, &["```", "abcd", "```"]);

        let src = "left```abcd```right";
        let out: Vec<_> = split_inclusive(src, "```").collect();
        assert_eq!(&out, &["left", "```", "abcd", "```", "right",]);
    }

    mod split_attr_body_tests {
        use super::super::*;

        use proc_macro2::Ident;
        use proc_macro2::Span;

        use pretty_assertions::assert_eq;

        fn i() -> Ident {
            Ident::new("fake", Span::call_site())
        }

        struct TestCase<'a> {
            ident: Ident,
            location: Location,
            input: &'a str,
            expect_location: Location,
            expect_attrs: Vec<Attr>,
        }

        fn check(case: TestCase) {
            let mut loc = case.location;
            let attrs = split_attr_body(&case.ident, case.input, &mut loc);
            assert_eq!(loc, case.expect_location);
            assert_eq!(attrs, case.expect_attrs);
        }

        #[test]
        fn one_line_one_diagram() {
            let case = TestCase {
                ident: i(),
                location: Location::OutsideDiagram,
                input: "```mermaid abcd```",
                expect_location: Location::OutsideDiagram,
                expect_attrs: vec![
                    Attr::DiagramStart(i()),
                    Attr::DiagramEntry(i(), "abcd".into()),
                    Attr::DiagramEnd(i()),
                ],
            };

            check(case)
        }

        #[test]
        fn one_line_multiple_diagrams() {
            let case = TestCase {
                ident: i(),
                location: Location::OutsideDiagram,
                input: "```mermaid abcd``` ```mermaid efgh``` ```mermaid ijkl```",
                expect_location: Location::OutsideDiagram,
                expect_attrs: vec![
                    Attr::DiagramStart(i()),
                    Attr::DiagramEntry(i(), "abcd".into()),
                    Attr::DiagramEnd(i()),
                    Attr::DocComment(i(), " ".into()),
                    Attr::DiagramStart(i()),
                    Attr::DiagramEntry(i(), "efgh".into()),
                    Attr::DiagramEnd(i()),
                    Attr::DocComment(i(), " ".into()),
                    Attr::DiagramStart(i()),
                    Attr::DiagramEntry(i(), "ijkl".into()),
                    Attr::DiagramEnd(i()),
                ],
            };

            check(case)
        }

        #[test]
        fn other_snippet() {
            let case = TestCase {
                ident: i(),
                location: Location::OutsideDiagram,
                input: "```rust panic!()```",
                expect_location: Location::OutsideDiagram,
                expect_attrs: vec![Attr::DocComment(i(), "``` rust panic!() ```".into())],
            };

            check(case)
        }

        #[test]
        fn carry_over() {
            let case = TestCase {
                ident: i(),
                location: Location::OutsideDiagram,
                input: "left```mermaid abcd```right",
                expect_location: Location::OutsideDiagram,
                expect_attrs: vec![
                    Attr::DocComment(i(), "left".into()),
                    Attr::DiagramStart(i()),
                    Attr::DiagramEntry(i(), "abcd".into()),
                    Attr::DiagramEnd(i()),
                    Attr::DocComment(i(), "right".into()),
                ],
            };

            check(case)
        }

        #[test]
        fn multiline_termination() {
            let case = TestCase {
                ident: i(),
                location: Location::InsideDiagram,
                input: "abcd```",
                expect_location: Location::OutsideDiagram,
                expect_attrs: vec![
                    Attr::DiagramEntry(i(), "abcd".into()),
                    Attr::DiagramEnd(i()),
                ],
            };

            check(case)
        }

        #[test]
        fn multiline_termination_single_token() {
            let case = TestCase {
                ident: i(),
                location: Location::InsideDiagram,
                input: "```",
                expect_location: Location::OutsideDiagram,
                expect_attrs: vec![Attr::DiagramEnd(i())],
            };

            check(case)
        }

        #[test]
        fn multiline_termination_carry() {
            let case = TestCase {
                ident: i(),
                location: Location::InsideDiagram,
                input: "abcd```right",
                expect_location: Location::OutsideDiagram,
                expect_attrs: vec![
                    Attr::DiagramEntry(i(), "abcd".into()),
                    Attr::DiagramEnd(i()),
                    Attr::DocComment(i(), "right".into()),
                ],
            };

            check(case)
        }
    }
}
