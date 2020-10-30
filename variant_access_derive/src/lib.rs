extern crate proc_macro;

use std::any::type_name;
#[allow(unused_imports)]
use std::iter::Enumerate;
use std::collections::HashMap;
use proc_macro::TokenStream;
use quote::quote;
use syn::{self, Ident, Data, DeriveInput, Token};

#[proc_macro_derive(VariantAccess)]
pub fn variant_access_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_variant_access(&ast)
}


/// Makes a map of the form < field type : field name >
///
/// Provides validation
///     -- that no field type is used twice.
///     -- that input is enum.
///     -- that all field types are primitive and do not have named fields
/// if any of these validations fail, this function panics and halts compilation
///
/// Example:
///     enum Enum {
///         f1(i64),
///         f2(bool)
///     }
/// returns [ < i64: f1 > , < bool, f2 > ]
///
/// Example:
///     enum Enum {
///         f1(i64),
///         f2(bool),
///         f3(i64),
///     }
/// panics as two distinct fields have type i64.
///
/// Example:
///     enum Enum {
///         f1(i64, i32),
///         f2{x: bool}
///     }
/// panics as f1 does not have a primitive type or because f2 has a named field.
fn fetch_types_from_enum(ast: &DeriveInput) -> HashMap<&Ident, &Ident> {
    let mut types: HashMap<&Ident, &Ident> = HashMap::new();
    if let Data::Enum(data) = &ast.data {
        for var in data.variants.iter() {

            if let syn::Fields::Unnamed(_field) = &var.fields{
                if _field.unnamed.len() > 1 {
                    panic!("Cannot only derive for enums with primitive variants,\
                           found complex type");
                }

                for field_entry in _field.unnamed.iter() {

                    if let syn::Type::Path(_type) = &field_entry.ty {
                        if let Some(_type) = &_type.path.segments.last(){

                            let found_before = types.insert(&_type.ident, &var.ident);
                            if let Some(_) = found_before {
                                panic!("Cannot derive VariantAccess for enum with multiple fields \
                                        of same type");
                            }
                        }
                    }
                }
            } else {
                panic!("Cannot derive VariantAccess for with whose types have named fields.")
            }
        }
    } else {
        panic!("Can only derive VariantAccess for enums.")
    };
    types
}

/// Implements HasVariant trait that determines if one of the enum fields contains the input type
///
/// Example:
///     enum Enum {
///         f1(i64),
///         f2(bool)
///     }
/// in_variant::<i64>() returns true
/// in_variant::<i32>() returns false
fn impl_has_variant(ast: &DeriveInput, types: &HashMap<&Ident, &Ident>) -> TokenStream {
    let name = &ast.ident;
    let mut piece : String = format!("impl HasVariant for {}", name.to_string());
    piece.push_str(" { fn has_variant<T>(&self) -> bool { ");

    for (ix, type_) in types.keys().enumerate() {
        if ix == types.len() - 1 {
            piece.push_str(&format!("std::any::type_name::<T>() == std::any::type_name::<{}>()",
                                    type_.to_string()));
        } else {
            piece.push_str(&format!("std::any::type_name::<T>() == std::any::type_name::<{}>() || ",
                                    type_.to_string()));
        }
    }
    piece.push_str("} fn type_of<T>(&self, _: T) -> &'static str { std::any::type_name::<T>() } }");
    piece.parse().unwrap()
}

fn impl_contains_variant(ast: &DeriveInput, types: &HashMap<&Ident, &Ident>) -> TokenStream {
    let name = &ast.ident.to_string();
    let mut piece : String = format!("impl ContainsVariant for {}", name);
    piece.push_str(" { fn contains_variant<T>(&self) -> Result<bool, ()> { \
                    if self.has_variant::<T>() { return match self { ");
    for (ix, field_) in types.values().enumerate() {
        piece.push_str(&format!("{}::{}(inner) => Ok(self.type_of(*inner) == std::any::type_name::<T>())",
                                name, field_));
       if ix != types.len() - 1 {
           piece.push_str(", ");
       }  else {
           piece.push_str("}; } Err(()) } }");
       }
    }
    println!("{}", piece);
    return piece.parse().unwrap();
}

fn impl_variant_access(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let types = fetch_types_from_enum(ast);
    let mut tokens = impl_has_variant(&ast, &types);

    tokens.extend::<TokenStream>(impl_contains_variant(&ast, &types));
    tokens
}
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
