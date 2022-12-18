#![allow(dead_code, unused_variables)]

use not_so_fast::*;

fn main() {
    // - Introduction -

    // This example file shows how to use Validator derive macro to generate
    // validation code for your types from attribute annotations. Keep in
    // mind that the Validator macro does not understand types in your program
    // and sometimes you'll need to provide it with more context.

    // - Basics -

    // You can apply Validate derive on structs and enums, regardless of
    // whether their fields are named or not.

    #[derive(Validate)]
    struct OkStruct {
        a: String,
    }

    #[derive(Validate)]
    enum OkEnum {
        One,
        Two(String),
        Three { a: String },
    }

    // `Validate` derive implements traits `ValidateArgs` and (indirectly)
    // `Validate` for our types. However, the default implementation simply
    // returns `ValidationErrors::ok()`. To validate fields inside struct/enum,
    // we will need to annotate them appropriately.

    // - String -

    // - Number -

    // - Vec -

    // - Custom -

    // To cover all your validation needs, you'll likely have to write custom
    // validation functions. We can attach them to struct/enum field with
    // `custom` argument.

    #[derive(Validate)]
    struct StructWithCustom {
        #[validate(custom = validate_name)] // or #[validate(custom(function = validate_name))]
        name: String,
    }

    fn validate_name(name: &String) -> ValidationErrors {
        ValidationErrors::error_if(name.starts_with("_"), || Error::with_code("underscore"))
    }

    assert!(StructWithCustom { name: "n".into() }.validate().is_ok());
    assert!(StructWithCustom { name: "_n".into() }.validate().is_err());

    // Sometimes you need to check values of multiple fields to tell if an
    // object is valid or not. This can be achieved with `custom` attribute
    // applied on the struct/enum itself.

    #[derive(Validate)]
    #[validate(custom = validate_struct)] // or #[validate(custom(function = validate_struct))]
    struct StructWithCustom2 {
        #[validate(custom = validate_name)]
        name: String,
        age: usize,
    }

    fn validate_struct(s: &StructWithCustom2) -> ValidationErrors {
        ValidationErrors::error_if(s.name.len() > s.age, || Error::with_code("name_invariant"))
    }

    assert!(StructWithCustom2 {
        name: "n".repeat(10),
        age: 20
    }
    .validate()
    .is_ok());

    assert!(StructWithCustom2 {
        name: "n".repeat(50),
        age: 40
    }
    .validate()
    .is_err());

    // - Nested -

    // If field's type derives `Validate`, or has custom implementation of
    // `ValidateArgs` trait, we can validate that field with `nested` argument.

    #[derive(Validate)]
    struct Field;

    #[derive(Validate)]
    struct StructWithNestedField {
        #[validate(nested)]
        field: Field,
    }

    assert!(StructWithNestedField { field: Field }.validate().is_ok());

    // - Parametrization -

    // If your `custom` or `nested` validators have parameters, you can specify
    // them with `args` macro.

    struct StructWithParamArgs {
        field:
        name: String,

    }

    // not-so-fast exposes two traits for data validation - `Validate` and
    // `ValidateArgs`. `Validate` is implemented for types  actually a subtrait of `ValidateArgs` with
    // associated type `Args` equal `()`. Types that don't have parameterized
    // validation will implemented both traits.

    // use not_so_fast::Validate;
    assert!(OkStruct { a: "hello".into() }.validate().is_ok());

    // use not_so_fast::ValidateArgs;
    assert!(OkStruct { a: "hello".into() }.validate_args(()).is_ok());

    // When validation is dependant on external data, we can parameterize
    // derived validator using `args` argument on the validated type.

    #[derive(Validate)]
    #[validate(args(a: u64, b: bool))]
    struct StructWithArgs {
        a: String,
    }

    // Since `StructWithArgs` requires arguments for validation, it will only
    // implement `ValidateArgs` trait. `validate_args` method will accept a
    // reference to value and arguments: one of type `u64` and one of type
    // `bool`. The parameter names specified in `validate` attribute are
    // irrelevant at call site, but necessary for routing arguments to custom
    // and nested validators.

    assert!(StructWithArgs { a: "hello".into() }
        .validate_args((54, true))
        .is_ok());

    // Finally, we could add args to outer type and forward them to field
    // validator. Remember that args are not checked in the macro. If their
    // types or arity don't match requirements of the field, the compiler will
    // warn you about type mismatch.

    #[derive(Validate)]
    #[validate(args(a: u64, b: bool, c: char))]
    struct StructWithArgs2 {
        #[validate(custom(function = validate_name_with_param, args(100)))]
        b: String,
        #[validate(nested(args(a, false)))]
        a: FieldWithArgs,
    }

    #[derive(Validate)]
    #[validate(args(a: u64, b: bool))]
    struct InnerWithArgs;

    assert!(OuterWithArgs(InnerWithArgs)
        .validate_args((100, true, 'c'))
        .is_ok());

    // Notice that `OkStructParams` does not implement `Validate` trait.
}
