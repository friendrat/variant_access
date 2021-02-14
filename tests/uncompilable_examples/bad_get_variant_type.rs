use variant_access_traits::*;
use variant_access_derive::*;

#[derive(VariantAccess)]
enum Enum {
     F1(i64),
     F2(bool)
}

fn main() {
    let instance = Enum::F1(42);
    let _: Result<&i32, VariantAccessError> = instance.get_variant();
}
