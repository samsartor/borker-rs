extern crate proc_macro;

use proc_macro::{TokenStream};
use proc_macro2::{TokenStream as TokenStream2, Span};
use quote::{ToTokens, quote};

use syn::parse::{Parse, ParseStream, Result as SynResult};
use syn::{
    Ident,
    Type,
    parenthesized,
    braced,
    parse_macro_input,
    parse_quote,
    punctuated::Punctuated,
    token
};

#[proc_macro]
pub fn classy(stream: TokenStream) -> TokenStream {
    let items = parse_macro_input!(stream as ClassInput);
    items.bindgen().into()
}

mod kw {
    use syn::custom_keyword;

    custom_keyword!(class);
    custom_keyword!(extends);
}

#[derive(Debug)]
struct ClassArg {
    name: Ident,
    delin: token::Colon,
    ty: Type,
}

impl Parse for ClassArg {
    fn parse(input: ParseStream) -> SynResult<Self> {
        Ok(ClassArg {
            name: input.parse()?,
            delin: input.parse()?,
            ty: input.parse()?,
        })
    }
}

impl ClassArg {
    fn getter_param(&self) -> TokenStream2 {
        let name = &self.name;
        let ty = &self.ty;
        quote! { #name: #ty }
    }

    fn setter_param(&self) -> TokenStream2 {
        let name = &self.name;
        let ty = &self.ty;
        let string: Type = parse_quote! { String };
        let slice_str: Type = parse_quote! { &str };
        let vec_u8: Type = parse_quote! { Vec<u8> };
        let slice_u8: Type = parse_quote! { &[u8] };
        if *ty == string {
            quote! { #name: #slice_str }
        } else if *ty == vec_u8 {
            quote! { #name: #slice_u8 }
        } else {
            quote! { #name: #ty }
        }
    }

    fn js_type(&self) -> TokenStream2 {
        if self.ty == parse_quote! { String } {
            quote! { string }
        } else if self.ty == parse_quote! { Vec<u8> } {
            quote! { Uint8Array }
        } else {
            quote! { any }
        }
    }

    fn bindgen(&self, class_name: &Ident) -> TokenStream2 {
        quote! {}
    }
}

#[derive(Debug)]
struct ClassFunction {
    name: Ident,
    parens: token::Paren,
    args: Punctuated<ClassArg, token::Comma>,
    body_delin: Option<token::Brace>,
    body: Option<TokenStream2>,
}

impl Parse for ClassFunction {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let name = input.parse()?;
        let args_tokens;
        let parens = parenthesized!(args_tokens in input);
        let args = Punctuated::parse_terminated(&args_tokens)?;
        let mut body_delin = None;
        let mut body = None;
        if input.peek(token::Brace) {
            let body_tokens;
            body_delin = Some(braced!(body_tokens in input));
            body = Some(body_tokens.parse()?);
        }

        Ok(ClassFunction {
            name,
            parens,
            args,
            body_delin,
            body,
        })
    }
}

impl ClassFunction {
    fn bindgen(&self, class_name: &Ident) -> TokenStream2 {
        let mut bindgen_args = TokenStream2::empty();
        let mut name = self.name.clone();
        let args = self.args.iter().map(|arg| {
            let name = &arg.name;
            let ty = &arg.ty;
            quote! { #name: #ty }
        });
        if name.to_string() == "constructor" {
            quote! {
                #[wasm_bindgen(constructor)]
                pub fn new(#(#args),*) -> #class_name;
            }
        } else {
            quote! {
                #[wasm_bindgen(method)]
                pub fn #name(_: &#class_name, #(#args),*);
            }
        }

    }
}

#[derive(Debug)]
enum ClassItem {
    Arg(ClassArg),
    Func(ClassFunction),
}

impl Parse for ClassItem {
    fn parse(input: ParseStream) -> SynResult<Self> {
        if input.peek2(token::Colon) {
            Ok(ClassItem::Arg(input.parse()?))
        } else {
            Ok(ClassItem::Func(input.parse()?))
        }
    }
}

impl ClassItem {
    fn bindgen(&self, class_name: &Ident) -> TokenStream2 {
        match self {
            ClassItem::Arg(arg) => arg.bindgen(class_name),
            ClassItem::Func(func) => func.bindgen(class_name),
        }
    }
}

#[derive(Debug)]
struct ClassDef {
    class: kw::class,
    class_name: Ident,
    extends: Option<kw::extends>,
    base_class: Option<Ident>,
    body_delin: token::Brace,
    body: Punctuated<ClassItem, token::Semi>,
}

impl Parse for ClassDef {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let class = input.parse()?;
        let class_name = input.parse()?;
        let mut extends = None;
        let mut base_class = None;
        if input.peek(kw::extends) {
            extends = Some(input.parse()?);
            base_class = Some(input.parse()?);
        }
        let body_tokens;
        let body_delin = braced!(body_tokens in input);
        let body = Punctuated::parse_terminated(&body_tokens)?;

        Ok(ClassDef {
            class,
            class_name,
            extends,
            base_class,
            body_delin,
            body,
        })
    }
}

impl ClassDef {
    fn bindgen(&self) -> TokenStream2 {
        let name = &self.class_name;
        let mut bindgen_args = TokenStream2::empty();
        if let Some(ref base) = self.base_class {
            bindgen_args = quote! { extends = #base };
        }
        let gens = self.body.iter().map(|item| item.bindgen(name));
        quote! {
            #[wasm_bindgen(#bindgen_args)]
            pub type #name;

            #(#gens)*
        }
    }
}

#[derive(Debug)]
struct ClassInput {
    defs: Punctuated<ClassDef, token::Semi>,
}

impl Parse for ClassInput {
    fn parse(input: ParseStream) -> SynResult<Self> {
        Ok(ClassInput {
            defs: Punctuated::parse_terminated(&input)?,
        })
    }
}

impl ClassInput {
    fn bindgen(&self) -> TokenStream2 {
        let defs = self.defs.iter().map(ClassDef::bindgen);
        quote! {
            #[wasm_bindgen]
            extern "C" {
                #(#defs)*
            }
        }
    }
}
