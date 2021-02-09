use variant_access_derive::*;

#[derive(VariantAccess, PartialEq, Debug)]
pub enum Enum<'a, Y: 'static, X: 'static> {
    F1(&'a Y),
    F2(X)
}

fn main() {

}

