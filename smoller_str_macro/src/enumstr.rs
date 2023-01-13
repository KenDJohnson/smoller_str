use crate::shared::*;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    Attribute, Data, DataEnum, DeriveInput, Error, Fields, Ident, LitByteStr, LitStr, Result,
    Visibility,
};

pub struct EnumStrInput<'a> {
    pub variants: Vec<(Ident, LitStr)>,
    pub ident: &'a Ident,
    pub vis: &'a Visibility,
}

impl<'a> EnumStrInput<'a> {
    pub fn new(variants: Vec<(Ident, LitStr)>, ident: &'a Ident, vis: &'a Visibility) -> Self {
        Self {
            variants,
            ident,
            vis,
        }
    }
}

impl<'a> ToTokens for EnumStrInput<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let variants = &self.variants;
        let ty_ident = self.ident;

        let byte_str_to_some = variants.iter().map(|(ident, val)| {
            let bytes = LitByteStr::new(val.value().as_bytes(), val.span());
            quote! { #bytes => Some(#ty_ident::#ident), }
        });
        let to_str = variants
            .iter()
            .map(|(ident, val)| quote!( #ty_ident::#ident => #val, ));

        let vis = self.vis;

        let literal_values = variants.iter().map(|(ident, _)| quote! { Self::#ident });

        let impls = quote! {

            impl #ty_ident {
                #vis const fn new(s: &str) -> Option<Self> {
                    match s.as_bytes() {
                        #(#byte_str_to_some)*
                        _ => None,
                    }
                }
                #vis const fn as_str(&self) -> &'static str {
                    match self {
                        #(#to_str)*
                    }
                }
            }

            impl smoller_str::EnumStr for #ty_ident {
                const VALUES: &'static [Self] = &[
                    #( #literal_values ),*
                ];
                fn as_str(&self) -> &'static str {
                    #ty_ident::as_str(self)
                }
            }

            impl std::fmt::Display for #ty_ident {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    f.write_str(#ty_ident::as_str(self))
                }
            }

            impl std::str::FromStr for #ty_ident {
                type Err = ();
                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    #ty_ident::new(s).ok_or(())
                    // match s {
                    //     #(to_variant_ok)*
                    //     _ => Err(())
                    // }
                }
            }

            impl AsRef<str> for #ty_ident {
                fn as_ref(&self) -> &str {
                    #ty_ident::as_str(self)
                }
            }

            impl PartialEq<str> for #ty_ident {
                fn eq(&self, other: &str) -> bool {
                    self.as_str() == other
                }
            }

            impl PartialEq<&'_ str> for #ty_ident {
                fn eq(&self, other: &&str) -> bool {
                    self.as_str() == *other
                }
            }

            impl PartialEq<str> for &'_ #ty_ident {
                fn eq(&self, other: &str) -> bool {
                    #ty_ident::as_str(*self) == other
                }
            }

            impl PartialEq<#ty_ident> for str {
                fn eq(&self, other: &#ty_ident) -> bool {
                    self == other.as_str()
                }
            }
        };

        tokens.extend(impls);
    }
}

// #[derive(Debug)]
// struct EnumStrDerive<'a> {
//     body: &'a DataEnum,
//     attrs: &'a [Attribute],
//     vis: &'a Visibility,
//     ident: &'a Ident,
// }

// impl<'a> EnumStrDerive<'a> {
//     fn build(self) -> Result<TokenStream> {
//         let variants = self.parse_variants()?;
//         dbg!(&variants);
//         dbg!(&self.attrs);
//         // let to_variant_ok = variants
//         //     .iter()
//         //     .map(|(ident, val)| quote! { #val => Ok(#ty_ident::#ident), })
//         //     .collect();

//         Ok(EnumStrInput::new(variants, self.ident, self.vis).to_token_stream())
//     }

//     fn parse_variants(&self) -> Result<Vec<(&'a Ident, LitStr)>> {
//         let mut variants = Vec::with_capacity(self.body.variants.len());
//         for v in &self.body.variants {
//             if v.fields != Fields::Unit {
//                 return Err(Error::new_spanned(
//                     v,
//                     "only unit variants can be used with EnumStr",
//                 ));
//             }
//             let attr = match v.attrs.iter().find(|attr| attr.path.is_ident("value")) {
//                 Some(a) => a,
//                 None => {
//                     return Err(Error::new_spanned(
//                         v,
//                         "`#[value(\"...\")]` attribute required",
//                     ))
//                 }
//             };

//             let val: LitStr = attr.parse_args()?;
//             variants.push((&v.ident, val));
//         }
//         Ok(variants)
//     }
// }

pub fn derive(node: &DeriveInput) -> Result<TokenStream> {
    let enum_node = match &node.data {
        Data::Enum(e) => e,
        _ => {
            return Err(Error::new_spanned(
                node,
                "EnumStr can only be used on enums",
            ))
        }
    };

    let variants = parse_variants(enum_node.variants.iter())?;
    let mut value_variants = Vec::with_capacity(variants.len());
    for variant in variants {
        match variant {
            VariantData::Field(ident, _) => {
                return Err(Error::new_spanned(ident, "only unit variants are allowed"))
            }
            VariantData::Value(ident, value) => {
                value_variants.push((ident.clone(), value));
            }
        }
    }

    Ok(EnumStrInput::new(value_variants, &node.ident, &node.vis).to_token_stream())
}
