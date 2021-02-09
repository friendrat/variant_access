use variant_access_derive::*;
use variant_access_traits::*;

#[derive(VariantAccess, PartialEq, Debug)]
pub enum Enum<Y: 'static, X> {
    F1(Y),
    F2(X)
}

fn main() {

}

