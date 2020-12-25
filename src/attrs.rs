use itertools::{Either, Itertools};
use proc_macro2::TokenStream;
use proc_macro_error::{abort, emit_call_site_warning};
use quote::quote;
use std::{cell::Cell, iter};
use syn::{Attribute, Ident, MetaNameValue};

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

impl quote::ToTokens for Attrs {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut attrs = self.0.iter().peekable();
        while let Some(attr) = attrs.next() {
            match attr {
                Attr::Forward(attr) => attr.to_tokens(tokens),
                Attr::DocComment(_, comment) => tokens.extend(quote! {
                    #[doc = #comment]
                }),
                Attr::DiagramStart(_) => {
                    let preabmle = iter::once(r#"<div class="mermaid">"#);
                    let postamble = iter::once("</div>");

                    let diagram = attrs
                        .by_ref()
                        .take_while(|x| !x.is_diagram_end())
                        .map(Attr::expect_diagram_entry_text);

                    let body = preabmle.chain(diagram).chain(postamble).join("\n");

                    tokens.extend(generate_diagram_rustdoc(&body));
                }
                // If that happens, then the parsing stage is faulty: doc comments outside of
                // in between Start and End tokens are to be emitted as Attr::Forward
                Attr::DiagramEntry(_, body) => {
                    emit_call_site_warning!("encountered an unexpected attribute that's going to be ignored, this is a bug! ({})", body);
                }
                Attr::DiagramEnd(_) => (),
            }
        }
    }
}

fn generate_diagram_rustdoc(body: &str) -> TokenStream {
    quote! {
        #[doc = r#"<script src="https://cdn.jsdelivr.net/npm/mermaid/dist/mermaid.min.js"></script>"#]
        #[doc = r#"<script>window.mermaid == null && mermaid.initialize({startOnLoad:true});</script>"#]
        #[doc = #body]
    }
}

impl From<Vec<Attribute>> for Attrs {
    fn from(attrs: Vec<Attribute>) -> Self {
        let mut out = Attrs::default();
        out.push_attrs(attrs);
        out
    }
}

impl Attrs {
    pub fn push_attrs(&mut self, attrs: Vec<Attribute>) {
        use syn::Lit::*;
        use syn::Meta::*;

        // Iterator state
        let is_inside_diagram = Cell::new(false);

        let attrs = attrs.into_iter().flat_map(|attr| match attr.parse_meta() {
            Ok(NameValue(MetaNameValue {
                lit: Str(s), path, ..
            })) if path.is_ident("doc") => {
                let ident = path.get_ident().unwrap();
                Either::Left(split_attr_body(ident, &s.value(), &is_inside_diagram))
            }
            _ => Either::Right(iter::once(Attr::Forward(attr))),
        });

        let mut diagram_start_ident = None;

        let attrs = attrs.inspect(|attr| match attr {
            Attr::DiagramStart(ident) => diagram_start_ident = Some(ident.clone()),
            Attr::DiagramEnd(_) => diagram_start_ident = None,
            _ => (),
        });

        self.0.extend(attrs);

        if let Some(ident) = diagram_start_ident.as_ref() {
            abort!(ident, "diagram code block is not terminated");
        }
    }
}


// This implementation cannot handle nested markdown code snippets, 
// though that shouldn't be an issue since Mermaid doesn't support markdown, so such input is highly unlikely.
//
// I don't like this method, but after rewriting it 5 times, 
// I think I'll just keep it as it is until I get some will to tackle it again.
fn split_attr_body(
    ident: &Ident,
    input: &str,
    is_inside: &Cell<bool>,
) -> impl Iterator<Item = Attr> {
    const TICKS: &str = "```";
    const MERMAID: &str = "mermaid";

    let mut attrs = vec![];
    let mut buffer: Vec<&str> = vec![];
    let mut prev: Option<&str> = None;

    // It's not str::split_whitespace because we wanna preserve empty entries
    let tokens = split_inclusive(input, TICKS);

    // Special case: empty strings outside the diagram span should be still generated
    if tokens.is_empty() && !is_inside.get() {
        attrs.push(Attr::DocComment(ident.clone(), buffer.drain(..).join(" ")));
    }

    for token in tokens {
        if token == TICKS {
            if is_inside.get() {
                is_inside.set(false);

                // disallow empty lines inside the diagram
                let s = buffer.drain(..).filter(|s| !s.trim().is_empty()).join(" ");
                if !s.is_empty() {
                    attrs.push(Attr::DiagramEntry(ident.clone(), s));
                }

                attrs.push(Attr::DiagramEnd(ident.clone()))
            } else {
                prev.replace(token);
            }
        } else if token.starts_with(MERMAID) && prev == Some(&TICKS) {
            prev = None;
            if !is_inside.get() {
                is_inside.set(true);

                if !buffer.is_empty() {
                    attrs.push(Attr::DocComment(ident.clone(), buffer.drain(..).join(" ")));
                }

                attrs.push(Attr::DiagramStart(ident.clone()));

                // Extract whatever is in the same token after "mermaid", removing whitespaces
                let postfix = token.trim_start_matches(MERMAID).trim();
                if !postfix.is_empty() {
                    buffer.push(postfix);
                }
            }
        } else {
            buffer.extend(prev.into_iter());
            buffer.push(token);
        }
    }

    if !prev.is_none() || !buffer.is_empty() {
        let leftover = buffer.drain(..).chain(prev.into_iter()).join("");
        let attr = if is_inside.get() {
            Attr::DiagramEntry(ident.clone(), leftover)
        } else {
            Attr::DocComment(ident.clone(), leftover)
        };
        attrs.push(attr);
    }

    attrs.into_iter()
}

// TODO: remove once str::split_inclusive is stable
fn split_inclusive<'a, 'b: 'a>(input: &'a str, delim: &'b str) -> Vec<&'a str> {
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

    tokens
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
        const ERR_MSG: &str =
            "unexpected attribute inside a diagram definition: only #[doc] is allowed";
        match self {
            Attr::DiagramEntry(_, body) => body.as_str(),
            _ => {
                if let Some(ident) = self.as_ident() {
                    abort!(ident, ERR_MSG)
                } else {
                    panic!(ERR_MSG)
                }
            }
        }
    }
}



#[cfg(test)]
mod tests {
    use super::{Attr, split_inclusive};
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
                (a, b) => discriminant(a) == discriminant(b)
            }
        }
    }

    #[test]
    fn temporaty_split_inclusive() {
        let src = "```";
        let out: Vec<_> = split_inclusive(src, "```");
        assert_eq!(&out, &[
            "```",
        ]);

        let src = "```abcd```";
        let out: Vec<_> = split_inclusive(src, "```");
        assert_eq!(&out, &[
            "```",
            "abcd",
            "```"
        ]);

        let src = "left```abcd```right";
        let out: Vec<_> = split_inclusive(src, "```");
        assert_eq!(&out, &[
            "left",
            "```",
            "abcd",
            "```",
            "right",
        ]);
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
            is_inside: bool,
            input: &'a str,
            expect_is_inside: bool,
            expect_attrs: Vec<Attr>,
        }
        
        fn check(case: TestCase) {
            let is_inside = Cell::new(case.is_inside);
            let attrs: Vec<_> = split_attr_body(&case.ident, case.input, &is_inside).collect();
            assert_eq!(is_inside.get(), case.expect_is_inside);
            assert_eq!(attrs, case.expect_attrs);
        }
    
        #[test]
        fn one_line_one_diagram() {
            let case = TestCase {
                ident: i(),
                is_inside: false,
                input: "```mermaid abcd```",
                expect_is_inside: false,
                expect_attrs: vec![
                    Attr::DiagramStart(i()),
                    Attr::DiagramEntry(i(), "abcd".into()),
                    Attr::DiagramEnd(i()),
                ]
            };
    
            check(case)
        }
        
        #[test]
        fn one_line_multiple_diagrams() {
            let case = TestCase {
                ident: i(),
                is_inside: false,
                input: "```mermaid abcd``` ```mermaid efgh``` ```mermaid ijkl```",
                expect_is_inside: false,
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
                ]
            };
    
            check(case)
        }
    
        #[test]
        fn other_snippet() {
            let case = TestCase {
                ident: i(),
                is_inside: false,
                input: "```rust panic!()```",
                expect_is_inside: false,
                expect_attrs: vec![
                    Attr::DocComment(i(), "```rust panic!()```".into()),
                ]
            };
    
            check(case)
        }
    
        #[test]
        fn carry_over() {
            let case = TestCase {
                ident: i(),
                is_inside: false,
                input: "left```mermaid abcd```right",
                expect_is_inside: false,
                expect_attrs: vec![
                    Attr::DocComment(i(), "left".into()),
                    Attr::DiagramStart(i()),
                    Attr::DiagramEntry(i(), "abcd".into()),
                    Attr::DiagramEnd(i()),
                    Attr::DocComment(i(), "right".into()),
                ]
            };
    
            check(case)
        }

        #[test]
        fn multiline_termination() {
            let case = TestCase {
                ident: i(),
                is_inside: true,
                input: "abcd```",
                expect_is_inside: false,
                expect_attrs: vec![
                    Attr::DiagramEntry(i(), "abcd".into()),
                    Attr::DiagramEnd(i()),
                ]
            };

            check(case)
        }


        #[test]
        fn multiline_termination_single_token() {
            let case = TestCase {
                ident: i(),
                is_inside: true,
                input: "```",
                expect_is_inside: false,
                expect_attrs: vec![
                    Attr::DiagramEnd(i()),
                ]
            };

            check(case)
        }

        #[test]
        fn multiline_termination_carry() {
            let case = TestCase {
                ident: i(),
                is_inside: true,
                input: "abcd```right",
                expect_is_inside: false,
                expect_attrs: vec![
                    Attr::DiagramEntry(i(), "abcd".into()),
                    Attr::DiagramEnd(i()),
                    Attr::DocComment(i(), "right".into()),
                ]
            };
    
            check(case)
        }
    }
}