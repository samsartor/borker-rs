extern crate proc_macro;

use proc_macro::{TokenStream};
use proc_macro2::{TokenStream as TokenStream2};
use quote::{quote};

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
    fn getter_return(&self) -> TokenStream2 {
        let ty = &self.ty;
        quote! { #ty }
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

    fn bindgen(&self, class_name: &Ident) -> TokenStream2 {
        let setter_name = Ident::new(&format!("set_{}", self.name), self.name.span());
        let getter_name = &self.name;
        let getter_return= self.getter_return();
        let setter_param = self.setter_param();
        quote! {
            #[wasm_bindgen(method, getter)]
            pub fn #getter_name(_: &#class_name) -> #getter_return;

            #[wasm_bindgen(method, setter)]
            pub fn #setter_name(_: &#class_name, #setter_param);
        }
    }

    fn ts_type(&self) -> TokenStream2 {
        if self.ty == parse_quote! { String } {
            quote! { string }
        } else if self.ty == parse_quote! { Vec<u8> } {
            quote! { Uint8Array }
        } else if self.ty == parse_quote! { u8 } {
            quote! { number }
        } else {
            quote! { any }
        }
    }

    fn ts(&self) -> TokenStream2 {
        let name = &self.name;
        let ty = self.ts_type();
        quote! { #name : #ty }
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
    fn bindgen(&self, class_name: &Ident, abstr: bool) -> TokenStream2 {
        let args = self.args.iter().map(|arg| {
            let name = &arg.name;
            let ty = &arg.ty;
            quote! { #name: #ty }
        });
        let name = &self.name;
        if name.to_string() == "constructor" {
            if abstr { return TokenStream2::new(); }
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


    fn js(&self) -> TokenStream2 {
        let name = &self.name;
        let args = self.args.iter().map(|arg| &arg.name);
        let body = self.body.as_ref().expect("function must have js body");
        quote! {
            #name(#(#args),*) {
                #body
            }
        }
    }

    fn ts(&self, abstr: bool) -> TokenStream2 {
        let name = &self.name;
        if name.to_string() == "constructor" && abstr { return TokenStream2::new(); }
        let args = self.args.iter().map(ClassArg::ts);
        quote! { #name(#(#args),*); }
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
    fn bindgen(&self, class_name: &Ident, abstr: bool) -> TokenStream2 {
        match self {
            ClassItem::Arg(arg) => arg.bindgen(class_name),
            ClassItem::Func(func) => func.bindgen(class_name, abstr),
        }
    }

    fn js(&self) -> TokenStream2 {
        match self {
            ClassItem::Arg(_) => TokenStream2::new(),
            ClassItem::Func(func) => func.js(),
        }
    }

    fn ts(&self, abstr: bool) -> TokenStream2 {
        match self {
            ClassItem::Arg(arg) => arg.ts(),
            ClassItem::Func(func) => func.ts(abstr),
        }
    }
}

#[derive(Debug)]
struct ClassDef {
    abstr: Option<token::Abstract>,
    class: kw::class,
    class_name: Ident,
    extends: Option<kw::extends>,
    base_class: Option<Ident>,
    body_delin: token::Brace,
    body: Punctuated<ClassItem, token::Semi>,
}

impl Parse for ClassDef {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let mut abstr = None;
        if input.peek(token::Abstract) {
            abstr = Some(input.parse()?);
        }
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
            abstr,
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
        let mut bindgen_args = TokenStream2::new();
        if let Some(ref base) = self.base_class {
            bindgen_args = quote! { extends = #base };
        }
        let gens = self.body.iter().map(|item| item.bindgen(name, self.abstr.is_some()));
        quote! {
            #[wasm_bindgen(#bindgen_args)]
            pub type #name;

            #(#gens)*
        }
    }

    fn js(&self) -> TokenStream2 {
        let name = &self.class_name;
        let mut extends = TokenStream2::new();
        if let Some(ref base) = self.base_class {
            extends = quote! { extends #base };
        }
        let jss = self.body.iter().map(|item| item.js());
        quote! {
            class #name #extends {
                #(#jss)*
            }
        }
    }

    fn ts(&self) -> TokenStream2 {
        let class_name = &self.class_name;
        let mut def = quote! { class #class_name };
        if let Some(ref base) = self.base_class {
            def = quote! { #def extends #base };
        }
        if self.abstr.is_some() {
            def = quote! { abstract #def };
        }
        let tss = self.body.iter().map(|item| item.ts(self.abstr.is_some()));
        quote! {
            #def {
                #(#tss;)*
            }
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
        let jss = self.defs.iter().map(ClassDef::js);
        let js = (quote! { #(#jss)* }).to_string();
        let tss = self.defs.iter().map(ClassDef::ts);
        let ts = (quote! { #(#tss)* }).to_string();
        quote! {
            #[wasm_bindgen(typescript_custom_section)]
            const __TS_APPEND: &'static str = #ts;

            #[wasm_bindgen(inline_js = #js)]

            #[wasm_bindgen]
            extern "C" {
                #(#defs)*
            }
        }
    }
}
