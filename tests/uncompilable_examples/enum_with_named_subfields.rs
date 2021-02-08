use variant_access_derive::*;

#[derive(VariantAccess)]
enum Enum {
    F1{a: i64, b: bool},
    F2(bool)
}

fn main() {

}
