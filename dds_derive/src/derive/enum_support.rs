use syn::{
    parse::Parse, punctuated::Punctuated, DataEnum, Expr, ExprLit, Fields, Ident, Lit,
    MetaNameValue, Token,
};

pub enum BitBound {
    Bit8,
    Bit16,
    Bit32,
}

#[derive(Default)]
pub struct VariantArgs {
    pub discriminant: Option<Expr>,
}

impl Parse for VariantArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut discriminant = None;

        let args = Punctuated::<MetaNameValue, Token![,]>::parse_terminated(input)?;
        for arg in args {
            let ident = arg.path.require_ident()?;

            if ident == "discriminant" {
                if discriminant.is_some() {
                    return Err(syn::Error::new(
                        ident.span(),
                        "Argument `discriminant` already defined",
                    ));
                }
                discriminant = Some(arg.value);
            } else {
                return Err(syn::Error::new(
                    ident.span(),
                    format!("Unknown argument `{ident}`"),
                ));
            }
        }

        Ok(Self { discriminant })
    }
}

// The return of this function is a Vec instead of a HashMap so that the tests give
// consistent results. Iterating over a HashMap gives different order of members every time.
// The order is also important for the XML string generation.
pub fn read_enum_variant_discriminant_mapping(data_enum: &DataEnum) -> Vec<(Ident, usize)> {
    let mut map = Vec::new();
    let mut discriminant = 0;
    for variant in data_enum.variants.iter() {
        if let Some((_, discriminant_expr)) = &variant.discriminant {
            match discriminant_expr {
                Expr::Lit(ExprLit { lit, .. }) => match lit {
                    Lit::Int(lit_int) => {
                        discriminant = lit_int
                            .base10_parse()
                            .expect("Integer should be verified by compiler")
                    }
                    _ => panic!("Only literal integer discrimimants are expected"),
                },
                _ => panic!("Only literal discrimimants are expected"),
            }
        }
        map.push((variant.ident.clone(), discriminant));
        discriminant += 1;
    }

    map
}

pub fn get_enum_bitbound(max_discriminant: &usize) -> BitBound {
    if max_discriminant >= &0 && max_discriminant <= &(u8::MAX as usize) {
        BitBound::Bit8
    } else if max_discriminant > &(u8::MAX as usize) && max_discriminant <= &(u16::MAX as usize) {
        BitBound::Bit16
    } else if max_discriminant > &(u16::MAX as usize) && max_discriminant <= &(u32::MAX as usize) {
        BitBound::Bit32
    } else {
        panic!("Enum discriminant value outside of supported range")
    }
}

pub fn is_enum_xtypes_union(data_enum: &DataEnum) -> bool {
    data_enum
        .variants
        .iter()
        .any(|v| !matches!(&v.fields, Fields::Unit))
}
