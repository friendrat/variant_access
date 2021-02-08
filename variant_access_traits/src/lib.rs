use std::{error::Error, fmt};

/// Custom errors for this crate. Keeps a record of the enum and requested type that produced
/// the error
#[derive(Debug)]
pub struct VariantAccessError {
    enum_name : String,
    requested_type: String,
    msg: String
}

impl VariantAccessError {

    /// Makes the appropriate error message for when get_variant fails
    pub fn wrong_active_field(type_name: &str, requested_type: &str) -> VariantAccessError {
        VariantAccessError{
            enum_name: type_name.to_string(),
            requested_type: requested_type.to_string(),
            msg: format!("Active filed of enum <{}> is not of requested type <{}>",
                         type_name, requested_type)
        }
    }

    /// Makes the appropriate error message for when has_variant fails
    pub fn invalid_type(type_name: &str, requested_type: &str) -> VariantAccessError {
        VariantAccessError{
            enum_name: type_name.to_string(),
            requested_type: requested_type.to_string(),
            msg: format!("Requested type <{}> does not match the type of any field \
                           in enum <{}>",  requested_type, type_name)
        }
    }
}
impl Error for VariantAccessError { }

impl fmt::Display for VariantAccessError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VariantAccessError :: {}", self.msg)
    }
}

/// This trait is for querying an enum.
///
/// has_variant is intended to determine if one of the variant fields is of the specified type.
/// contains_variants is intended to determine if the active field is of the specified type. If none
/// of the fields is of specified type, it is intended that an Err be returned.
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
pub trait ContainsVariant {
    fn has_variant<T: 'static>(&self) -> bool;
    fn contains_variant<T: 'static>(&self) -> Result<bool, VariantAccessError>;
}


/// This trait is for extracting a reference to the raw values in an enum
///
/// get_variant returns a reference to the raw value of the active field if it has the same type
/// as the specified type. Otherwise, an Err should be returned. It is intended to use this function
/// in conjunction with has_variant / contains_variant to know that safe unwrapping can occur.
///
/// get_variant_mut is similar except it is for returning a mutable reference to the raw value of the
/// active field.
///
/// # Example:
/// ```
/// #[derive(VariantAccess)]
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
pub trait GetVariant<T, Marker> {
    fn get_variant(&self) -> Result<&T, VariantAccessError>;
    fn get_variant_mut(&mut self) -> Result<&mut T, VariantAccessError>;
}

/// This trait is for setting an inner value with the correct associated type to the given value
///
/// set_variant looks at the type of the parameter value and if one of the fields has this type,
/// the enums active field is changed to this field with inner value set to given parameter.
///
/// This method is guaranteed to succeed in the sense that if an value is given whose type does not
/// match the type of any field, the program will not compile.
/// # Example:
/// ```
/// enum Enum {
///     F1(i64),
///     F2(bool)
/// }
/// let mut instance = Enum::F1(42);
///
/// instance.set_variant(false); // instance now is equal to Enum::F2(false)
/// // instance.set_variant(""); will not compile as Enum has no field of type &str
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
pub trait SetVariant<T, Marker> {
    fn set_variant(&mut self, value: T);
}
