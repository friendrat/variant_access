# Variant Access &emsp; [![Latest Version] [crates.io]]

[Latest Version]: https://img.shields.io/crates/v/variant_access.svg
[crates.io]: https://crates.io/crates/variant_access
A set of traits and macros for defining a common api for Rust enums based on the std::variant api in the C++ standard library

## Basic Usage

Consider the following enum:
``` rust
enum Enum {
    F1(i32),
    F2(bool)
} 
```
We of course may use such an enum directly using the field names and match statements.
The traits provided in this crate allow one to perform that same access of inner values without
explicit use of tags. 

This is modelled on the api for the C++ type std::variant which is like Rust enums but without explicit
names for each possible active field. Consider the following example:
``` rust
let instance = Enum::F1(42);
if instance.has_variant::<i32>() && instance.contains_variant::<i32>().unwrap() {
    let inner: &i32 = instance.get_variant().unwrap();
    ...
}
```
The above code first checks that instance has a field of type `i32`, then checks that this is the active field,
and then gets a reference to the raw value contained therein.

In general, the traits provided in this crate give the following functionality to enums:
``` rust
let mut instance = ...;

// determines whether on of the possible fields has type T
let result: bool = instance.has_variant::<T>();

// determines whether or not the active field has type T.
// If not field has type T, returns Err
let result: Result<bool, ..> = instance.contains_variant::<T>();

// retrieves a reference to the raw value of the field of type T. If the
// active field is not of type T, get_variant returns Err causing the following
// code to panic. If no field has type T, the following will not compile.
let inner: &T = instance.get_variant().unwrap();

// retrieves a mutable reference to the raw value of the field of type T. If the
// active field is not of type T, get_variant returns Err causing the following
// code to panic. If no field has type T, the following will not compile.
let inner: &mut T = instance.get_variant_mut().unwrap();   

// If instance has a field of type bool, this becomes the active field with
// value of `false`. Otherwise, this will not compile.
instance.set_variant(false);

// Since instance can have multiple number-like fields, the following can be
// used to enforce that the field of type i64 is set (if it exists).
// Otherwise the outcome will be ambiguous to the user.
instance.set_variant(3 as i64);
```
For basic enum types, these traits can be derived using the `derive_variant_access` macro. This macro
derives all the traits in this crate. 
``` rust
#[derive(VariantAccess)]
enum Enum {
    F1(i32),
    F2(bool)
}
``` 
A lot of restrictions apply in order for this macro to succeed. First of all, it can only be applied to enums.
Secondly, each field must have a unique type. If any field of the enum itself has more than one field or any 
named fields, the macro will not work (this may be expanded in the future). If any of these conditions are not met,
the code will not compile. This crate is meant for Rust enums that closely resemble C++ variants with only primitive
types.
## Motivation

Out of the box, accessing the active fields in a Rust enum requires direct use of the tags used for the active field.
This is problematic in cases where a more uniform interface for variant / union like types is needed. A prime example is
computer generated code.

For types that are auto-generated, it is difficult to support union types as one needs to generate an enum whose name
and field tags will also be auto-generated and thus opaque to any user. Thus a uniform interface that does not require
knowledge of field names allows the use of such auto-generated types, without being over burdensome to the end user.

As an example, code-generated from protobuf schemas by default make all 
inner values private and provide getters and setters to uniformize interaction with these entities. 