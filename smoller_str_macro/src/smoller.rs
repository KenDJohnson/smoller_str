use crate::{enumstr, shared::*};

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    Attribute, Data, DeriveInput, Error, Expr, ExprAssign, ExprLit, Ident, Lit, LitStr, Result,
    Token, Visibility,
};

use convert_case::{Case, Casing};

// pub fn derive(node: &DeriveInput) -> Result<TokenStream> {
//     let enum_node = match &node.data {
//         Data::Enum(e) => e,
//         _ => {
//             return Err(Error::new_spanned(
//                 node,
//                 "SmollerStr can only be used on enums",
//             ))
//         }
//     };

//     let variants = parse_variants(enum_node.variants.iter())?;
//     let mut value_variants = Vec::with_capacity(variants.len());
//     for variant in variants {
//         match variant {
//             VariantData::Field(ident, _) => {
//                 return Err(Error::new_spanned(ident, "only unit variants are allowed"))
//             }
//             VariantData::Value(ident, value) => {
//                 value_variants.push((ident, value));
//             }
//         }
//     }

//     let val = EnumStrDerive {
//         body: enum_node,
//         attrs: &node.attrs,
//         vis: &node.vis,
//         ident: &node.ident,
//     };
//     // dbg!(&val);
//     val.build()
// }

// pub struct SmollerInput {
//     attrs: Vec<Attribute>,
// }

// impl Parse for SmollerInput {
//     fn parse(input: ParseStream) -> Result<Self> {
//         let attrs: Vec<Attribute> = input.call(Attribute::parse_outer)?;
//     }
// }

// fn check_derive()

pub struct SmollerOpts {
    pub impl_deref: bool,
    pub no_derives: bool,
}

fn parse_setting(e: &ExprAssign) -> Result<(&Ident, bool)> {
    let Expr::Path(name) = &*e.left else {
        return Err(Error::new_spanned(&e.left, "expected an identifier"));
    };
    if name.path.segments.len() != 1 {
        return Err(Error::new_spanned(&e.left, "expected an identifier"));
    }
    let ident = &name.path.segments[0].ident;
    let Expr::Lit(ExprLit { lit: Lit::Bool(lit), .. }) = &*e.right else {
        return Err(Error::new_spanned(&e.right, "expected a bool"));
    };
    Ok((ident, lit.value))
}

impl Parse for SmollerOpts {
    fn parse(input: ParseStream) -> Result<Self> {
        // let attrs = input.call(Attribute::parse_outer)?;
        // dbg!(&attrs);
        // assert_eq!(attrs.len(), 1);
        // let attr = &attrs[0];
        // assert!(attr.path.is_ident("smoller_str"));
        let exprs: Punctuated<Expr, Token![,]> = input.parse_terminated(Expr::parse)?;
        let mut opts = Self::default();
        for expr in exprs.iter() {
            match expr {
                Expr::Assign(e) => {
                    let (name, value) = parse_setting(e)?;
                    if name == "deref" {
                        opts.impl_deref = value;
                    } else if name == "derives" {
                        opts.no_derives = !value;
                    } else {
                        return Err(Error::new_spanned(name, "invalid argument"));
                    }
                }
                _ => {
                    return Err(Error::new_spanned(expr, "invalid argument"));
                }
            }
        }
        // let exprs = attr.parse_args_with(Punctuated::<Expr, Token![,]>::parse_terminated)?;
        Ok(opts)
    }
}

impl Default for SmollerOpts {
    fn default() -> Self {
        Self {
            impl_deref: true,
            no_derives: false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum SmolItem {
    String(LitStr),
    Value(LitStr, Ident),
}
impl Parse for SmolItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let lit: LitStr = input.parse()?;
        if input.peek(Token![=]) {
            let _: Token![=] = input.parse()?;
            let ident = input.parse()?;
            Ok(Self::Value(lit, ident))
        } else {
            Ok(Self::String(lit))
        }
    }
}
#[derive(Debug)]
pub struct SmolItems(pub Vec<SmolItem>);

impl Parse for SmolItems {
    fn parse(input: ParseStream) -> Result<Self> {
        let string_items: Punctuated<SmolItem, Token![,]> =
            input.parse_terminated(SmolItem::parse)?;
        Ok(Self(string_items.iter().cloned().collect()))
    }
}

pub fn smoller_func(opts: SmollerOpts, name: Ident, strings: Vec<SmolItem>) -> Result<TokenStream> {
    let variants: Vec<_> = strings
        .iter()
        .map(|item| {
            match item {
                SmolItem::String(lit) => {
                    let value = lit.value();
                    let value_parts = value
                        .split_whitespace()
                        .filter(|part| !part.is_empty() && part.chars().all(char::is_alphanumeric))
                        .collect::<Vec<_>>();
                    let value = value_parts.join(" ");

                    let ident =
                        format_ident!("{}", value.to_case(Case::UpperCamel), span = lit.span());
                    quote! {
                        #[value(#value)] #ident
                    }
                }
                SmolItem::Value(lit, ident) => {
                    quote! {
                        #[value(#lit)] #ident
                    }
                }
            }
            // (ident, lit.clone())
        })
        .collect();
    dbg!(&variants);

    // let opts = SmollerOpts::default();

    // let enum_item_toks = quote! {
    //     #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumStr)]
    //     pub enum #name {
    //         #( #variants ),*
    //     }
    // }
    // .into();

    // let enum_item = parse_macro_input!(enum_item_toks as DeriveInput);
    let enum_item = parse_quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumStr)]
        pub enum #name {
            #( #variants ),*
        }
    };

    smoller(opts, enum_item)
}

pub fn smoller(opts: SmollerOpts, mut item: DeriveInput) -> Result<TokenStream> {
    let mut derive = None;
    item.attrs.retain(|attr| {
        if attr.path.is_ident("derive") {
            derive = Some(attr.clone());
            false
        } else {
            true
        }
    });

    let enum_ident = item.ident.clone();
    let repr_ident = format_ident!("{}Repr", enum_ident);
    item.ident = repr_ident.clone();

    let vis = item.vis.clone();
    item.vis = Visibility::Inherited;

    let repr_variants = match &item.data {
        Data::Enum(e) => e
            .variants
            .iter()
            .cloned()
            .map(|mut variant| {
                variant.attrs.retain(|attr| !attr.path.is_ident("value"));
                variant
            })
            .collect::<Vec<_>>(),
        _ => {
            return Err(Error::new_spanned(
                item,
                "`smol_str` can only be used on enums",
            ))
        }
    };

    let derive = match derive {
        Some(d) => d,
        None => {
            return Err(Error::new_spanned(
                item,
                "`smoller_str` must derive `EnumStr`",
            ))
        }
    };
    let derives = derive
        .parse_args_with(Punctuated::<Ident, Token![,]>::parse_separated_nonempty)?
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    dbg!(&derives);

    macro_rules! has_derive {
        ($trait:ident) => {
            derives.iter().any(|d| d == stringify!($trait))
        };
    }

    let repr_derives = derives.iter().filter(|&d| d != "EnumStr");

    let mut tokens = quote! {
        #[derive( #(#repr_derives),* )]
        #vis enum #repr_ident {
            #( #repr_variants ),*
        }
    };

    tokens.extend(enumstr::derive(&item)?);

    let wrapper_derives = derives
        .iter()
        .filter(|&d| !(d == "EnumStr" || d == "Copy" || d == "Hash"))
        .collect::<Vec<_>>();

    let impl_hash = has_derive!(Hash).then(|| {
        quote! {
            impl std::hash::Hash for #enum_ident {
                fn hash<H: std::hash::Hasher>(&self, hasher: &mut H) {
                    smoller_str::SmollerStr::as_str(self).hash(hasher)
                }
            }
        }
    });

    // if !has_derive!(Eq) {
    //     return Err(Error::new_spanned(
    //         derive,
    //         "`smoller_str` must derive `PartialEq`",
    //     ));
    // }

    let deref = opts.impl_deref.then(|| {
        quote! {
            impl std::ops::Deref for #enum_ident {
                type Target = str;

                fn deref(&self) -> &str {
                    smoller_str::SmollerStr::as_str(self)
                }
            }
        }
    });

    let literal_values = repr_variants
        .iter()
        .map(|v| quote! { Self::Builtin(#repr_ident :: #v) });

    tokens.extend(quote! {
        #[derive( #(#wrapper_derives),* )]
        #vis enum #enum_ident {
            Builtin(#repr_ident),
            // Unknown(smoller_str::SmolStr),
            Unknown(std::sync::Arc<str>)
        }

        impl smoller_str::SmollerStr for #enum_ident {
            const BUILTIN: &'static [Self] = &[
                #( #literal_values ),*
            ];

            fn new<S: AsRef<str> + ?Sized>(s: &S) -> Self {
                s.as_ref().parse().unwrap()
            }

            fn as_str(&self) -> &str {
                match self {
                    Self::Builtin(s) => s.as_str(),
                    Self::Unknown(s) => s,
                }
            }

            fn is_heap_allocated(&self) -> bool {
                matches!(self, Self::Unknown(_))
            }

            fn is_builtin_value(&self) -> bool {
                matches!(self, Self::Builtin(_))
            }
        }

        impl std::str::FromStr for #enum_ident {
            type Err = std::convert::Infallible;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                if let Ok(smoller) = s.parse() {
                    Ok(Self::Builtin(smoller))
                } else {
                    // Ok(Self::Unknown(smoller_str::SmolStr::new(s)))
                    Ok(Self::Unknown(s.into()))
                }
            }
        }

        #impl_hash

        impl PartialEq<str> for #enum_ident {
            fn eq(&self, other: &str) -> bool {
                smoller_str::SmollerStr::as_str(self) == other
            }
        }

        impl PartialEq<str> for &'_ #enum_ident {
            fn eq(&self, other: &str) -> bool {
                smoller_str::SmollerStr::as_str(*self) == other
            }
        }

        impl std::fmt::Display for #enum_ident {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str(smoller_str::SmollerStr::as_str(self))
            }
        }

        #deref

        impl std::borrow::Borrow<str> for #enum_ident {
            fn borrow(&self) -> &str {
                smoller_str::SmollerStr::as_str(self)
            }
        }

        impl AsRef<str> for #enum_ident {
            fn as_ref(&self) -> &str {
                smoller_str::SmollerStr::as_str(self)
            }
        }


    });

    // macro_rules! check_derive {
    //     ($($trait:ident),+ $(,)?) => {{
    //         $(
    //             if !derives.iter().any(|d| d == stringify!($trait)) {
    //                 return Err(Error::new_spanned(derives, concat!("`smoller_str` must derive `", stringify!($trait), "`")))
    //             }
    //         )+
    //     }}
    // }
    // check_derive!(EnumStr, Clone, Copy, PartialEq, Eq);

    // if !derives.iter().any(|d| d == "EnumStr") {
    //     return Err(Error::new_spanned(derives, "`smoller_str` must derive `EnumStr`"))
    // }
    Ok(tokens)
}
