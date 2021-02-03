use variant_access_traits::{ContainsVariant, GetVariant, SetVariant, VariantAccessError};
use variant_access_derive::*;
use std::error::Error;
use std::fmt;

// Custom error used in the tests below
#[derive(Debug)]
struct TestError {
    details: String
}

impl TestError {
    fn new(msg: &str) -> TestError {
        TestError{details: msg.to_string()}
    }
}

impl fmt::Display for TestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for TestError {
    fn description(&self) -> &str {
        &self.details
    }
}


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

#[derive(Debug, PartialEq)]
struct Complex {
    field_one: bool,
    field_two: f64
}

mod namespace {
    #[derive(Debug, PartialEq)]
    pub struct Complex {
        field_one: bool,
        field_two: f64
    }
}

#[derive(VariantAccess, PartialEq, Debug)]
enum ComplexEnum {
    F1(Complex),
    F2(namespace::Complex)
}

#[test]
fn test_has_variant() {
    let test = Test::F1(42);
    assert!(test.has_variant::<i32>());
    assert!(test.has_variant::<bool>());
    assert!(!test.has_variant::<i64>());
}

#[test]
fn test_contains_variant() -> Result<(), Box<dyn Error>> {
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
    Ok(())
}

#[test]
fn test_contains_variant_error() -> Result<(), Box<dyn Error>> {
    let test = Test::F1(42);
    if let Ok(_) = test.contains_variant::<i64>() {
        return Err(Box::new(TestError::new("Expected contains_variant to return Err!")));
    }

    Ok(())
}

#[test]
fn test_get_variant() -> Result<(), Box<dyn Error>> {
    let test = Test::F1(42);
    let test_inner_value: &i32 = test.get_variant().expect("Expected get_variant to return value");
    assert_eq!(test_inner_value, &42);

    let test= Test::F2(false);
    let test_inner_value: &bool = test.get_variant().expect("Expected get_variant to return value");
    assert_eq!(test_inner_value, &false);
    Ok(())
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