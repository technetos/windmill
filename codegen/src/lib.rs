extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{
    parenthesized, parse_macro_input, punctuated::Punctuated, FnArg, Generics, Ident, LitInt,
    LitStr, Pat, PatIdent, PatType, Result, Token, Type, TypePath, Visibility,
};

trait LitIntExt {
    fn from_usize(int: usize) -> LitInt {
        LitInt::new(&format!("{}", int), Span::call_site())
    }
}

impl LitIntExt for LitInt {}

trait IdentExt {
    fn prepend(&self, string: &str) -> Ident;
}

impl IdentExt for syn::Ident {
    fn prepend(&self, string: &str) -> Ident {
        Ident::new(&format!("{}{}", string, self), self.span())
    }
}

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

        self.segments.iter().for_each(|segment| match segment {
            Segment::Static(static_segment) => {
                let content = &static_segment.content;
                static_segments.push(quote!(#content));
            }
            _ => {}
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

        self.segments.iter().for_each(|segment| match segment {
            Segment::Dynamic(dynamic_segment) => {
                let name = &dynamic_segment.field_name.to_string();
                dynamic_segment_names.push(quote!(#name));
            }
            _ => {}
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

/// The `route!` macro is used to generate a [`Route`](struct.Route.html) from a path.  
/// ```
/// route!(/"path"/param/"path"/"path")
/// ```
/// Where `"path"` is a static segment that must be matched verbatim and `param` is a parameter
/// that is captured and made available through the [`.params()`](struct.Req.html#method.params)
/// method on [`Req`](struct.Req.html).  
///
/// ## Paths
///
/// The `route!` macro takes a list of string literals and idents beginning with and separated by `/`.  
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

#[derive(Debug)]
struct Service {
    ident: proc_macro2::TokenStream,
    ty: proc_macro2::TokenStream,
}

#[derive(Debug)]
struct Endpoint {
    tokens: proc_macro2::TokenStream,
}

impl Parse for Endpoint {
    fn parse(input: ParseStream) -> Result<Self> {
        let _visibility: Visibility = input.parse()?;
        let _async: Option<Token![async]> = input.parse()?;
        let _fn: Token![fn] = input.parse()?;
        let fn_name: Ident = input.parse()?;
        let _generics: Generics = input.parse()?;
        let content;
        let _paren = parenthesized!(content in input);
        let args: Punctuated<FnArg, Token![,]> = content.parse_terminated(FnArg::parse)?;

        let mut props = vec![];

        for arg in args {
            if let FnArg::Typed(PatType { pat, ty, .. }) = arg {
                if let Type::Path(TypePath { path, .. }) = *ty {
                    if let Pat::Ident(PatIdent { ident, .. }) = *pat {
                        props.push(Service {
                            ident: quote!(#ident),
                            ty: quote!(#path),
                        });
                    }
                }
            }
        }

        let mut args = vec![];
        let mut props_calls = vec![];

        for prop in &props {
            let ident = &prop.ident;
            let ty = &prop.ty;

            let props_call = quote! {
                let (req, params, #ident) =
                    <#ty as Props>::call(req, params).await?;
            };
            props_calls.push(props_call);
            args.push(quote!(#ident));
        }

        let generated_props_calls = quote! {
            #(
                #props_calls
            )*
        };

        let generated_endpoint_call = quote! {
            Ok(#fn_name(#(#args),*).await?)
        };

        let hidden_fn_name = fn_name.prepend("___");

        let endpoint_fn = quote! {
            async fn #hidden_fn_name(
                req: http_types::Request,
                params: Params
            ) -> Result<http_types::Response, Error> {
                #generated_props_calls
                #generated_endpoint_call
            }
        };

        while !input.is_empty() {
            let _ = input.step(|cursor| {
                let mut rest = *cursor;
                while let Some((_, next)) = rest.token_tree() {
                    rest = next;
                }
                Ok(((), rest))
            });
        }

        Ok(Self {
            tokens: endpoint_fn,
        })
    }
}

#[proc_macro_attribute]
pub fn endpoint(attrs: TokenStream, tokens: TokenStream) -> TokenStream {
    let tokens_clone = tokens.clone();
    let input = parse_macro_input!(tokens_clone as Endpoint);

    let endpoint_fn = input.tokens;
    let tokens2: proc_macro2::TokenStream = tokens.into();

    let output = quote! {
        #endpoint_fn
        #tokens2
    };

    output.into()
}
