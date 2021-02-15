

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

impl variant_access_traits::ContainsVariant for Enum {
    fn has_variant<T: 'static>(&self) -> bool {
        is_integral::<T>() || is_float::<T>()
    }
    fn contains_variant<T: 'static>(&self) -> Result<bool, variant_access_traits::VariantAccessError> {
        if self.has_variant::<T>() {
            match &self {
                Enum::F1(_) => Ok(is_integral::<T>()),
                Enum::F2(_) => Ok(is_float::<T>()),
            }
        } else {
            Err(variant_access_traits::VariantAccessError::invalid_type("Enum", std::any::type_name::<T>()))
        }
    }
}

impl variant_access_traits::GetVariant<i64> for Enum {
    fn get_variant(&self) -> Result<&i64, variant_access_traits::VariantAccessError> {
        match &self {
            Enum::F1(inner) => Ok(inner),
            Enum::F2(_) => Err(variant_access_traits::VariantAccessError::wrong_active_field("Enum", "i64"))
        }
    }

    fn get_variant_mut(&mut self) -> Result<&mut i64, variant_access_traits::VariantAccessError> {
        match self {
            Enum::F1(inner) => Ok(inner),
            Enum::F2(_) => Err(variant_access_traits::VariantAccessError::wrong_active_field("Enum", "i64"))
        }
    }
}

impl variant_access_traits::GetVariant<f64> for Enum {
    fn get_variant(&self) -> Result<&f64, variant_access_traits::VariantAccessError> {
        match &self {
            Enum::F2(inner) => Ok(inner),
            Enum::F1(_) => Err(variant_access_traits::VariantAccessError::wrong_active_field("Enum", "i64"))
        }
    }

    fn get_variant_mut(&mut self) -> Result<&mut f64, variant_access_traits::VariantAccessError> {
        match self {
            Enum::F2(inner) => Ok(inner),
            Enum::F1(_) => Err(variant_access_traits::VariantAccessError::wrong_active_field("Enum", "i64"))
        }
    }
}

impl variant_access_traits::SetVariant<i64> for Enum {
    fn set_variant(&mut self, value: i64) {
        *self = Enum::F1(value);
    }
}

impl variant_access_traits::SetVariant<f64> for Enum {
    fn set_variant(&mut self, value: f64) {
        *self = Enum::F2(value);
    }
}

impl variant_access_traits::CreateVariantFrom<i64> for Enum {
    fn create_variant_from(value: i64) -> Enum {
        Enum::F1(value)
    }
}

impl variant_access_traits::CreateVariantFrom<f64> for Enum {
    fn create_variant_from(value: f64) -> Enum {
        Enum::F2(value)
    }
}

fn main () {

}
