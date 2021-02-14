use variant_access_traits::*;
use variant_access_derive::*;

#[derive(VariantAccess)]
pub enum Enum<Y: 'static, X: 'static> {
    F1(Y),
    F2(X)
}

fn main() {
    let _ = Enum::<i64, bool>::create_variant_from(2.0);
}