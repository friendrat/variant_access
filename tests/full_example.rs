use variant_access_traits::*;

/// This hand written example closely resembles the code produced by the derive macro.
/// It has been included to demonstrate how the traits work under the hood and perhaps
/// inspire others in how they might write these traits themselves.

enum Enum {
    F1(i64),
    F2(f64)
}

fn is_integral<T: 'static>() -> bool {
    std::any::TypeId::of::<T>() == std::any::TypeId::of::<i64>()
}

fn is_float<T: 'static>() -> bool {
    std::any::TypeId::of::<f64>() == std::any::TypeId::of::<T>()
}

impl ContainsVariant for Enum {
    fn has_variant<T: 'static>(&self) -> bool {
        is_integral::<T>() || is_float::<T>()
    }
    fn contains_variant<T: 'static>(&self) -> Result<bool, VariantAccessError> {
        if self.has_variant::<T>() {
            match &self {
                Enum::F1(_) => Ok(is_integral::<T>()),
                Enum::F2(_) => Ok(is_float::<T>()),
            }
        } else {
            Err(VariantAccessError::invalid_type("Enum", std::any::type_name::<T>()))
        }
    }
}

impl GetVariant<i64> for Enum {
    fn get_variant(&self) -> Result<&i64, VariantAccessError> {
        match &self {
            Enum::F1(inner) => Ok(inner),
            Enum::F2(_) => Err(VariantAccessError::wrong_active_field("Enum", "i64"))
        }
    }

    fn get_variant_mut(&mut self) -> Result<&mut i64, VariantAccessError> {
        match self {
            Enum::F1(inner) => Ok(inner),
            Enum::F2(_) => Err(VariantAccessError::wrong_active_field("Enum", "i64"))
        }
    }
}

impl GetVariant<f64> for Enum {
    fn get_variant(&self) -> Result<&f64, VariantAccessError> {
        match &self {
            Enum::F2(inner) => Ok(inner),
            Enum::F1(_) => Err(VariantAccessError::wrong_active_field("Enum", "i64"))
        }
    }

    fn get_variant_mut(&mut self) -> Result<&mut f64, VariantAccessError> {
        match self {
            Enum::F2(inner) => Ok(inner),
            Enum::F1(_) => Err(VariantAccessError::wrong_active_field("Enum", "i64"))
        }
    }
}

impl SetVariant<i64> for Enum {
    fn set_variant(&mut self, value: i64) {
        *self = Enum::F1(value);
    }
}

impl SetVariant<f64> for Enum {
    fn set_variant(&mut self, value: f64) {
        *self = Enum::F2(value);
    }
}

impl CreateVariantFrom<i64> for Enum {
    fn create_variant_from(value: i64) -> Enum {
        Enum::F1(value)
    }
}

impl CreateVariantFrom<f64> for Enum {
    fn create_variant_from(value: f64) -> Enum {
        Enum::F2(value)
    }
}

fn main () {

}
