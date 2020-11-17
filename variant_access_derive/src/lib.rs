extern crate proc_macro;

#[allow(unused_imports)]
use std::any::type_name;
#[allow(unused_imports)]
use std::iter::Enumerate;

use std::collections::HashMap;
use proc_macro::TokenStream;
use syn::{self, Ident, Data, DeriveInput};


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
/// # Example
/// ```
/// enum Enum {
///     F1(i64),
///     F2(bool)
/// }
/// ```
/// returns [ < i64: f1 > , < bool: f2 > ]
///
/// # Example
/// ```
/// #[derive(VariantAccess)]
/// enum Enum {
///     F1(i64),
///     F2(bool),
///     F3(i64),
/// }
/// ```
/// panics in this function as two distinct fields have type i64.
///
/// # Example
/// ```
/// #[derive(VariantAccess)]
/// enum Enum {
///     F1(i64, i32),
///     F2{x: bool}
/// }
/// ```
/// panics in this function as F1 does not have a primitive type or because F2 has a named field.
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

                            let found_before = types.insert(&_type.ident,
                                                            &var.ident);
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

/// Implements ContainsVariant trait that determines
/// if one of the enum fields contains the input type
///
/// # Example
/// ```
///enum Enum {
///    F1(i64),
///    F2(bool)
///}
/// let result: bool = has_variant::<i64>(); // assigns true to result
/// let result: bool = has_variant::<i32>(); // assigns false to result
///
/// let instance = Enum::f1(42);
/// let result = instance.contains_variant::<i64>(); // result has value Ok(true)
/// let result = instance.contains_variant::<bool>(); // result has value Ok(false)
/// let result = instance.contains_variant::<i32>(); // result has value Err
/// ```
fn impl_contains_variant(ast: &DeriveInput, types: &HashMap<&Ident, &Ident>) -> TokenStream {
    let name = &ast.ident;
    let mut piece : String = format!("impl ContainsVariant for {}", name.to_string());
    piece.push_str(" { fn has_variant<T>(&self) -> bool { ");

    for (ix, type_) in types.keys().enumerate() {
        if ix == types.len() - 1 {
            piece.push_str(&format!("std::any::type_name::<T>()\
                                           == std::any::type_name::<{}>()",
                                    type_.to_string()));
        } else {
            piece.push_str(&format!("std::any::type_name::<T>() \
                                           == std::any::type_name::<{}>() || ",
                                    type_.to_string()));
        }
    }

    piece.push_str("} ");
    piece.push_str("fn contains_variant<T>(&self) -> Result<bool, VariantAccessError> { \
                    if self.has_variant::<T>() { return match self { ");
    for (ix, (type_, field_)) in types.iter().enumerate() {
        piece.push_str(&format!("{}::{}(inner) => \
                                Ok(std::any::type_name::<{}>() == std::any::type_name::<T>())",
                                name, field_, type_));
        if ix != types.len() - 1 {
            piece.push_str(", ");
        }  else {
            piece.push_str("}; } ");
            piece.push_str( &format!(" Err(VariantAccessError::invalid_type(\"{}\", std::any::type_name::<T>()))",
                                     name));
            piece.push_str(" } }");
        }
    }
    piece.parse().unwrap()
}

/// Implements the GetVariant trait that retrieves the
/// tagged value of the requested type, if possible
///
/// # Example:
/// ```
/// enum Enum {
///     F1(i64),
///     F2(bool)
/// }
/// let instance = Enum::f1(42);
///
/// let inner: &i64 = instance.get_variant::<i64>().unwrap(); // assigns &42 to inner_value
/// // let inner: &bool = instance.get_variant::<bool>().unwrap() // panics because of unhandled Err.
/// // let inner: &i32 = instance.get_variant::<i32>().unwrap() // will not compile as GetVariant<i32> is not implemented for Enum.
/// ```
/// Works similarly for get_variant_mut if instance is mutable; returns mutable references instead.
fn impl_get_variant(ast: &DeriveInput, types: &HashMap<&Ident, &Ident>) -> TokenStream {
    let name = &ast.ident.to_string();
    let mut piece = String::new();
    for (_type, field_) in types.iter() {
        piece.push_str(&format!("impl GetVariant<{}> for {}", _type.to_string(), name));
        piece.push_str(" { ");
        piece.push_str(&format!(" fn get_variant(&self) -> Result<&{}, VariantAccessError>",
                                _type.to_string()));
        piece.push_str("{  match self { ");
        piece.push_str(&format!("{}::{}(inner) => Ok(inner), ",name, field_));
        piece.push_str(&format!("_ => Err(VariantAccessError::wrong_active_field(\"{}\", \"{}\"))",
                                name, _type.to_string()));
        piece.push_str("} } ");

        piece.push_str(&format!(" fn get_variant_mut(&mut self) -> Result<&mut {}, VariantAccessError>",
                                _type.to_string()));
        piece.push_str("{  match self { ");
        piece.push_str(&format!("{}::{}(inner) => Ok(inner), ", name, field_));
        piece.push_str(&format!("_ => Err(VariantAccessError::wrong_active_field(\"{}\", \"{}\"))",
                                name, _type.to_string()));
        piece.push_str("} } }");
    }

    return piece.parse().unwrap();
}

/// Implements the SetVariant trait that sets the
/// tagged value of the field whose type matches the input value, if possible
///
/// # Example:
/// ```
/// enum Enum {
///     F1(i64),
///     F2(bool)
/// }
/// let mut instance = Enum::F1(42);
///
/// instance.set_variant(false); // instance now is equal to Enum::F2(false)
/// // instance.set_variant(""); will not compile as Enum has not field of type &str
/// ```
/// This method uses type inference to try and determine which field to use. However this can
/// be ambiguuous sometimes.
///
/// # Example:
/// ```
/// enum Enum {
///     F1(i32),
///     F2(i64)
/// }
///
/// let mut instance = Enum::F1(42);
/// instance.set_variante(1); // Is instance equal to Enum::F1(1) or Enum::F2(1) ???
///
/// // Do this instead
/// instance.set_variant(1 as i32); // instance equals Enum::F1(1)
/// instance.set_variant(1 as i64); // instance equal Enum::F2(1)
/// ```
fn impl_set_variant(ast: &DeriveInput, types: &HashMap<&Ident, &Ident>) -> TokenStream {
    let name = &ast.ident.to_string();
    let mut piece = String::new();
    for (_type, field_) in types.iter() {
        piece.push_str(&format!("impl SetVariant<{}> for {}", _type.to_string(), name));
        piece.push_str(" { ");
        piece.push_str(&format!(" fn set_variant(&mut self, value: {})",
                                _type.to_string()));
        piece.push_str("{ ");
        piece.push_str(&format!("*self = {}::{}(value);", name, field_));
        piece.push_str("} } ");

    }
    return piece.parse().unwrap()
}
/// Implements both the ContainsVariant and GetVariant traits
fn impl_variant_access(ast: &DeriveInput) -> TokenStream {
    let mut tokens: TokenStream = "".parse().unwrap();
    let types = fetch_types_from_enum(ast);

    tokens.extend::<TokenStream>(impl_contains_variant(&ast, &types));
    tokens.extend::<TokenStream>(impl_get_variant(&ast, &types));
    tokens.extend::<TokenStream>(impl_set_variant(&ast, &types));
    tokens
}

