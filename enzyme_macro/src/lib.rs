extern crate proc_macro;

use heck::CamelCase;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Ident, LitStr, Result, Token, Type};

#[derive(Debug)]
struct Route {
    pub segments: Vec<Segment>,
}

impl Route {
    pub fn struct_name(&self) -> Ident {
        let mut string = String::new();
        self.segments.iter().for_each(|segment| match segment {
            Segment::Static(static_segment) => {
                string += &static_segment.content.value().to_camel_case();
            }
            Segment::Dynamic(dynamic_segment) => {
                string += &dynamic_segment.field_name.to_string().to_camel_case();
            }
        });

        Ident::new(&string, Span::call_site())
    }

    pub fn fields(&self) -> Vec<proc_macro2::TokenStream> {
        let mut tokens = vec![];
        self.segments.iter().for_each(|segment| match segment {
            Segment::Dynamic(dynamic_segment) => {
                let field = &dynamic_segment.field_name;
                let field_type = &dynamic_segment.ty;

                tokens.push(quote!(#field: #field_type));
            }
            _ => {},
        });

        tokens
    }
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

        let segments = {
            let mut segments = vec![];
            while !input.is_empty() {
                let lookahead = input.lookahead1();
                if lookahead.peek(LitStr) {
                    segments.push(input.parse().map(Segment::Static)?);
                } else if lookahead.peek(Ident) {
                    segments.push(input.parse().map(Segment::Dynamic)?);
                } else if lookahead.peek(Token![/]) {
                    let _: Token![/] = input.parse()?;
                } else {
                    return Err(lookahead.error());
                }
            }
            segments
        };

        Ok(Self { segments })
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

#[proc_macro]
pub fn route(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as Route);

    let struct_name = input.struct_name();
    let fields = input.fields();

    let output = quote! {
//        |s: &str| -> Result<Route, ()> {
//            struct UsersUserIdMe {
//                user_id: i32,
//            }
//
//            impl FromStr for UsersUserIdMe {
//                fn from_str(blah) -> blah {
//                    expects "users" value "me"
//                }
//            }
//
//            UsersUserIdMe::from_str(s)?
//        }
        #[derive(Debug)]
        struct #struct_name {
            #(#fields,)*
        }
    };
    
    output.into()
}
