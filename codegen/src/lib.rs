extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{
    parse_macro_input, Ident, LitInt, LitStr, Result, Token,
};

trait LitIntExt {
    fn from_usize(int: usize) -> LitInt {
        LitInt::new(&format!("{}", int), Span::call_site())
    }
}

impl LitIntExt for LitInt {}

#[derive(Debug)]
struct Route {
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
            while !input.is_empty() {
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

        Ok(Self {
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
    fn static_segments(&self) -> proc_macro2::TokenStream {
        let mut static_segments = vec![];

        self.segments.iter().for_each(|segment| {
            match segment {
                Segment::Static(static_segment) => {
                    let content = &static_segment.content;
                    static_segments.push(quote!(#content));
                }
                _ => {}
            }
        });

        let static_positions = &self.static_segment_positions;

        let static_segment_inserts = quote! {
            #(
                static_segments.push(StaticSegment {
                    value: #static_segments,
                    position: #static_positions,
                });
            )*
        };

        quote! {
            {
                let mut static_segments = vec![];
                #static_segment_inserts
                static_segments
            }
        }
    }

    fn dynamic_segments(&self) -> proc_macro2::TokenStream {
        let mut dynamic_segment_names = vec![];

        self.segments.iter().for_each(|segment| {
            match segment {
                Segment::Dynamic(dynamic_segment) => {
                    let name = &dynamic_segment.field_name.to_string();
                    dynamic_segment_names.push(quote!(#name));
                }
                _ => {}
            }
        });

        let dynamic_positions = &self.dynamic_segment_positions;

        let dynamic_segment_inserts = quote! {
            #(
                dynamic_segments.push(DynamicSegment {
                    name: #dynamic_segment_names,
                    position: #dynamic_positions,
                });
            )*
        };

        quote! {
            {
                let mut dynamic_segments = vec![];
                #dynamic_segment_inserts
                dynamic_segments
            }
        }
    }
}

#[proc_macro]
pub fn route(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as Route);

    let dynamic_segments = input.dynamic_segments();
    let static_segments = input.static_segments();

    let output = quote! {
        Route {
            static_segments: #static_segments,
            dynamic_segments: #dynamic_segments,
            handler: None,
        }
    };

    output.into()
}
