use variant_access_traits::{HasVariant, ContainsVariant};
use variant_access_derive::*;

#[derive(VariantAccess)]
enum Test {
    F1(i32),
    F2(bool)
}

#[test]
fn test_derive() {
    let test = Test::F1(3);
    assert!(test.has_variant::<i32>());
}
