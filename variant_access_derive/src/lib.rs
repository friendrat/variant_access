mod parse_enum;
mod templates;

use crate::parse_enum::*;
use crate::templates::*;

extern crate proc_macro;

#[allow(unused_imports)]
use std::any::type_name;
#[allow(unused_imports)]
use std::iter::Enumerate;

use proc_macro::TokenStream;
use quote::ToTokens;
use std::collections::HashMap;
use syn::{self, Data, DeriveInput, GenericParam, Ident};
use tera::*;

#[proc_macro_derive(VariantAccess)]
pub fn variant_access_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_variant_access(&ast)
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
fn impl_contains_variant(
    ast: &DeriveInput,
    name: &str,
    params: &[String],
    types: &HashMap<String, &Ident>,
    templater: &Tera,
) -> TokenStream {
    // generic is a parameter name guaranteed not to be equal to the enum generic parameter names
    let (param_string, generic) = if !params.is_empty() {
        (
            format!("<{}>", ast.generics.params.to_token_stream()),
            params.concat(),
        )
    } else {
        (String::from(""), String::from("T"))
    };
    let mut context = Context::new();
    context.insert("generics", &param_string);
    context.insert("template", &generic);
    context.insert("fullname", &name);
    context.insert(
        "matches",
        &types
            .keys()
            .map(|type_| format!("std::any::TypeId::of::<{}>()", type_))
            .collect::<Vec<String>>(),
    );

    context.insert(
        "branches",
        &types
            .iter()
            .map(|(type_, field_)| {
                format!(
                    "{}::{}(_) => Ok(std::any::TypeId::of::<{}>()",
                    &ast.ident.to_string(),
                    field_.to_string(),
                    type_
                )
            })
            .collect::<Vec<String>>(),
    );

    let impl_string = templater
        .render("contains_variant", &context)
        .expect("Failed to fill in ContainsVariant template");

    impl_string.parse().unwrap()
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
fn impl_get_variant(
    ast: &DeriveInput,
    name: &str,
    params: &[String],
    types: &HashMap<String, &Ident>,
    templater: &Tera,
) -> TokenStream {
    let mut impl_string = String::new();
    // Determines if we are implementing the trait over generics
    let generics = if !params.is_empty() {
        format!("<{}>", ast.generics.params.to_token_stream())
    } else {
        String::from("")
    };
    for (type_, field_) in types.iter() {
        let mut context = Context::new();
        context.insert("generics", &generics);
        context.insert("Type", &type_);
        context.insert(
            "Marker",
            &format!(
                "variant_access_{}::{}",
                ast.ident.to_string(),
                field_.to_string()
            ),
        );
        context.insert("fullname", name);
        context.insert("name", &ast.ident.to_string());
        context.insert("field", &field_.to_string());
        impl_string.push_str(
            &templater
                .render("get_variant", &context)
                .expect("Failed to fill in GetVariant template"),
        );
    }

    impl_string.parse().unwrap()
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
fn impl_set_variant(
    ast: &DeriveInput,
    name: &str,
    params: &[String],
    types: &HashMap<String, &Ident>,
    templater: &Tera,
) -> TokenStream {
    let mut impl_string = String::new();
    let generics = if !params.is_empty() {
        format!("<{}>", ast.generics.params.to_token_stream())
    } else {
        String::from("")
    };
    for (type_, field_) in types.iter() {
        let mut context = Context::new();
        context.insert("generics", &generics);
        context.insert("Type", &type_);
        context.insert(
            "Marker",
            &format!(
                "variant_access_{}::{}",
                ast.ident.to_string(),
                field_.to_string()
            ),
        );
        context.insert("fullname", name);
        context.insert("name", &ast.ident.to_string());
        context.insert("field", &field_.to_string());
        impl_string.push_str(
            &templater
                .render("set_variant", &context)
                .expect("Failed to fill in GetVariant template"),
        );
    }
    impl_string.parse().unwrap()
}

/// This trait allows one to create a new instance of an enum from a value whose type matches one
/// of the types of the field of the enum.
/// # Example:
///```
/// use variant_access_traits::*;
/// use variant_access_derive::*;
///
/// enum Enum {
///     F1(i64),
///     F2(bool)
/// }
///
/// let instance = Enum::create_from(false); // instance is now equal to Enum::F2(false)
/// // let instance = Enum::create_from("") // will not compile as Enum has no field of type &str
///```
/// Similar to the `SetVariant`, the `as` keyword should be used in the case of ambiguous typing.
///
/// This trait has a generic paramer `Marker` for adding marker structs. This is used if implementing
/// this trait for enums with more than one generic parameter in order to avoid definition clashes.
fn impl_create_variant(
    ast: &DeriveInput,
    name: &str,
    params: &[String],
    types: &HashMap<String, &Ident>,
    templater: &Tera,
) -> TokenStream {
    let mut impl_string = String::new();
    let generics = if !params.is_empty() {
        format!("<{}>", &ast.generics.params.to_token_stream())
    } else {
        String::from("")
    };
    for (type_, field_) in types.iter() {
        let mut context = Context::new();
        context.insert("generics", &generics);
        context.insert("Type", &type_);
        context.insert(
            "Marker",
            &format!(
                "variant_access_{}::{}",
                ast.ident.to_string(),
                field_.to_string()
            ),
        );
        context.insert("fullname", name);
        context.insert("name", &ast.ident.to_string());
        context.insert("field", &field_.to_string());
        impl_string.push_str(
            &templater
                .render("create_variant", &context)
                .expect("Failed to fill in CreateVariantFrom template"),
        );
    }
    impl_string.parse().unwrap()
}

/// Implements ContainsVariant, GetVariant, SetVariant, and CreateVariantFrom traits
fn impl_variant_access(ast: &DeriveInput) -> TokenStream {
    let mut tera = Tera::new("/dev/null/*").unwrap();
    tera.add_raw_template("contains_variant", CONTAINS_VARIANT_TEMPLATE)
        .unwrap();
    tera.add_raw_template("get_variant", GET_VARIANT_TEMPLATE)
        .unwrap();
    tera.add_raw_template("set_variant", SET_VARIANT_TEMPLATE)
        .unwrap();
    tera.add_raw_template("create_variant", CREATE_VARIANT_TEMPLATE)
        .unwrap();
    let mut tokens: TokenStream = "".parse().unwrap();

    let (name, params) = fetch_name_with_generic_params(&ast);
    let types = fetch_types_from_enum(&ast);

    tokens.extend::<TokenStream>(create_marker_structs(&ast.ident.to_string(), &types));
    tokens.extend::<TokenStream>(impl_contains_variant(&ast, &name, &params, &types, &tera));
    tokens.extend::<TokenStream>(impl_get_variant(&ast, &name, &params, &types, &tera));
    tokens.extend::<TokenStream>(impl_set_variant(&ast, &name, &params, &types, &tera));
    tokens.extend::<TokenStream>(impl_create_variant(&ast, &name, &params, &types, &tera));
    tokens
}
