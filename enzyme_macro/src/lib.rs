extern crate proc_macro;

use heck::CamelCase;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Ident, LitStr, Result, Token, Type, punctuated::Punctuated};

#[derive(Debug)]
struct Route {
    pub context: Ident,
    pub handler: Ident,
    pub segments: Vec<Segment>,
    pub static_segment_positions: Vec<usize>,
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
    pub ty: Type,
}

impl Parse for Route {
    fn parse(input: ParseStream) -> Result<Self> {
        let _: Token![/] = input.parse()?;

        let mut count = 0;
        let mut static_segment_positions = vec![];

        let segments = {
            let mut segments = vec![];
            while !input.is_empty() && !input.peek(Token![=]) && !input.peek2(Token![>]) {
                let lookahead = input.lookahead1();
                if lookahead.peek(LitStr) {
                    segments.push(input.parse().map(Segment::Static)?);
                    static_segment_positions.push(count);
                    count += 1;
                } else if lookahead.peek(Ident) {
                    segments.push(input.parse().map(Segment::Dynamic)?);
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

        let context: Ident = input.parse()?;

        let _ = input.parse::<Token![=]>()?;
        let _ = input.parse::<Token![>]>()?;

        let handler: Ident = input.parse()?;

        Ok(Self { 
            context,
            handler,
            segments,
            static_segment_positions,
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
        let _: Token![:] = input.parse()?;
        let ty = input.parse()?;

        Ok(Self { field_name, ty })
    }
}

impl Route {
    fn static_segments(&self) -> proc_macro2::TokenStream {
        let mut segments = vec![];
        let mut positions = vec![];
        self.segments.iter().for_each(|segment| {
            if let Segment::Static(static_segment) = segment {
                let content = &static_segment.content;
                segments.push(quote!(#content));
            }
        });
        self.static_segment_positions.iter().for_each(|pos| {
            positions.push(quote!(#pos));
        });

        quote! {
            #(
                static_segments.push(enzyme::router::StaticSegment {
                    value: #segments,
                    position: #positions,
                });
            )*
        }
    }
}
                

#[proc_macro]
pub fn route(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as Route);

    let push_statements = input.static_segments();
    let route = input.handler;
    let context = input.context;

    let output = quote! {
        || -> enzyme::router::Route {
            let mut static_segments = vec![];

            #push_statements

            enzyme::router::Route {
                static_segments,
                handler: Box::new(enzyme::endpoint::Endpoint::new(#route, #context)),
            }
        }
    };
    
    output.into()
}
