extern crate proc_macro;

#[allow(unused_imports)]
use std::any::type_name;
#[allow(unused_imports)]
use std::iter::Enumerate;

use std::collections::HashMap;
use proc_macro::TokenStream;
use syn::{self, Ident, Data, DeriveInput, GenericParam};
use quote::{ToTokens};


#[proc_macro_derive(VariantAccess)]
pub fn variant_access_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_variant_access(&ast)
}

/// If the decorated enum has generic template parameters, we determine those here.
/// We also validate that there are no lifetime parameters. If there are, the
/// macro panics.
///
/// Otherwise, the full declaration of the type and a vector of the generic parameters is returned.
///
/// # Example
/// ```
/// #[derive(VariantAccess)]
/// enum Enum<T: 'static, X: 'static> {
///     F1(T),
///     F2(X)
/// }
/// ```
/// This function then returns `(Enum<T, X>, vec!("T", "X"))`
///
/// # Example
/// ```
/// #[derive(VariantAccess)]
/// enum Enum<'a, T: 'a, X> {
///     F1(T),
///     F2(X)
/// }
/// ```
/// This function panics on the above example.
fn fetch_name_with_generic_params(ast: &DeriveInput) -> (String, Vec<String>) {
    let mut param_string = String::from("");
    let params: Vec<String> = ast.generics.params
        .iter()
        .filter_map(|param|
            match param {
                GenericParam::Lifetime(inner) => {
                    if inner.lifetime.ident.to_token_stream().to_string() != String::from("static") {
                        panic!("VariantAccess can only be derived for types with static lifetimes");
                    } else {
                        None
                    }
                }
                GenericParam::Type(inner) => {
                    param_string.push_str(&format!("{},", inner.ident.to_string()));
                    Some(inner.ident.to_string())
                },
                GenericParam::Const(inner) => {
                    param_string.push_str(&format!("{},", inner.ident.to_string()));
                    Some(inner.ident.to_string())
                }
            })
        .collect();
    param_string.pop();
    if !params.is_empty() {
        (format!("{}<{}>", ast.ident, param_string), params)
    } else {
        (ast.ident.to_string(), params)
    }

}

/// Determines the full path of a type including all of its nested namespaces.
/// This is used later to ensure that the derive macro can work with full
/// namespace resolution.
///
/// # Exmample
/// ```
/// pub struct Complex {
///     field_one: bool,
///     field_two: f64
/// }
///
/// pub mod namespace {
///     use super::*;
///
///     pub struct Complex<T> {
///         pub field_one: bool,
///         pub field_two: T
///     }
///
///     #[derive(VariantAccess)]
///     pub enum ComplexEnum<T> {
///         F1(Complex<T>),
///         F2(super::Complex)
///     }
/// }
///```
/// The path name of the type of `F1` is `namespace::Complex<T>` and the path name of the type of
/// `F2` is `Complex`.
fn parse_path(path: &syn::Path) -> String {
    let mut fullname = String::from("");
    let _ = path.segments.pairs()
        .map(|segment| {
            fullname.push_str(&segment.to_token_stream().to_string())
        } )
        .collect::<()>();

    // remove extraneous whitespace
    fullname.retain(|c| c != ' ');
    fullname
}


/// Makes a map of the form < field type : field name >
///
/// Provides validation
///     -- that no field type is used twice.
///     -- that input is enum.
///     -- that all field types do not have named fields
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
/// panics in this function as F1 has multiple unnamed types or because F2 has a named field.
fn fetch_types_from_enum(ast: &DeriveInput) -> HashMap<String, &Ident> {
    let mut types: HashMap<String, &Ident> = HashMap::new();

    if let Data::Enum(data) = &ast.data {
        for var in data.variants.iter() {

            if let syn::Fields::Unnamed(field_) = &var.fields{
                if field_.unnamed.len() > 1 {
                    panic!("Can only derive for enums whose types do not contain multiple fields.");
                }

                for field_entry in field_.unnamed.iter() {
                    if let syn::Type::Path(type_) = &field_entry.ty {
                        if let Some(_) = types.insert(parse_path(&type_.path), &var.ident) {
                            panic!("Cannot derive VariantAccess for enum with multiple fields of same type");
                        }
                    }
                }
            } else {
                panic!("Cannot derive VariantAccess for enums whose types have named fields.")
            }
        }
    } else {
        panic!("Can only derive VariantAccess for enums.")
    };
    types
}

/// In order to avoid conflicting definitions of the GetVariant / SetVariant traits for
/// enums that are generic over more than one parameter, we use marker structs:
/// see https://stackoverflow.com/questions/52281091/can-i-avoid-eager-ambiguity-resolution-for-trait-implementations-with-generics/52692592#52692592
/// This function generates these structs and places them in a private module.
/// # Example
/// ```
/// #[derive(VariantAccess)]
/// enum Enum<X, Y> {
///     F1(X),
///     F2(Y)
/// }
/// ```
/// produces the following module
/// ```
/// #[allow(non_snake_case)]
/// mod variant_access_Enum {
///     pub (crate) struct F1;
///     pub (crate) struct F2;
/// }
/// ```
fn create_marker_structs(name: &str, types: &HashMap<String, &Ident>) -> TokenStream {
    let mut piece = format!("#[allow(non_snake_case)]\n mod variant_access_{}", name);
    piece.push_str("{ ");
    for field_ in types.values() {
        piece.push_str(&format!("pub (crate) struct {};", field_.to_string()));
    }
    piece.push_str("} ");
    piece.parse().unwrap()
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
fn impl_contains_variant(ast: &DeriveInput,
                         name: &str,
                         params: &[String],
                         types: &HashMap<String, &Ident>) -> TokenStream {
    // generic is a parameter name guaranteed not to be equal to the enum generic parameter names
    let (param_string, generic) = if !params.is_empty() {
        (format!("<{}>", ast.generics.params.to_token_stream()), params.concat())
    } else {
        (String::from(""), String::from("T"))
    };

    let mut piece : String = format!("impl{} ContainsVariant for {}", param_string, name);
    piece.push_str("{ ");
    piece.push_str(&format!(" fn has_variant<{}: 'static>(&self) -> bool ", &generic));
    piece.push_str("{ ");

    for (ix, type_) in types.keys().enumerate() {
        if ix == types.len() - 1 {
            piece.push_str(&format!("std::any::TypeId::of::<{}>()\
                                           == std::any::TypeId::of::<{}>()",
                                    &generic, type_));
        } else {
            piece.push_str(&format!("std::any::TypeId::of::<{}>() \
                                           == std::any::TypeId::of::<{}>() || ",
                                    &generic, type_));
        }
    }

    piece.push_str("} ");
    piece.push_str(&format!("fn contains_variant<{}: 'static>(&self) -> Result<bool, VariantAccessError>", &generic));
    piece.push_str(" { ");
    piece.push_str(&format!("if self.has_variant::<{}>()", &generic));
    piece.push_str("{ return match self { ");
    for (ix, (type_, field_)) in types.iter().enumerate() {
        piece.push_str(&format!("{}::{}(inner) => \
                                Ok(std::any::TypeId::of::<{}>() == std::any::TypeId::of::<{}>())",
                                ast.ident.to_string(), field_, type_, &generic));
        if ix != types.len() - 1 {
            piece.push_str(", ");
        }  else {
            piece.push_str("}; } ");
            piece.push_str( &format!(" Err(VariantAccessError::invalid_type(\"{}\", std::any::type_name::<{}>()))",
                                     name, &generic));
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
/// // let inner: &bool = instance.get_variant().unwrap() // panics because of unhandled Err.
/// // let inner: &i32 = instance.get_variant().unwrap() // will not compile as GetVariant<i32> is not implemented for Enum.
/// ```
/// Works similarly for get_variant_mut if instance is mutable; returns mutable references instead.
fn impl_get_variant(ast: &DeriveInput,
                    name: &str,
                    params: &[String],
                    types: &HashMap<String, &Ident>) -> TokenStream {

    let mut piece = String::new();
    for (type_, field_) in types.iter() {
        // Determines if we are implementing the trait over generics
        let param_string = if !params.is_empty() {
            format!("<{}>", ast.generics.params.to_token_stream())
        } else {
            String::from("")
        };
        // The template parameter + marker struct that we are implementing GetVariant for
        let template_params = format!("{}, variant_access_{}::{}",
                                      &type_,
                                      ast.ident.to_string(),
                                      field_.to_string());
        piece.push_str(&format!("impl{} GetVariant<{}> for {}", param_string, template_params, name));
        piece.push_str(" { ");
        piece.push_str(&format!(" fn get_variant(&self) -> Result<&{}, VariantAccessError>",
                                type_));
        piece.push_str("{  match self { ");
        piece.push_str(&format!("{}::{}(inner) => Ok(inner), ", ast.ident.to_string(), field_));
        piece.push_str(&format!("_ => Err(VariantAccessError::wrong_active_field(\"{}\", \"{}\"))",
                                name, type_));
        piece.push_str("} } ");

        piece.push_str(&format!(" fn get_variant_mut(&mut self) -> Result<&mut {}, VariantAccessError>",
                                type_));
        piece.push_str("{  match self { ");
        piece.push_str(&format!("{}::{}(inner) => Ok(inner), ", ast.ident.to_string(), field_));
        piece.push_str(&format!("_ => Err(VariantAccessError::wrong_active_field(\"{}\", \"{}\"))",
                                name, type_));
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
fn impl_set_variant(ast: &DeriveInput,
                    name: &str,
                    params: &[String],
                    types: &HashMap<String, &Ident>) -> TokenStream {

    let mut piece = String::new();
    for (type_, field_) in types.iter() {
        let param_string = if !params.is_empty() {
            format!("<{}>", ast.generics.params.to_token_stream())
        } else {
            String::from("")
        };
        // The template parameter + marker struct that we are implementing GetVariant for
        let template_params = format!("{}, variant_access_{}::{}",
                                      &type_,
                                      ast.ident.to_string(),
                                      field_.to_string());
        piece.push_str(&format!("impl{} SetVariant<{}> for {}", param_string, template_params, name));
        piece.push_str(" { ");
        piece.push_str(&format!(" fn set_variant(&mut self, value: {})",
                                type_));
        piece.push_str("{ ");
        piece.push_str(&format!("*self = {}::{}(value);", ast.ident.to_string(), field_));
        piece.push_str("} } ");

    }
    return piece.parse().unwrap()
}
/// Implements both ContainsVariant, GetVariant, and SetVariant traits
fn impl_variant_access(ast: &DeriveInput) -> TokenStream {
    let mut tokens: TokenStream = "".parse().unwrap();

    let (name, params) = fetch_name_with_generic_params(&ast);
    let types = fetch_types_from_enum(&ast);

    tokens.extend::<TokenStream>(create_marker_structs(&ast.ident.to_string(), &types));
    tokens.extend::<TokenStream>(impl_contains_variant(&ast, &name, &params, &types));
    tokens.extend::<TokenStream>(impl_get_variant(&ast, &name, &params, &types));
    tokens.extend::<TokenStream>(impl_set_variant(&ast, &name, &params, &types));
    tokens
}

