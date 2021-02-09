use variant_access_derive::*;
use variant_access_traits::*;

#[derive(Debug)]
pub struct Test<T, U>{
    inner: T,
    outer: U,
}

#[derive(VariantAccess, PartialEq, Debug)]
pub enum Enum<Y: 'static, X: 'static> {
    F1(Y),
    F2(Test<X, Y>)
}

fn main() {

}