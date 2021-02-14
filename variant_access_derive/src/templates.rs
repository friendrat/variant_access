pub (crate) const CONTAINS_VARIANT_TEMPLATE: &str = r#"
impl{{ generics }} ContainsVariant for {{ fullname }} {
    fn has_variant<{{ template }} : 'static>(&self) -> bool {
        {%- for M in matches %}
        if std::any::TypeId::of::<{{ template }}>() == {{ M }} {
            return true;
        }
        {%- endfor %}
        false
    }

    fn contains_variant<{{ template }} : 'static>(&self) -> Result<bool, VariantAccessError> {
        if self.has_variant::<{{ template }}>() {
            match self {
                {%- for B in branches %}
                {{ B }} == std::any::TypeId::of::<{{ template }}>()),
                {%- endfor %}
                _ => Ok(false)
            }
        } else {
            Err(VariantAccessError::invalid_type("{{ fullname }}", std::any::type_name::<{{ template }}>()))
        }
    }
}
"#;

pub (crate) const GET_VARIANT_TEMPLATE: &str = r#"
impl{{ generics }} GetVariant<{{ Type }}, {{ Marker }} > for {{ fullname }} {
    fn get_variant(&self) -> Result<&{{ Type }}, VariantAccessError> {
        match &self {
            {{ name }}::{{ field }}(inner) => Ok(inner),
            _ => Err(VariantAccessError::wrong_active_field("{{ fullname }}", "{{ Type }}"))
        }
    }

    fn get_variant_mut(&mut self) -> Result<&mut {{ Type }}, VariantAccessError> {
        match self {
            {{ name }}::{{ field }}(inner) => Ok(inner),
            _  => Err(VariantAccessError::wrong_active_field("{{ fullname }}", "{{ Type }}"))
        }
    }
}"#;

pub (crate) const SET_VARIANT_TEMPLATE: &str = r#"
impl{{ generics }} SetVariant<{{ Type }}, {{ Marker }} > for {{ fullname }} {
    fn set_variant(&mut self, value: {{ Type }}) {
        *self = {{ name }}::{{ field }}(value);
    }
}"#;

pub (crate) const CREATE_VARIANT_TEMPLATE: &str = r#"
impl{{ generics }} CreateVariantFrom<{{ Type }}, {{ Marker }}> for {{ fullname }} {
    fn create_variant_from(value : {{ Type }}) -> Self {
        {{ name }}::{{ field }}(value)
    }
}
"#;