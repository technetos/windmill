extern crate proc_macro;

use heck::{CamelCase, SnakeCase};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{
    parse_macro_input, punctuated::Punctuated, DeriveInput, Ident, LitInt, LitStr, Result, Token,
    Type,
};

trait LitIntExt {
    fn from_usize(int: usize) -> LitInt {
        LitInt::new(&format!("{}", int), Span::call_site())
    }
}

impl LitIntExt for LitInt {}

#[derive(Debug)]
struct Route {
    pub handler: Ident,
    pub segments: Vec<Segment>,
    pub static_segment_positions: Vec<LitInt>,
    pub dynamic_segment_positions: Vec<LitInt>,
}

#[derive(Debug)]
enum Segment {
    Static(StaticSegment),
    Dynamic(DynamicSegment),
}

#[derive(Debug)]
struct StaticSegment {
    pub content: LitStr,
}

#[derive(Debug)]
struct DynamicSegment {
    pub field_name: Ident,
}

impl Parse for Route {
    fn parse(input: ParseStream) -> Result<Self> {
        let _: Token![/] = input.parse()?;

        let mut count = 0;
        let mut static_segment_positions = vec![];
        let mut dynamic_segment_positions = vec![];

        let segments = {
            let mut segments = vec![];
            while !input.is_empty() && !input.peek(Token![=]) && !input.peek2(Token![>]) {
                let lookahead = input.lookahead1();
                if lookahead.peek(LitStr) {
                    segments.push(input.parse().map(Segment::Static)?);
                    static_segment_positions.push(LitInt::from_usize(count));
                    count += 1;
                } else if lookahead.peek(Ident) {
                    segments.push(input.parse().map(Segment::Dynamic)?);
                    dynamic_segment_positions.push(LitInt::from_usize(count));
                    count += 1;
                } else if lookahead.peek(Token![/]) {
                    let _: Token![/] = input.parse()?;
                } else {
                    return Err(lookahead.error());
                }
            }
            segments
        };

        let _ = input.parse::<Token![=]>()?;
        let _ = input.parse::<Token![>]>()?;

        let handler: Ident = input.parse()?;

        Ok(Self {
            handler,
            segments,
            static_segment_positions,
            dynamic_segment_positions,
        })
    }
}

impl Parse for StaticSegment {
    fn parse(input: ParseStream) -> Result<Self> {
        let content = input.parse()?;

        Ok(StaticSegment { content })
    }
}

impl Parse for DynamicSegment {
    fn parse(input: ParseStream) -> Result<Self> {
        let field_name = input.parse()?;

        Ok(Self { field_name })
    }
}

impl Route {
    fn push_statements(&self) -> proc_macro2::TokenStream {
        let mut static_segments = vec![];
        let mut dynamic_segment_names = vec![];

        self.segments.iter().for_each(|segment| {
            match segment {
                Segment::Static(static_segment) => {
                    let content = &static_segment.content;
                    static_segments.push(quote!(#content));
                }
                Segment::Dynamic(dynamic_segment) => {
                    let name = &dynamic_segment.field_name.to_string();
                    dynamic_segment_names.push(quote!(#name));
                }
            }
        });

        let static_positions = &self.static_segment_positions;
        let dynamic_positions = &self.dynamic_segment_positions;

        let static_segment_inserts = quote! {
            #(
                static_segments.push(enzyme::router::StaticSegment {
                    value: #static_segments,
                    position: #static_positions,
                });
            )*
        };

        let dynamic_segment_inserts = quote! {
            #(
                dynamic_segments.push(enzyme::router::DynamicSegment {
                    name: #dynamic_segment_names,
                    position: #dynamic_positions,
                });
            )*
        };

        quote! {
            #static_segment_inserts
            #dynamic_segment_inserts
        }
    }
}

#[proc_macro]
pub fn route(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as Route);

    let push_statements = input.push_statements();
    let route = input.handler;

    let output = quote! {
        || -> enzyme::router::Route {
            let mut static_segments = vec![];
            let mut dynamic_segments = vec![];

            #push_statements

            enzyme::router::Route {
                static_segments,
                dynamic_segments,
                handler: Box::new(enzyme::endpoint::Endpoint::new(#route)),
            }
        }
    };

    output.into()
}

trait IdentExt {
    fn as_snake_case(&self) -> Ident;
}

impl IdentExt for Ident {
    fn as_snake_case(&self) -> Ident {
        Ident::new(&self.to_string().to_snake_case(), self.span())
    }
}

#[proc_macro_derive(Context)]
pub fn context_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let context_func = &input.ident.as_snake_case();

    let tokens = quote! {
        impl enzyme::context::Context for #name {
            fn from_parts(
                parts: Parts
            ) -> std::pin::Pin<Box<futures::future::Future<Output = WebResult<Self>> + Send>>
            {
                use futures::future::FutureExt;
                async move { #context_func(parts).await }.boxed()
            }
        }
    };

    tokens.into()
}
