#![forbid(unsafe_code)]

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as Tokens};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{
    Expr, Ident, LitStr, Pat, Path, Result, Token, braced, parenthesized, parse_macro_input,
};

/// Builds Ink trees through the public builder API.
///
/// ```text
/// Input   := Node+
/// Node    := Element | LitStr | "{" Expr "}"
///          | "if" Expr "{" Node* "}" ("else" "if" ... | "else" "{" Node* "}")?
///          | "for" Pat "in" Expr "{" Node* "}"
/// Element := Path ("(" Args? ")")? ("{" Node* "}")?
/// Args    := Arg ("," Arg)* ","?
/// Arg     := Ident ":" Expr | Expr
/// ```
///
/// Positional arguments precede named arguments and call `Path::new(...)`.
/// Without positional arguments, construction uses `<Path>::default()`.
/// A bare identifier is a positional expression, not a flag: use `bold: true`.
/// Named arguments call builder methods in order, except `key`, which is
/// applied after all children. Paths, expressions and method names retain
/// their spans for compiler diagnostics.
///
/// Each element threads a builder through statements. Children go to that
/// builder's `child` method; `if` branches and `for` iterations thread the
/// parent, so their children need not share a type. Conditions and iterables
/// parse without eager braces. The builder binding uses a mixed-site span
/// to keep it separate from user bindings. The outer block imports
/// `::ink_rs::Styled as _` with `#[allow(unused_imports)]` for style methods.
///
/// An expression child passes through unchanged. String literals containing
/// `{` expand to `::std::format!(literal)`, with identifier captures and `{{`
/// escapes; other strings pass through unchanged. Put other expressions in
/// braces, such as `{format!("{} total", passed + failed)}`. Strings under a
/// `View` fail the builder's `Into<Element>` bound: text belongs inside `Text`.
/// Components use expressions such as `{component(f, props)}`.
///
/// One element or expression root returns its value, retaining its builder
/// type unless `key` returns an `Element`. Multiple roots use
/// `::ink_rs::Element::fragment([::ink_rs::Element::from(root), ...])`.
/// String, `if` and `for` roots must be wrapped in an element or a `{}`
/// expression. The macro adds no capabilities beyond the builder API.
///
/// ```ignore
/// rsx! {
///     View(flex_direction: FlexDirection::Column, margin_top: 1) {
///         Text(bold: true) { "Test Suites:" }
///         if failed > 0 {
///             Text(color: Color::Red) { "{failed} failed" }
///         }
///         for name in names {
///             Text(key: name) { "{name}" }
///         }
///     }
/// }
/// ```
#[proc_macro]
pub fn rsx(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Input);
    let roots = input.0.iter().map(Node::value);
    let value = if input.0.len() == 1 {
        quote! { #(#roots)* }
    } else {
        quote! { ::ink_rs::Element::fragment([#(::ink_rs::Element::from(#roots)),*]) }
    };
    quote! {{
        #[allow(unused_imports)]
        use ::ink_rs::Styled as _;
        #value
    }}
    .into()
}

struct Input(Vec<Node>);

impl Parse for Input {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            return Err(input.error("expected an element or a {} expression"));
        }
        let mut nodes = Vec::new();
        while !input.is_empty() {
            if input.peek(LitStr) || input.peek(Token![if]) || input.peek(Token![for]) {
                return Err(input.error("wrap this root in an element or a {} expression"));
            }
            nodes.push(input.parse()?);
        }
        Ok(Self(nodes))
    }
}

enum Node {
    Element(Box<Element>),
    String(LitStr),
    Expr(Box<Expr>),
    If {
        condition: Box<Expr>,
        then_nodes: Vec<Node>,
        else_nodes: Vec<Node>,
    },
    For {
        pattern: Box<Pat>,
        iterable: Box<Expr>,
        nodes: Vec<Node>,
    },
}

struct Element {
    path: Path,
    positional: Vec<Expr>,
    named: Vec<(Ident, Expr)>,
    children: Vec<Node>,
}

fn nodes_in_braces(input: ParseStream) -> Result<Vec<Node>> {
    let body;
    braced!(body in input);
    let mut nodes = Vec::new();
    while !body.is_empty() {
        nodes.push(body.parse()?);
    }
    Ok(nodes)
}

impl Parse for Node {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(LitStr) {
            Ok(Self::String(input.parse()?))
        } else if input.peek(syn::token::Brace) {
            let body;
            braced!(body in input);
            let expr = body.parse()?;
            if !body.is_empty() {
                return Err(body.error("expected one expression"));
            }
            Ok(Self::Expr(expr))
        } else if input.peek(Token![if]) {
            input.parse::<Token![if]>()?;
            let condition = Box::new(input.call(Expr::parse_without_eager_brace)?);
            let then_nodes = nodes_in_braces(input)?;
            let else_nodes = if input.peek(Token![else]) {
                input.parse::<Token![else]>()?;
                if input.peek(Token![if]) {
                    vec![input.parse()?]
                } else {
                    nodes_in_braces(input)?
                }
            } else {
                Vec::new()
            };
            Ok(Self::If {
                condition,
                then_nodes,
                else_nodes,
            })
        } else if input.peek(Token![for]) {
            input.parse::<Token![for]>()?;
            let pattern = Box::new(input.call(Pat::parse_single)?);
            input.parse::<Token![in]>()?;
            let iterable = Box::new(input.call(Expr::parse_without_eager_brace)?);
            let nodes = nodes_in_braces(input)?;
            Ok(Self::For {
                pattern,
                iterable,
                nodes,
            })
        } else {
            Ok(Self::Element(Box::new(input.parse()?)))
        }
    }
}

impl Parse for Element {
    fn parse(input: ParseStream) -> Result<Self> {
        let path = input.parse()?;
        let mut positional = Vec::new();
        let mut named = Vec::new();
        if input.peek(syn::token::Paren) {
            let args;
            parenthesized!(args in input);
            while !args.is_empty() {
                if args.peek(Ident) && args.peek2(Token![:]) && !args.peek2(Token![::]) {
                    let name = args.parse()?;
                    args.parse::<Token![:]>()?;
                    named.push((name, args.parse()?));
                } else {
                    if !named.is_empty() {
                        return Err(args.error("positional arguments must precede named arguments"));
                    }
                    positional.push(args.parse()?);
                }
                if !args.is_empty() {
                    args.parse::<Token![,]>()?;
                }
            }
        }
        let children = if input.peek(syn::token::Brace) {
            nodes_in_braces(input)?
        } else {
            Vec::new()
        };
        Ok(Self {
            path,
            positional,
            named,
            children,
        })
    }
}

fn builder() -> Ident {
    Ident::new("__ink_rsx_builder", Span::mixed_site())
}

fn children(nodes: &[Node]) -> Tokens {
    let b = builder();
    let statements = nodes.iter().map(|node| match node {
        Node::If {
            condition,
            then_nodes,
            else_nodes,
        } => {
            let then_body = children(then_nodes);
            let else_body = children(else_nodes);
            quote! { let #b = if #condition { #then_body } else { #else_body }; }
        }
        Node::For {
            pattern,
            iterable,
            nodes,
        } => {
            let body = children(nodes);
            quote! {
                let #b = {
                    let mut #b = #b;
                    for #pattern in #iterable { #b = { #body }; }
                    #b
                };
            }
        }
        _ => {
            let value = node.value();
            quote! { let #b = #b.child(#value); }
        }
    });
    quote! { #(#statements)* #b }
}

impl Node {
    fn value(&self) -> Tokens {
        match self {
            Self::Element(element) => {
                let Element {
                    path,
                    positional,
                    named,
                    children: nodes,
                } = element.as_ref();
                let b = builder();
                let constructor = if positional.is_empty() {
                    quote! { <#path>::default() }
                } else {
                    quote! { <#path>::new(#(#positional),*) }
                };
                let methods = named.iter().filter(|(name, _)| name != "key");
                let methods = methods.map(|(name, expr)| quote! { let #b = #b.#name(#expr); });
                let keys = named.iter().filter(|(name, _)| name == "key");
                let keys = keys.map(|(name, expr)| quote! { .#name(#expr) });
                let body = children(nodes);
                quote! {{
                    let #b = #constructor;
                    #(#methods)*
                    let #b = { #body };
                    #b #(#keys)*
                }}
            }
            Self::String(literal) if literal.value().contains('{') => {
                quote! { ::std::format!(#literal) }
            }
            Self::String(literal) => quote! { #literal },
            Self::Expr(expr) => quote! { #expr },
            Self::If { .. } | Self::For { .. } => unreachable!("control flow is a child"),
        }
    }
}
