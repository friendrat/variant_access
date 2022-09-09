use variant_access_derive::*;
use variant_access_traits::*;

#[cfg(test)]
mod test_basic {

    use super::*;
    #[derive(VariantAccess, PartialEq, Debug)]
    enum Test {
        F1(i32),
        F2(bool),
    }

    #[derive(VariantAccess, PartialEq, Debug)]
    enum AmbiguousTest {
        F1(i32),
        F2(i64),
    }

    #[derive(Debug, PartialEq)]
    struct TestStruct {
        field: Test,
    }

    #[derive(Debug, PartialEq)]
    struct AmbiguousStruct {
        field: AmbiguousTest,
    }

    #[test]
    fn test_has_variant() {
        let test = Test::F1(42);
        assert!(test.has_variant::<i32>());
        assert!(test.has_variant::<bool>());
        assert!(!test.has_variant::<i64>());
    }

    #[test]
    fn test_contains_variant() {
        let test = Test::F1(42);
        assert!(test.contains_variant::<i32>().expect("Test failed"));
        assert!(!test.contains_variant::<bool>().expect("Test failed"));
        let test = Test::F2(false);
        assert!(!test.contains_variant::<i32>().expect("Test failed"));
        assert!(test.contains_variant::<bool>().expect("Test failed"));
    }

    #[test]
    fn test_contains_variant_error() {
        let test = Test::F1(42);
        let _ = test.contains_variant::<i64>().expect_err("Test failed");
    }

    #[test]
    fn test_get_variant() {
        let test = Test::F1(42);
        let test_inner_value: i32 = test.get_variant().expect("Test failed");
        assert_eq!(test_inner_value, 42);

        let test = Test::F2(false);
        let test_inner_value: bool = test.get_variant().expect("Test failed");
        assert!(!test_inner_value);
    }

    #[test]
    fn test_get_variant_ref() {
        let test = Test::F1(42);
        let test_inner_value: &i32 = test.get_variant_ref().expect("Test failed");
        assert_eq!(test_inner_value, &42);

        let test = Test::F2(false);
        let test_inner_value: &bool = test.get_variant_ref().expect("Test failed");
        assert_eq!(test_inner_value, &false);
    }

    #[test]
    #[should_panic]
    fn test_get_variant_error_from_wrong_variant() {
        let test = Test::F1(42);
        let _: bool = test.get_variant().expect("");
    }

    #[test]
    #[should_panic]
    fn test_get_variant_ref_error_from_wrong_variant() {
        let test = Test::F1(42);
        let _: &bool = test.get_variant_ref().expect("");
    }

    #[test]
    fn test_get_variant_mut() {
        let mut test = Test::F1(42);
        let inner: &mut i32 = test.get_variant_mut().expect("Test failed");
        *inner = 1;
        assert_eq!(test, Test::F1(1));
    }

    #[test]
    #[should_panic]
    fn test_get_variant_mut_error_from_wrong_variant() {
        let mut test = Test::F1(42);
        let _: &mut bool = test.get_variant_mut().expect("");
    }

    #[test]
    fn test_set_variant() {
        let mut test = Test::F2(false);
        test.set_variant(42);
        assert_eq!(test, Test::F1(42));
    }

    #[test]
    fn test_set_variant_ambiguous() {
        let mut test = AmbiguousTest::F1(42);
        test.set_variant(42_i64);
        assert_eq!(test, AmbiguousTest::F2(42));
    }

    #[test]
    fn test_trait_create_variant_from() {
        let test = Test::create_variant_from(2);
        assert_eq!(test, Test::F1(2));
    }

    #[test]
    fn test_trait_create_variant_from_ambiguous() {
        let test = AmbiguousTest::create_variant_from(42_i64);
        assert_eq!(test, AmbiguousTest::F2(42));
    }

    #[test]
    fn test_func_create_variant_from() {
        let test = TestStruct {
            field: create_variant_from(2),
        };
        assert_eq!(test, TestStruct { field: Test::F1(2) });
    }

    #[test]
    fn test_func_create_variant_from_ambiguous() {
        let test = AmbiguousStruct {
            field: create_variant_from(42_i64),
        };
        assert_eq!(
            test,
            AmbiguousStruct {
                field: AmbiguousTest::F2(42)
            }
        );
    }
}

#[cfg(test)]
mod test_namespaces {
    use super::*;
    #[derive(Debug, PartialEq)]
    pub struct Complex {
        field_one: bool,
        field_two: f64,
    }

    pub mod namespace {
        use super::*;
        #[derive(Debug, PartialEq)]
        pub struct Complex {
            pub field_one: bool,
            pub field_two: f64,
        }

        #[derive(VariantAccess, PartialEq, Debug)]
        pub enum ComplexEnum {
            F1(Complex),
            F2(super::Complex),
        }
    }

    #[test]
    /// This test checks that different ways of specifying the same namespace does not affect the
    /// correctness of the result
    fn test_correct_namespace_resolution() {
        let complex = namespace::ComplexEnum::F1(namespace::Complex {
            field_one: true,
            field_two: 2.0,
        });
        assert!(complex.has_variant::<namespace::Complex>());
        let value = complex
            .contains_variant::<namespace::Complex>()
            .expect("Test failed");
        assert!(value);

        assert!(complex.has_variant::<Complex>());
        assert!(complex.has_variant::<super::test_namespaces::Complex>());
        let value = complex.contains_variant::<Complex>().expect("Test failed");
        assert!(!value);
        let value = complex
            .contains_variant::<super::test_namespaces::Complex>()
            .expect("Test failed");
        assert!(!value);
    }
}


#[cfg(test)]
/// Tests that the derive macro correctly panics (thereby failing compilation) for the correct
/// cases.
mod test_compile_failures {
    #[test]
    fn test_uncompilable_examples() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/uncompilable_examples/bad_func_create_variant_from_type.rs");
        t.compile_fail("tests/uncompilable_examples/bad_get_variant_type.rs");
        t.compile_fail("tests/uncompilable_examples/bad_trait_create_variant_from_type.rs");
        t.compile_fail("tests/uncompilable_examples/enum_with_named_subfields.rs");
        t.compile_fail("tests/uncompilable_examples/enum_with_tuple_field.rs");
        t.compile_fail("tests/uncompilable_examples/get_wrong_variant_generics.rs");
        t.compile_fail("tests/uncompilable_examples/input_not_enum.rs");
        t.compile_fail("tests/uncompilable_examples/non_static_lifetime_parameter.rs");
        t.compile_fail("tests/uncompilable_examples/non_static_templates.rs");
        t.compile_fail("tests/uncompilable_examples/twice_used_field_type.rs");
        t.compile_fail("tests/uncompilable_examples/type_does_not_implement_debug.rs");
        t.compile_fail("tests/uncompilable_examples/type_does_not_implement_partialeq.rs");
    }
}

#[cfg(test)]
/// A hand coded example that closely resembles the code produced by the derive macro has been
/// included. This test ensures its correctness.
mod test_handwritten_example {
    #[test]
    fn test_handwritten_example() {
        let t = trybuild::TestCases::new();
        t.pass("tests/full_example.rs");
    }
}

#[cfg(test)]
mod test_template_types {
    use super::*;
    use std::fmt::Debug;

    #[derive(PartialEq, Debug)]
    pub struct Test<T, U> {
        inner: T,
        outer: U,
    }

    #[derive(VariantAccess, PartialEq, Debug)]
    pub enum Enum<Y: 'static, X: 'static> {
        F1(Y),
        F2(Test<X, Y>),
    }

    #[derive(PartialEq, Debug)]
    pub struct Wrapper(Enum<i64, bool>);

    #[test]
    fn test_has_variant() {
        let test = Enum::<i64, bool>::F1(0);
        assert!(test.has_variant::<i64>());
        assert!(test.has_variant::<Test<bool, i64>>());
        assert!(!test.has_variant::<Test<i64, bool>>());
    }

    #[test]
    fn test_contains_variant() {
        let test = Enum::<i64, bool>::F1(42);
        assert!(test.contains_variant::<i64>().expect("Test failed"));
        assert!(!test
            .contains_variant::<Test<bool, i64>>()
            .expect("Test failed"));
        let test = Enum::<i32, bool>::F2(Test::<bool, i32> {
            inner: true,
            outer: 2,
        });
        assert!(!test.contains_variant::<i32>().expect("Test failed"));
        assert!(test
            .contains_variant::<Test<bool, i32>>()
            .expect("Test failed"));
    }

    #[test]
    fn test_contains_variant_error() {
        let test = Enum::<i32, bool>::F1(42);
        let _ = test
            .contains_variant::<i64>()
            .expect_err("Expected contains_variant to return Err!");
    }

    #[test]
    fn test_get_variant() {
        let test = Enum::<i64, bool>::F2(Test::<bool, i64> {
            inner: true,
            outer: 2,
        });
        let test_field: Test<bool, i64> = test.get_variant().expect("Test failed");
        assert!(test_field.inner);
        let test = Enum::<i64, bool>::F1(42);
        let test_field: i64 = test.get_variant().expect("Test failed");
        assert_eq!(test_field, 42);
    }

    #[test]
    fn test_get_variant_ref() {
        let test = Enum::<i64, bool>::F2(Test::<bool, i64> {
            inner: true,
            outer: 2,
        });
        let test_field: &Test<bool, i64> = test.get_variant_ref().expect("Test failed");
        assert!(test_field.inner);
        let test = Enum::<i64, bool>::F1(42);
        let test_field: &i64 = test.get_variant_ref().expect("Test failed");
        assert_eq!(test_field, &42);
    }

    #[test]
    fn test_get_variant_mut() {
        let mut test = Enum::<i64, bool>::F2(Test::<bool, i64> {
            inner: true,
            outer: 2,
        });
        let test_field: &mut Test<bool, i64> = test.get_variant_mut().expect("Test failed");
        assert!(test_field.inner);
        test_field.inner = false;
        assert_eq!(
            test,
            Enum::<i64, bool>::F2(Test::<bool, i64> {
                inner: false,
                outer: 2
            })
        );
    }

    #[test]
    fn test_set_variant() {
        let mut test = Enum::<i64, bool>::F2(Test::<bool, i64> {
            inner: true,
            outer: 2,
        });
        test.set_variant(42);
        assert_eq!(test, Enum::<i64, bool>::F1(42));
    }

    #[test]
    fn test_trait_create_variant_from() {
        let test = Enum::<i64, bool>::create_variant_from(Test {
            inner: true,
            outer: 2,
        });
        assert_eq!(
            test,
            Enum::<i64, bool>::F2(Test {
                inner: true,
                outer: 2
            })
        );
    }

    #[test]
    fn test_func_create_variant_from() {
        let test = Wrapper(create_variant_from(Test {
            inner: true,
            outer: 2,
        }));
        assert_eq!(
            test,
            Wrapper(Enum::<i64, bool>::F2(Test {
                inner: true,
                outer: 2
            }))
        );
    }
}
