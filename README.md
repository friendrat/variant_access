# Variant Access &emsp; ![Latest Version] 

[Latest Version]: https://img.shields.io/crates/v/variant_access.svg
[crates.io]: https://crates.io/crates/variant_access
A set of traits and macros for defining a common api for Rust enums based on the std::variant api in the C++ standard library

[See the documentation](https://docs.rs/variant_access_traits) 
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
```rust
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
For enum types (subject to certain restrictions detailed in the Type Requirements section below), these traits can be
 derived using the `derive_variant_access` macro. This macro derives all the traits in this crate. 
``` rust
use variant_access_traits::*;
use variant_access_derive::*; 
#[derive(VariantAccess)]
enum Enum {
    F1(i32),
    F2(bool)
}
``` 
Note that we recommend to always add 
 ```rust
use variant_access_traits::*;
use variant_access_derive::*; 
``` 
to use these features.

Several restrictions apply in order for this macro to succeed. First of all, it can only be applied to enums.
Secondly, each field must have a unique type. If any field of the enum itself has more than one field or any 
named fields, the macro will not work (this may be expanded in the future). If any of these conditions are not met,
the code will not compile.
## Motivation

Out of the box, accessing the active fields in a Rust enum requires direct use of the tags used for the active field.
This is problematic in cases where a more uniform interface for variant / union like types is needed. A prime example is
computer generated code.

For types that are auto-generated, it is difficult to support union types as one needs to generate an enum whose name
and field tags will also be auto-generated and thus opaque to any user. Thus a uniform interface that does not require
knowledge of field names allows the use of such auto-generated types, without being over burdensome to the end user.

As an example, code-generated from protobuf schemas by default make all 
inner values private and provide getters and setters to uniformize interaction with these entities.

## Supported features

The derive macro is able to fully distinguish types, even those with the same name but in different modules. The full 
namespace resolution is achieved by using ``` std::any::TypeId```. For example
```rust
#[derive(Debug, PartialEq)]
 pub struct Complex {
    field_one: bool,
    field_two: f64
 }

 pub mod namespace {
    
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
```
works and the various trait methods can distinguish between the type ```Complex``` and ```namespace::Complex```.

Generics are also supported. For example 
```rust
#[derive(PartialEq, Debug)]
pub struct Test<T, U>{
   inner: T,
   outer: U,
}

#[derive(VariantAccess, PartialEq, Debug)]
pub enum Enum<Y: 'static, X: 'static> {
    F1(Y),
    F2(Test<X, Y>)
}
``` 
works and means that the instantiated trait methods will automatically work, e.g., for the 
the type ```Enum<i64, bool>```. So for example, 
```rust
fn main() {
    let test = Enum::<i64, bool>::F2(Test{inner: true, outer: 2});
    let value: &Test<bool, i64> = test.get_variant().unwrap();
    assert_eq!(value, Test{inner: true, outer: 2});
}
```
In order to support enum definitions with more than one generic parameter, it was necessary to use
marker structs to avoid conflicting definitions, see this [question](https://stackoverflow.com/questions/52281091/can-i-avoid-eager-ambiguity-resolution-for-trait-implementations-with-generics/52692592#52692592) on Stackoverflow.

As such, in the above example, the following module and marker structs will also be created:
```rust
#[allow(non_snake_case)]
mod variant_access_Enum {
    pub (crate) struct F1;
    pub (crate) struct F2;
}
```
So beware in case you were thinking of creating the module ```variant_access_Enum``` yourself! :stuck_out_tongue_closed_eyes:

We also provide a trait and function for creating instance of variants given a value of a certain type. Consider the
 following example:
 ```rust
 use variant_access_traits::*;
 use variant_access_derive::*;

 #[(VariantAccessDerive)]
 enum HorribleComputerGeneratedEnumName {
     AwfulComputerGeneratedField1(f64),
     AwfulComputerGeneratedField2(bool)
 }

 struct LovelyStruct {
     lovely_field_name: HorribleComputerGeneratedEnumName
 }

 fn main() {
     let lovely = LovelyStruct{lovely_field_name: create_variant_from(3.0)};
 }

```
The `create_variant_from` function is able to deduce that since `lovely_field_name` is of type `HorribleComputerGeneratedEnumName`
and the input to the function is an `f64`, that it should return `HorribleComputerGeneratedEnumName::AwfulComputerGeneratedField1(3.0)`.
This example goes back to the original motivation of this crate.
## Type Requirements

There are several requirements that your enum definition must satisfy in order for the traits and / or the 
derive macro to work. First of all, all types must subscribe to `'static`. This is a requirement of ```std::any::TypeId```
(as apparently it is difficult to distinguish to types that differ only in lifetime). Furthermore, this a trait bound
for some of the variant_access traits. 

This also means that when using generics in your enum definition, you must add the `'static` trait bound (see the 
example in the previous section).

For the derive macro to work, it is also necessary that all field types of the enum implement the `PartialEq` and `Debug`
traits.

For a more complete list of restrictions and misuses, see the `uncompilable_examples` subdirectory in the `tests` folder.

## Known Issues
Currently, the only main known issue involves running the test suite for this crate. `cargo test` fails due to an
issue in `trybuild` and Rust workspaces. I have been unable to resolve it, but running the tests one by one
via `cargo test --package variant_access --test tests {{test name}}` ensures that all are passing. 