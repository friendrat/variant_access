extern crate proc_macro;

use std::any::type_name;
use std::iter::Enumerate;
use std::collections::HashMap;
use proc_macro::TokenStream;
use quote::quote;
use syn::{self, Ident, Data, Type, TypePath, DeriveInput};

#[proc_macro_derive(VariantAccess)]
pub fn variant_access_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_variant_access(&ast)
}

fn type_of<T>(_: T) -> &'static str {
    type_name::<T>()
}

fn in_variant<T>() -> bool {
    type_name::<T>() == type_name::<i64>() || type_name::<T>() == type_name::<bool>()
}

/// Makes a map of the form < field type : field name >
///
/// Provides validation that checks that no field type is used twice, else panics.
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
fn make_has_variant(ast: &DeriveInput, types: HashMap<&Ident, &Ident>) -> TokenStream {
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
    piece.push_str("} }");
    piece.parse().unwrap()
}

fn impl_variant_access(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let types = fetch_types_from_enum(ast);
    let mut tokens = make_has_variant(&ast, types);

    let gen = quote! {
        impl ContainsVariant for #name {
            fn contains_variant<T>(&self) -> Result<bool, ()> {

                Ok(self.has_variant::<T>())
            }
        }
    };
    tokens.extend::<TokenStream>(gen.into());
    tokens
}
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
