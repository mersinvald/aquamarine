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

        let mut diagram_start_ident = None;

        let attrs = attrs.into_iter().flat_map(|attr| match attr.parse_meta() {
            Ok(NameValue(MetaNameValue {
                lit: Str(s), path, ..
            })) if path.is_ident("doc") => {
                let ident = path.get_ident().unwrap();

                let body = s.value();
                let (pre, start, body, end, post) = parse_attr_body(&body);

                // TODO: replace with generator sometime in the future
                let mut temp = vec![];

                if start.is_some() {
                    if let Some(s) = pre {
                        temp.push(Attr::DocComment(ident.clone(), s.to_owned()));
                    }
                    temp.push(Attr::DiagramStart(ident.clone()));
                    diagram_start_ident.replace(ident.clone());
                }

                if let Some(body) = body {
                    // HACK: body that only has whitespaces and is on the same line with start or end token
                    //       should be filtered-out because otherwise it inserts an empty line
                    //       caused by the leading whitespace most people add in their doc strings
                    let skip_empty_body = start.is_some() || end.is_some();

                    if !body.trim().is_empty() || !skip_empty_body {
                        let body = body.to_owned();
                        if diagram_start_ident.is_some() {
                            temp.push(Attr::DiagramEntry(ident.clone(), body))
                        } else {
                            temp.push(Attr::Forward(attr));
                        }
                    }
                }

                if end.is_some() {
                    diagram_start_ident = None;
                    temp.push(Attr::DiagramEnd(ident.clone()));
                    if let Some(s) = post {
                        temp.push(Attr::DocComment(ident.clone(), s.to_owned()));
                    }
                }

                Either::Left(temp.into_iter())
            }
            _ => Either::Right(iter::once(Attr::Forward(attr))),
        });

        self.0.extend(attrs);

        if let Some(ident) = diagram_start_ident.as_ref() {
            abort!(ident, "diagram code block is not terminated");
        }
    }
}

type AttrBodyParts<'a> = (
    Option<&'a str>,
    Option<&'a str>,
    Option<&'a str>,
    Option<&'a str>,
    Option<&'a str>,
);

// This function should be called "things you do not to tokenize"
// TODO: make an actual tokenizer -- this garbage would break on one-liners with multiple diagrams
fn parse_attr_body(input: &str) -> AttrBodyParts {
    const ENTRY: &str = "```mermaid";
    const EXIT: &str = "```";

    // Why as_ref on ranges:
    // https://github.com/rust-lang/rust/pull/27186

    // Calculate start, end, and body spans
    let ss = input.find(ENTRY).map(|sp| sp..sp + ENTRY.len());
    let ss_ref = ss.as_ref();

    let es = {
        let offset = ss_ref.map(|x| x.end).unwrap_or(0);
        input[offset..]
            .find(EXIT)
            .map(|p| p + offset..p + offset + EXIT.len())
    };
    let es_ref = es.as_ref();

    let bs = {
        let ss_end = ss_ref.map(|ss| ss.end).unwrap_or(0);
        let es_start = es_ref.map(|es| es.start).unwrap_or(input.len());
        ss_end..es_start
    };

    // Extract the slices
    let nonempty = |x: &&str| !x.is_empty();

    let pre = ss_ref.map(|ss| &input[..ss.start]).filter(nonempty);
    let body = Some(&input[bs]);
    let post = es_ref.map(|es| &input[es.end..]).filter(nonempty);

    let end = es.map(|es| &input[es]);
    let start = ss.map(|ss| &input[ss]);

    (pre, start, body, end, post)
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
