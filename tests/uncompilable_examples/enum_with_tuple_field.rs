use variant_access_derive::*;

#[derive(VariantAccess)]
enum Enum {
    F1(i64, bool),
    F2(bool)
}

fn main() {

}
