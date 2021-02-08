use variant_access_traits::{ContainsVariant, GetVariant, SetVariant, VariantAccessError};
use variant_access_derive::*;


#[cfg(test)]
mod test_basic {

    use super::*;
    #[derive(VariantAccess, PartialEq, Debug)]
    enum Test {
        F1(i32),
        F2(bool)
    }

    #[derive(VariantAccess, PartialEq, Debug)]
    enum AmbiguousTest {
        F1(i32),
        F2(i64)
    }


    #[test]
    fn test_has_variant() {
        let test = Test::F1(42);
        assert!(test.has_variant::<i32>());
        assert!(test.has_variant::<bool>());
        assert!(!test.has_variant::<i64>());
    }

    #[test]
    fn test_contains_variant(){
        let test = Test::F1(42);
        assert!(test.contains_variant::<i32>().expect("Expected contains_variant to return boolean;\
                                                   exception encountered instead"));
        assert!(!test.contains_variant::<bool>().expect("Expected contains_variant to return boolean;\
                                                   exception encountered instead"));
        let test = Test::F2(false);
        assert!(!test.contains_variant::<i32>().expect("Expected contains_variant to return boolean;\
                                                   exception encountered instead"));
        assert!(test.contains_variant::<bool>().expect("Expected contains_variant to return boolean;\
                                                   exception encountered instead"));
    }

    #[test]
    fn test_contains_variant_error() {
        let test = Test::F1(42);
        let _ = test.contains_variant::<i64>().expect_err("Expected contains_variant to return Err!");
    }

    #[test]
    fn test_get_variant() {
        let test = Test::F1(42);
        let test_inner_value: &i32 = test.get_variant().expect("Expected get_variant to return value");
        assert_eq!(test_inner_value, &42);

        let test= Test::F2(false);
        let test_inner_value: &bool = test.get_variant().expect("Expected get_variant to return value");
        assert_eq!(test_inner_value, &false);
    }

    #[test]
    #[should_panic]
    fn test_get_variant_error_from_wrong_variant()  {
        let test = Test::F1(42);
        let _: &bool = test.get_variant().expect("");
    }


    #[test]
    fn test_get_variant_mut() {
        let mut test = Test::F1(42);
        let inner: &mut i32 = test.get_variant_mut().expect("Expected get_variant_mut to return value");
        *inner = 1;
        assert_eq!(test, Test::F1(1));
    }

    #[test]
    #[should_panic]
    fn test_get_variant_mut_error_from_wrong_variant()  {
        let mut test = Test::F1(42);
        let _: &mut bool = test.get_variant_mut().expect("");
    }

    #[test]
    fn test_set_variant(){
        let mut test = Test::F2(false);
        test.set_variant(42);
        assert_eq!(test, Test::F1(42));
    }

    #[test]
    fn test_set_variant_ambiguous(){
        let mut test = AmbiguousTest::F1(42);
        test.set_variant(42 as i64);
        assert_eq!(test, AmbiguousTest::F2(42));
    }
}

#[cfg(test)]
mod test_namespaces {
    use super::*;
    #[derive(Debug, PartialEq)]
    pub struct Complex {
        field_one: bool,
        field_two: f64
    }

    pub mod namespace {
        use super::*;
        #[derive(Debug, PartialEq)]
        pub struct Complex {
            pub field_one: bool,
            pub field_two: f64
        }

        #[derive(VariantAccess, PartialEq, Debug)]
        pub enum ComplexEnum {
            F1(Complex),
            F2(super::Complex)
        }
    }

    #[test]
    /// This test checks that different ways of specifying the same namespace does not affect the
    /// correctness of the result
    fn test_correct_namespace_resolution() {

        let complex = namespace::ComplexEnum::F1(namespace::Complex{field_one: true, field_two: 2.0});
        assert!(complex.has_variant::<namespace::Complex>());
        let value = complex.contains_variant::<namespace::Complex>().expect("Test failed");
        assert_eq!(value, true);

        assert!(complex.has_variant::<Complex>());
        assert!(complex.has_variant::<super::test_namespaces::Complex>());
        let value = complex.contains_variant::<Complex>().expect("Test failed");
        assert_eq!(value, false);
        let value = complex.contains_variant::<super::test_namespaces::Complex>().expect("Test failed");
        assert_eq!(value, false);
    }
}

#[cfg(test)]
/// Tests that the derive macro correctly panics (thereby failing compilation) for the correct
/// cases.
mod test_compile_failures {
    #[test]
    fn test_uncompilable_examples() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/uncompilable_examples/bad_get_variant_type.rs");
        t.compile_fail("tests/uncompilable_examples/enum_with_named_subfields.rs");
        t.compile_fail("tests/uncompilable_examples/enum_with_tuple_field.rs");
        t.compile_fail("tests/uncompilable_examples/input_not_enum.rs");
        t.compile_fail("tests/uncompilable_examples/twice_used_field_type.rs");
    }
}

#[cfg(test)]
mod test_template_types {
    use super::*;
    use std::fmt::Debug;

    #[derive(VariantAccess, PartialEq, Debug)]
    pub enum CRAZY<Y: 'static, X: 'static> {
        F1(Y),
        F2(X)
    }


    #[test]
    fn test_it_works() {

    }
}
