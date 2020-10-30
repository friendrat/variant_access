
pub trait HasVariant {
    fn has_variant<T>(&self) -> Result<bool, ()>;
}

pub trait GetVariant<T> {
    fn get_variant(&self) -> Result<&T, ()>;
}


/*
fn type_of<T>(_: T) -> &'static str {
    type_name::<T>()
}
pub fn in_variant<T>() -> bool {
    type_name::<T>() == type_name::<i64>() || type_name::<T>() == type_name::<bool>()
}

/// Variant definition
enum VariantIntBool {
    Int(i64),
    Bool(bool)
}


impl HasVariant for VariantIntBool{
    fn has_variant<T>(&self) -> Result<bool, ()>{

        if in_variant::<T>() {
            return match self {
                VariantIntBool::Int(inner) => Ok(type_of(*inner) == type_name::<T>()),
                VariantIntBool::Bool(inner) => Ok(type_of(*inner) == type_name::<T>())

            };
        }
        Err(())
    }
}

impl GetVariant<i64> for VariantIntBool {
    fn get_variant(&self) -> Result<&i64, ()> {
        match self {
            VariantIntBool::Int(inner) => Ok(inner),
            _ => Err(())
        }
    }
}

impl GetVariant<bool> for VariantIntBool {
    fn get_variant(&self) -> Result<&bool, ()> {
        match self {
            VariantIntBool::Bool(inner) => Ok(inner),
            _ => Err(())
        }
    }
}

fn common_type_name<T>() -> Result<String, ()> {
    match std::any::type_name::<T>() {
        "()" => Ok("Null".to_string()),
        "bool" => Ok("Bool".to_string()),
        "i32" => Ok("Int".to_string()),
        "i64" => Ok("Long".to_string()),
        "f32" => Ok("Float".to_string()),
        "f64" => Ok("Double".to_string()),
        "&str" | "String" => Ok("String".to_string()),
        "u8" => Ok("Byte".to_string()),
        _ => Err(())
    }
}

fn common_name(type_str: &str) -> Result<String, ()> {
    match type_str {
        "()" => Ok("Null".to_string()),
        "bool" => Ok("Bool".to_string()),
        "i32" => Ok("Int".to_string()),
        "i64" => Ok("Long".to_string()),
        "f32" => Ok("Float".to_string()),
        "f64" => Ok("Double".to_string()),
        "&str" | "String" => Ok("String".to_string()),
        "u8" => Ok("Byte".to_string()),
        _ => Err(())
    }
}
macro_rules! declare_variant {
    ($name:ident, ($($field:ident),+), $($t:ty),+) => {
        #[derive(Debug)]
        enum $name {
            $(
                $field($t),
            )+

        }
    }
}

fn main() {
    let x = VariantIntBool::Bool(true);
    let y = x.has_variant::<bool>();

    if match y {
        Ok(inner) => inner,
        _ => {println!("This variant does not have a field of requested type"); false}
    } {
        let z: Result<&&str, ()> = x.get_variant();
    }


}*/