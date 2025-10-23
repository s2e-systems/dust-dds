use syn::{DataEnum, Expr, ExprLit, Ident, Lit};

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
