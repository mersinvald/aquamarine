use itertools::{Either, Itertools};
use proc_macro2::TokenStream;
use proc_macro_error::{abort, emit_call_site_warning};
use quote::quote;
use std::{fmt, iter};
use syn::{Attribute, Ident, MetaNameValue};

#[derive(Clone, Debug, Default)]
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

impl fmt::Debug for Attr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Attr::Forward(..) => f.write_str("Attr::Forward"),
            Attr::DocComment(..) => f.write_str("Attr::DocComment"),
            Attr::DiagramStart(..) => f.write_str("Attr::DiagramStart"),
            Attr::DiagramEntry(..) => f.write_str("Attr::DiagramEntry"),
            Attr::DiagramEnd(..) => f.write_str("Attr::DiagramEnd"),
        }
    }
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

        let mut is_inside_diagram = false;

        let attrs = attrs.into_iter().flat_map(|attr| match attr.parse_meta() {
            Ok(NameValue(MetaNameValue {
                lit: Str(s), path, ..
            })) if path.is_ident("doc") => {
                let ident = path.get_ident().unwrap();
                Either::Left(split_attr_body(ident, &s.value(), &mut is_inside_diagram))
            }
            _ => Either::Right(iter::once(Attr::Forward(attr))),
        });

        let mut diagram_start_ident = None;

        let attrs = attrs
            .inspect(|attr| match attr { 
                Attr::DiagramStart(ident) => diagram_start_ident = Some(ident.clone()),
                Attr::DiagramEnd(_) => diagram_start_ident = None,
                _ => ()
            });

        self.0.extend(attrs);

        if let Some(ident) = diagram_start_ident.as_ref() {
            abort!(ident, "diagram code block is not terminated");
        }
    }
}

fn split_attr_body(ident: &Ident, input: &str, is_inside_diagram: &mut bool) -> impl Iterator<Item=Attr> {
    const TICKS: &str = "```";
    const MMD: &str = "```mermaid";

    let mut is_inside = *is_inside_diagram;

    let mut attrs = vec![];
    let mut stack: Vec<&str> = vec![];

    let tokens = input.split(" ");
    for token in tokens {
        match token {
            TICKS => if is_inside {
                is_inside = false;
            
                // disallow empty lines inside the diagram
                let s = stack.drain(..).filter(|s| !s.trim().is_empty()).join(" ");
                if !s.is_empty() {
                    attrs.push(Attr::DiagramEntry(ident.clone(), s));
                }

                attrs.push(Attr::DiagramEnd(ident.clone()))
            }
            MMD => if !is_inside {
                is_inside = true;

                if !stack.is_empty() {
                    attrs.push(Attr::DocComment(ident.clone(), stack.drain(..).join(" ")));
                }
                
                attrs.push(Attr::DiagramStart(ident.clone()));
            }
            other => stack.push(other),
        }
    }

    if !stack.is_empty() {
        let leftover = stack.drain(..).join(" ");
        let attr = if is_inside {
            Attr::DiagramEntry(ident.clone(), leftover)
        } else {
            Attr::DocComment(ident.clone(), leftover)
        };
        attrs.push(attr);
    }

    *is_inside_diagram = is_inside;

    attrs.into_iter()
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
