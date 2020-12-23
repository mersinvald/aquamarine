use proc_macro2::TokenStream;
use proc_macro_error::{emit_call_site_warning, abort};
use itertools::{Itertools, Either};
use syn::{Attribute, MetaNameValue, Ident};
use quote::quote;
use std::{iter, fmt};

#[derive(Clone, Debug)]
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
            Attr::Forward(..) => f.write_str("Forward"),
            Attr::DocComment(..) => f.write_str("DocComment"),
            Attr::DiagramStart(..) => f.write_str("DiagramStart"),
            Attr::DiagramEntry(..) => f.write_str("DiagramEntry"),
            Attr::DiagramEnd(..) => f.write_str("DiagramEnd"),
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

                    let diagram = attrs.by_ref().take_while(|x| !x.is_diagram_end())
                        .map(Attr::expect_diagram_entry_text);

                    let body = preabmle.chain(diagram).chain(postamble).join("\n");

                    tokens.extend(generate_diagram_rustdoc(&body));
                },
                // If that happens, then the parsing stage is faulty: doc comments outside of
                // in between Start and End tokens are to be emitted as Attr::Forward
                Attr::DiagramEntry(_, body) => {
                    emit_call_site_warning!("encountered an unexpected attribute that's going to be ignored, this is a bug! ({})", body);
                },
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

pub fn convert_attrs(attrs: Vec<Attribute>) -> syn::Result<Attrs> {
    use syn::Lit::*;
    use syn::Meta::*;

    let mut diagram_start_ident = None;

    let attrs = attrs.into_iter().flat_map(|attr| {
        match attr.parse_meta() {
            Ok(NameValue(MetaNameValue { lit: Str(s), path, .. })) if path.is_ident("doc") => {
                let ident = path.get_ident().unwrap();

                let body = s.value();
                let (pre, start, body, end, post) = parse_attr_body(&body);

                let mut temp = vec![];

                if start.is_some() {
                    diagram_start_ident = Some(ident.clone());
                    pre.map(|s| temp.push(Attr::DocComment(ident.clone(), s.to_owned())));
                    temp.push(Attr::DiagramStart(ident.clone()));
                }

                if let Some(body) = body {
                    let body = body.to_owned();
                    if diagram_start_ident.is_some() {
                        temp.push(Attr::DiagramEntry(ident.clone(), body))
                    } else {
                        temp.push(Attr::Forward(attr));
                    }
                }

                if end.is_some() {
                    diagram_start_ident = None;
                    temp.push(Attr::DiagramEnd(ident.clone()));
                    post.map(|s| temp.push(Attr::DocComment(ident.clone(), s.to_owned())));
                }

                Either::Left(temp.into_iter())
            },
            _ => Either::Right(iter::once(Attr::Forward(attr)))
        }
    }).collect();

    if let Some(ident) = diagram_start_ident {
        abort!(ident, "diagram code block is not terminated");
    }

    Ok(Attrs(attrs))
}

fn parse_attr_body(input: &str) -> (Option<&str>, Option<&str>, Option<&str>, Option<&str>, Option<&str>) {
    const ENTRY: &str = "```mermaid";
    const EXIT: &str = "```";

    // TODO spans

    let sp = input.find(ENTRY);
    let ep = match sp {
        Some(spos) => input[spos + ENTRY.len()..].find(EXIT).map(|p| p + spos),
        None => input.find(EXIT),
    };
    let bp = sp.map(|x| x + ENTRY.len()).unwrap_or(0);

    let nonempty = |x: &&str| !x.is_empty();

    let pre = Some(&input[..sp.map(|x| x).unwrap_or(0)]).filter(nonempty);
    let start = sp.map(|pos| &input[pos..pos+ENTRY.len()]);
    
    let body = Some(&input[bp..ep.unwrap_or(input.len())])
        .map(str::trim_end)
        .filter(nonempty);

    let end = ep.map(|pos| &input[pos..pos+EXIT.len()]);

    let pp = ep.map(|x| x + EXIT.len()).unwrap_or(input.len());
    let post = Some(&input[pp..]).filter(nonempty);


    dbg!((pre, start, body, end, post))
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
        match self {
            Attr::DiagramEnd(_) => true,
            _ => false,
        }
    }

    pub fn expect_diagram_entry_text(&self) -> &str {
        const ERR_MSG: &str = "unexpected attribute inside a diagram definition: only #[doc] is allowed";
        match self {
            Attr::DiagramEntry(_, body) => body.as_str(),
            _ => if let Some(ident) = self.as_ident() {
                abort!(ident, ERR_MSG)
            } else {
                panic!(ERR_MSG)
            }
        }
    }
}