extern crate proc_macro;

use std::any::type_name;
use proc_macro::TokenStream;
use quote::quote;
use syn::{self, Data, Type, TypePath};

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

fn fetch_types_from_enum(ast: &syn::DeriveInput) -> Vec<&syn::Ident> {
    let mut types: Vec<&syn::Ident> = Vec::new();
    if let Data::Enum(data) = &ast.data {
        for var in data.variants.iter() {
            if let syn::Fields::Unnamed(_field) = &var.fields{
                if _field.unnamed.len() > 1 {
                    panic!("Cannot only derive for enums with primitive variants,\
                           found complex type");
                }
                for field_entry in _field.unnamed.iter() {
                    if let syn::Type::Path(_type) = &field_entry.ty {
                        if let Some(y) = &_type.path.segments.last(){
                            types.push(&y.ident);
                        }
                    }
                }
            }
        }
    } else{
        panic!("Can only derive VariantAccess for enums.")
    };
    types
}

/// Makes a helper function that determines if one of the enum fields contains the generic type
///
/// Example:
///     enum Enum {
///         f1(i64),
///         f2(bool)
///     }
/// in_variant::<i64>() returns true
/// in_variant::<i32>() returns false
fn make_in_variant(types: Vec<&syn::Ident>) -> TokenStream {
    let mut piece : String = "fn in_variant<T>() -> bool { ".to_owned();
    for type_ in types.iter() {
        if type_ == types.last().unwrap() {
            piece.push_str(&format!("std::any::type_name::<T>() == std::any::type_name::<{}>()",
                                type_.to_string()));
        } else {
            piece.push_str(&format!("std::any::type_name::<T>() == std::any::type_name::<{}>() || ",
                                type_.to_string()));
        }
    }
    piece.push_str("}");
    piece.parse().unwrap()
}

fn impl_variant_access(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let types = fetch_types_from_enum(ast);
    let mut tokens = make_in_variant(types);

    let gen = quote! {
        impl HasVariant for #name {
            fn has_variant<T>(&self) -> Result<bool, ()> {

                Ok(in_variant::<T>())
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
