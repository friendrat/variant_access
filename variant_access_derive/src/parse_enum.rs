use super::*;

/// If the decorated enum has generic template parameters,
/// we determine those here.We also validate that there
/// are no lifetime parameters. If there are, the macro
/// panics.
///
/// Otherwise, the full declaration of the type and a map
/// from each generic parameter to it's trait bounds is
/// returned.
///
/// # Example
/// ```
/// #[derive(VariantAccess)]
/// enum Enum<T: 'static + Debug, X: 'static> {
///     F1(T),
///     F2(X)
/// }
/// ```
/// This function then returns `(Enum<T, X>, vec!["T", "X"])`
///
/// # Example
/// ```
/// #[derive(VariantAccess)]
/// enum Enum<'a, T: 'a, X> {
///     F1(T),
///     F2(X)
/// }
/// ```
/// This function panics on the above example.
pub fn fetch_name_with_generic_params(ast: &DeriveInput) -> (String, Vec<String>) {
    let mut param_string = String::from("");
    let params: Vec<String> = ast
        .generics
        .params
        .iter()
        .filter_map(|param| match param {
            GenericParam::Lifetime(inner) => {
                if inner.lifetime.ident.to_token_stream().to_string() != String::from("static") {
                    panic!("VariantAccess can only be derived for types with static lifetimes");
                } else {
                    None
                }
            }
            GenericParam::Type(inner) => {
                param_string.push_str(&format!("{},", inner.ident.to_string()));
                Some(inner.ident.to_string())
            }
            GenericParam::Const(_) => {
                panic!("VariantAccess does not currently support const generics")
            }
        })
        .collect();
    param_string.pop();
    if !params.is_empty() {
        (format!("{}<{}>", ast.ident, param_string), params)
    } else {
        (ast.ident.to_string(), params)
    }
}

/// Recursively parse a type to construct the string representation
/// of it. This includes fully qualifying namespaces as necessary.
///
/// Not all types are supported as fields in the enum. Several of
/// these are due the fact that they are not 'static. It would
/// be nice support impl Traits.
fn parse_type(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Array(array) => parse_array(array),
        syn::Type::Tuple(tuple) => parse_tuple(tuple),
        syn::Type::Path(path) => parse_path(path),
        other @ _ => panic!(
            "VariantAccess cannot be derived for enums with a field of type: {:?}",
            other.to_token_stream()
        ),
    }
}

/// Determines the full path of a named type including all of its nested namespaces.
/// This is used later to ensure that the derive macro can work with full
/// namespace resolution.
///
/// # Exmample
/// ```
/// pub struct Complex {
///     field_one: bool,
///     field_two: f64
/// }
///
/// pub mod namespace {
///     use super::*;
///
///     pub struct Complex<T> {
///         pub field_one: bool,
///         pub field_two: T
///     }
///
///     #[derive(VariantAccess)]
///     pub enum ComplexEnum<T> {
///         F1(Complex<T>),
///         F2(super::Complex)
///     }
/// }
///```
/// The path name of the type of `F1` is `namespace::Complex<T>` and the path name of the type of
/// `F2` is `Complex`.
fn parse_path(path: &syn::TypePath) -> String {
    let mut fullname = String::from("");
    let _ = path
        .path
        .segments
        .pairs()
        .map(|segment| fullname.push_str(&segment.to_token_stream().to_string()))
        .collect::<()>();

    // remove extraneous whitespace
    fullname.retain(|c| c != ' ');
    fullname
}

/// Parse an array type
fn parse_array(array: &syn::TypeArray) -> String {
    let mut fullname = String::from("[");
    let inner_ty = parse_type(&array.elem);
    fullname.push_str(&inner_ty);
    let len = match &array.len {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(int),
            ..
        }) => int,
        _ => panic!(
            "VariantAccess can't be derived on array \
             types whose length is not expressed in terms of an \
             integer literal"
        ),
    };
    fullname.push_str(&format!(";{}]", len.base10_digits()));
    fullname.retain(|c| c != ' ');
    fullname
}

/// Given a tuple, recursive parses each type inside the type
/// to construct the name of the resulting type
fn parse_tuple(tuple: &syn::TypeTuple) -> String {
    let mut fullname = String::from("(");
    let _ = tuple
        .elems
        .pairs()
        .map(|segment| fullname.push_str(&format!("{},", parse_type(segment.value()))))
        .collect::<()>();
    // remove trailing comma
    let mut fullname = String::from(&fullname[..fullname.len() - 1]);
    fullname.push_str(")");
    // remove extraneous whitespace
    fullname.retain(|c| c != ' ');
    fullname
}

/// Makes a map of the form < field type : field name >
///
/// Provides validation
///     -- that no field type is used twice.
///     -- that input is enum.
///     -- that all field types do not have named fields
/// if any of these validations fail, this function panics
/// and halts compilation
///
/// # Example
/// ```
/// enum Enum {
///     F1(i64),
///     F2(bool)
/// }
/// ```
/// returns [ < i64: f1 > , < bool: f2 > ]
///
/// # Example
/// ```
/// #[derive(VariantAccess)]
/// enum Enum {
///     F1(i64),
///     F2(bool),
///     F3(i64),
/// }
/// ```
/// panics in this function as two distinct fields have type i64.
///
/// # Example
/// ```
/// #[derive(VariantAccess)]
/// enum Enum {
///     F1(i64, i32),
///     F2{x: bool}
/// }
/// ```
/// panics in this function as F1 has multiple unnamed types
/// or because F2 has a named field.
pub fn fetch_types_from_enum(ast: &DeriveInput) -> HashMap<String, &Ident> {
    let mut types: HashMap<String, &Ident> = HashMap::new();

    if let Data::Enum(data) = &ast.data {
        for var in data.variants.iter() {
            if let syn::Fields::Unnamed(field_) = &var.fields {
                if field_.unnamed.len() > 1 {
                    panic!("Can only derive for enums whose types do not contain multiple fields.");
                }

                for field_entry in field_.unnamed.iter() {
                    if types
                        .insert(parse_type(&field_entry.ty), &var.ident)
                        .is_some()
                    {
                        panic!("Cannot derive VariantAccess for enum with multiple fields of same type");
                    }
                }
            } else {
                panic!("Cannot derive VariantAccess for enums whose types have named fields.")
            }
        }
    } else {
        panic!("Can only derive VariantAccess for enums.")
    };
    types
}

/// In order to avoid conflicting definitions of the GetVariant / SetVariant traits for
/// enums that are generic over more than one parameter, we use marker structs:
/// see https://stackoverflow.com/questions/52281091/can-i-avoid-eager-ambiguity-resolution-for-trait-implementations-with-generics/52692592#52692592
/// This function generates these structs and places them in a private module.
/// # Example
/// ```
/// #[derive(VariantAccess)]
/// enum Enum<X, Y> {
///     F1(X),
///     F2(Y)
/// }
/// ```
/// produces the following module
/// ```
/// #[allow(non_snake_case)]
/// mod variant_access_Enum {
///     pub (crate) struct F1;
///     pub (crate) struct F2;
/// }
/// ```
pub fn create_marker_structs(name: &str, types: &HashMap<String, &Ident>) -> TokenStream {
    let mut piece = format!("#[allow(non_snake_case)]\n mod variant_access_{}", name);
    piece.push_str("{ ");
    for field_ in types.values() {
        piece.push_str(&format!("pub (crate) struct {};", field_.to_string()));
    }
    piece.push_str("} ");
    piece.parse().unwrap()
}

#[cfg(test)]
mod test_parsers {
    use super::*;

    #[test]
    fn test_parse_tuple() {
        let ast: DeriveInput = syn::parse_str(
            r#"
            enum TupleTest {
                F1((i64, bool)),
            }
        "#,
        )
        .unwrap();
        let types = fetch_types_from_enum(&ast);
        let type_names: Vec<_> = types.keys().collect();
        assert_eq!(type_names, vec!["(i64,bool)"]);
    }

    #[test]
    fn test_parse_array() {
        let ast: DeriveInput = syn::parse_str(
            r#"
            enum TupleTest {
                F1([u8; 32]),
            }
        "#,
        )
        .unwrap();
        let types = fetch_types_from_enum(&ast);
        let type_names: Vec<_> = types.keys().collect();
        assert_eq!(type_names, vec!["[u8;32]"]);
    }

    #[test]
    fn test_parse_tuple_in_array() {
        let ast: DeriveInput = syn::parse_str(
            r#"
            enum TupleTest {
                F1([(i32, bool); 32]),
            }
        "#,
        ).unwrap();
        let types = fetch_types_from_enum(&ast);
        let type_names: Vec<_> = types.keys().collect();
        assert_eq!(type_names, vec!["[(i32,bool);32]"]);
    }

    #[test]
    fn test_parse_array_in_tuple() {
        let ast: DeriveInput = syn::parse_str(
            r#"
            enum TupleTest {
                F1((i32, [u8; 32])),
            }
        "#,
        ).unwrap();
        let types = fetch_types_from_enum(&ast);
        let type_names: Vec<_> = types.keys().collect();
        assert_eq!(type_names, vec!["(i32,[u8;32])"]);
    }

    #[test]
    fn test_parse_generics_arrays_and_tuples() {
        let ast: DeriveInput = syn::parse_str(
            r#"
            enum TupleTest<T: Debug> {
                F1([(T, [u8; 32]); 12]),
            }
        "#,
        ).unwrap();
        let types = fetch_types_from_enum(&ast);
        let type_names: Vec<_> = types.keys().collect();
        assert_eq!(type_names, vec!["[(T,[u8;32]);12]"]);
    }

    #[test]
    fn test_nested_generics() {
        let ast: DeriveInput = syn::parse_str(
            r#"
            enum TupleTest<T: Debug, H> {
                F1((Box<T>, PhantomData<H>)),
            }
        "#,
        ).unwrap();
        let types = fetch_types_from_enum(&ast);
        let type_names: Vec<_> = types.keys().collect();
        assert_eq!(type_names, vec!["(Box<T>,PhantomData<H>)"]);
    }
}
