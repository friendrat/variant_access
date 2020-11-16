/// This trait is for querying an enum.
///
/// has_variant is intended to determine if one of the variant fields is of the specified type
/// contains_variants is intended to determine if the active field is of the specified type. If none
/// of the fields is of specified type, it is intended that an Err be returned.
pub trait ContainsVariant {
    fn has_variant<T>(&self) -> bool;
    fn contains_variant<T>(&self) -> Result<bool, ()>;

}


/// This trait is for extracting a reference to the raw values in an enum
///
/// get_variant returns a reference to the raw value of the active field if it has the same type
/// as the specified type. Otherwise, an Err should be returned. It is intended to use this function
/// in conjunction with has_variant / contains_variant to know that safe unwrapping can occur.
///
/// get_variant_mut is similar except it is for return a mutable reference to the raw value of the
/// active field.
pub trait GetVariant<T> {
    fn get_variant(&self) -> Result<&T, ()>;
    fn get_variant_mut(&mut self) -> Result<&mut T, ()>;
}

/// This trait is for setting an inner value with the correct associated type to the given value
///
/// set_variant looks at the type of the parameter value and if one of the fields has this type,
/// the enums active field is changed to this field with inner value set to given parameter.
///
/// This method is guaranteed to succeed in the sense that if an value is given whose type does not
/// match the type of any field, the program will not compile.
pub trait SetVariant<T> {
    fn set_variant(&mut self, value: T);
}
