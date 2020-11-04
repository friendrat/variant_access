use variant_access_traits::{ContainsVariant, GetVariant};
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


#[derive(VariantAccess)]
enum Test {
    F1(i32),
    F2(bool)
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

