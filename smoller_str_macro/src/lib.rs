use proc_macro::TokenStream;

use syn::{bracketed, parse::Parse, parse_macro_input, DeriveInput, Ident, LitStr, Token};

use crate::smoller::SmollerOpts;

mod enumstr;
mod shared;
mod smoller;

#[proc_macro_derive(EnumStr, attributes(value))]
pub fn derive_enumstr(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    enumstr::derive(&input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

// #[proc_macro_derive(SmollerStr, attributes(value))]
// pub fn derive_smoller_str(input: TokenStream) -> TokenStream {
//     let input = parse_macro_input!(input as DeriveInput);
//     smoller::derive(&input)
//         .unwrap_or_else(|err| err.to_compile_error())
//         .into()
// }

#[proc_macro_attribute]
pub fn smoller_str(attr: TokenStream, item: TokenStream) -> TokenStream {
    dbg!(&attr);
    dbg!(&item);
    let input = parse_macro_input!(item as DeriveInput);

    let opts = if attr.is_empty() {
        SmollerOpts::default()
    } else {
        let attrs = parse_macro_input!(attr as SmollerOpts);
        // let args = attr
        // SmollerOpts {

        // }
        attrs
    };

    smoller::smoller(opts, input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro]
pub fn smoller_strings(item: TokenStream) -> TokenStream {
    dbg!(&item);

    #[derive(Debug)]
    struct Input {
        name: Ident,
        comma: Token![,],
        bracket: syn::token::Bracket,
        content: proc_macro2::TokenStream,
    }
    impl Parse for Input {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let content;
            Ok(Input {
                name: input.parse()?,
                comma: input.parse()?,
                bracket: bracketed!(content in input),
                content: content.parse()?,
            })
        }
    }
    let Input { name, content, .. } = parse_macro_input!(item as Input);
    let opts = SmollerOpts::default();

    let strings = match syn::parse2::<smoller::SmolItems>(content) {
        Ok(smoller::SmolItems(strings)) => strings,
        Err(e) => return e.to_compile_error().into(),
    };

    smoller::smoller_func(opts, name, strings)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro]
// pub fn include_smoller_strings_inner(item: TokenStream) -> TokenStream {
pub fn include_smoller_strings(item: TokenStream) -> TokenStream {
    dbg!(&item);
    struct Input {
        name: Ident,
        _comma: Token![,],
        file: LitStr,
    }
    impl Parse for Input {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            Ok(Input {
                name: input.parse()?,
                _comma: input.parse()?,
                file: input.parse()?,
            })
        }
    }

    let Input { name, file, .. } = parse_macro_input!(item as Input);

    // let span_file = ident.span().source_file().path();
    // let file_name = file.value();
    // let input_file = span_file.parent().unwrap().join(file_name);
    let input_file = file.value();

    let file_contents = match std::fs::read_to_string(&input_file) {
        Ok(c) => c.lines().collect::<Vec<_>>().join(",\n"),
        Err(e) => {
            return syn::Error::new_spanned(file, format!("Unable to read {input_file:?}: {e}"))
                .to_compile_error()
                .into();
        }
    };

    let strings = match syn::parse_str(&file_contents) {
        Ok(smoller::SmolItems(strings)) => strings,
        Err(e) => return e.to_compile_error().into(),
    };

    let opts = SmollerOpts::default();
    smoller::smoller_func(opts, name, strings)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
