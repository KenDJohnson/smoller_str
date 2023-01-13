use syn::{Error, Field, Fields, Ident, LitStr, Result, Variant};

// pub enum VariantKind<'a> {
//     Known(&'a Ident, LitStr),
//     Unknown(&'a Ident, )
// }
pub enum VariantData<'a> {
    Value(&'a Ident, LitStr),
    Field(&'a Ident, &'a Field),
}

pub fn parse_variants<'a>(
    variants: impl Iterator<Item = &'a Variant>,
) -> Result<Vec<VariantData<'a>>> {
    let mut res = Vec::new();

    for variant in variants {
        let ident = &variant.ident;
        let attr = variant
            .attrs
            .iter()
            .find(|attr| attr.path.is_ident("value"));
        let data = match &variant.fields {
            Fields::Named(_) => {
                return Err(Error::new_spanned(variant, "named fields are not allowed"))
            }
            Fields::Unnamed(f) => {
                if f.unnamed.len() != 1 {
                    return Err(Error::new_spanned(
                        f,
                        "only a single tuple field is allowed",
                    ));
                }
                if attr.is_some() {
                    return Err(Error::new_spanned(
                        f,
                        "variant field cannot be used with `#[value(\"...\")]` attribute",
                    ));
                }
                VariantData::Field(ident, &f.unnamed[0])
            }
            Fields::Unit => {
                let value = match attr {
                    Some(attr) => attr.parse_args()?,
                    None => {
                        return Err(Error::new_spanned(
                            variant,
                            "missing `#[value(\"...\")]` attribute",
                        ));
                    }
                };
                VariantData::Value(ident, value)
            }
        };
        res.push(data);
    }
    Ok(res)
}

// fn parse_value_variant(variant: &Variant, allow_field: bool) -> Result<(&Ident, Option<LitStr>)> {
//     if !allow_field && variant.fields != Fields::Unit {
//         return Err(Error::new_spanned(v, "only unit variants are allowed"));
//     }
//     let attr = match v.attrs.iter().find(|attr| attr.path.is_ident("value")) {
//         Some(a) => a,
//         None => {
//             return Err(Error::new_spanned(
//                 v,
//                 "`#[value(\"...\")]` attribute required",
//             ))
//         }
//     };
// }
