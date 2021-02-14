use variant_access_traits::*;
use variant_access_derive::*;

#[derive(VariantAccess)]
pub enum Enum<Y: 'static, X: 'static> {
    F1(Y),
    F2(X)
}

pub struct Wrapper(Enum<i64, bool>);

fn main() {
    let _ = Wrapper(create_variant_from(42.0));
}