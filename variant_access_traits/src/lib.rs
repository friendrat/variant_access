
pub trait HasVariant {
    fn has_variant<T>(&self) -> bool;
}

pub trait ContainsVariant {
    fn contains_variant<T>(&self) -> Result<bool, ()>;
}

pub trait GetVariant<T> {
    fn get_variant(&self) -> Result<&T, ()>;
}
