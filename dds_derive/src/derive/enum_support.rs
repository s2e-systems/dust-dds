use syn::{Fields, DataEnum, Ident, Expr, ExprLit, Lit};

pub enum BitBound {
    Bit8,
    Bit16,
    Bit32,
}

// The return of this function is a Vec instead of a HashMap so that the tests give
// consistent results. Iterating over a HashMap gives different order of members every time.
// The order is also important for the XML string generation.
pub fn read_enum_variant_discriminant_mapping(data_enum: &DataEnum) -> Vec<(Ident, usize)> {
    let mut map = Vec::new();
    let mut discriminant = 0;
    for variant in data_enum.variants.iter() {
        match variant.fields {
            Fields::Unit => (),
            _ => panic!("Only unit enums can be used when deriving CdrSerialize and CdrDeserialize"),
        }
        if let Some((_,discriminant_expr)) = &variant.discriminant {
            match discriminant_expr {
                Expr::Lit(ExprLit{lit,..}) => match lit {
                    Lit::Int(lit_int) => discriminant = lit_int.base10_parse().expect("Integer should be verified by compiler"),
                    _ => panic!("Only literal integer discrimimants are expected")
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
    if max_discriminant >= &0 && max_discriminant <= &(u8::MAX as usize)  {
        BitBound::Bit8
    } else if max_discriminant > &(u8::MAX as usize) && max_discriminant <= &(u16::MAX as usize) {
        BitBound::Bit16
    } else if max_discriminant > &(u16::MAX as usize) && max_discriminant <= &(u32::MAX as usize) {
        BitBound::Bit32
    } else {
        panic!("Enum discriminant value outside of supported range")
    }
}