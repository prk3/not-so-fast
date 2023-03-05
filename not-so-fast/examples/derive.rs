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

    // - Nested -

    // If field's type also derives `Validate`, or has custom implementation of
    // `ValidateArgs` trait, we can validate that field like this:

    #[derive(Validate)]
    struct Field(#[validate(range(max = 10))] i32);

    #[derive(Validate)]
    struct StructWithNestedField {
        #[validate] // or #[validate(nested)]
        field: Field,
    }

    assert!(StructWithNestedField { field: Field(15) }
        .validate()
        .is_err());

    // If field's `ValidateArgs` implementation has non-empty Args, you'll have
    // to provide values for these parameters with `args` attribute.

    #[derive(Validate)]
    #[validate(args(bottom: i32, top: i32))]
    struct FieldWithArgs(#[validate(range(min=bottom, max=top))] i32);

    #[derive(Validate)]
    struct StructWithNestedField2 {
        #[validate(nested(args(-100, 100)))]
        field: FieldWithArgs,
    }

    assert!(StructWithNestedField2 {
        field: FieldWithArgs(500)
    }
    .validate()
    .is_err());

    // - Custom -

    // To cover all your validation needs, you'll likely have to write custom
    // validation functions. We can attach them to struct/enum field with
    // `custom` argument.

    #[derive(Validate)]
    struct StructWithCustom {
        #[validate(custom = validate_name)] // or #[validate(custom(function = validate_name))]
        name: String,
    }

    fn validate_name(name: &str) -> ValidationNode {
        ValidationNode::error_if(name.starts_with("_"), || {
            ValidationError::with_code("underscore")
        })
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

    fn validate_struct(s: &StructWithCustom2) -> ValidationNode {
        ValidationNode::error_if(s.name.len() > s.age, || {
            ValidationError::with_code("name_invariant")
        })
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

    // Custom validation functions can have additional parameters, which you
    // are able to specify with `args` attribute.

    #[derive(Validate)]
    struct StructWithCustom3 {
        #[validate(custom(function = validate_bio, args(1000)))]
        bio: String,
    }

    fn validate_bio(bio: &str, word_count: usize) -> ValidationNode {
        ValidationNode::error_if(bio.split_whitespace().count() > word_count, || {
            ValidationError::with_code("word_count").and_param("max", word_count)
        })
    }

    // - Parametrization -

    // Sometimes the same type must be validated differently depending on the
    // context. Maybe a photo gallery attached to a blog post can have more
    // entries than a gallery attached to a comment. Parameterization allows us
    // to pass this sort of information to validation functions.

    // To express that a type has validation parameters, annotate it with
    // `args` attribute.

    #[derive(Validate)]
    #[validate(args(x: u64, y: i32, z: char))]
    struct StructWithArgs {}

    // Types with `args` don't implement `Validate` trait. We have to use
    // `ValidateArgs` trait instead. `ValidateArgs` exposes `validate_args`
    // method, which must be supplied with values for parameters.

    // This won't work!
    // use not_so_fast::Validate;
    // assert!(StructWithArgs {}.validate().is_ok());

    // This will.
    // use not_so_fast::ValidateArgs;
    assert!(StructWithArgs {}.validate_args((60, 40, '@')).is_ok());

    // The parameters can be forwarded to `custom` and `nested` validators with
    // `args` attribute, but also to built-in validators like `range` or
    // `length`. You can put parameter name everywhere a const goes.

    #[derive(Validate)]
    #[validate(args(x: u64, y: i32, z: char))]
    struct StructWithArgs2 {
        #[validate(range(min = 1, max = x))]
        a: u64,
        #[validate(custom(function = validate_b, args(z, 100)))]
        b: String,
        #[validate(nested(args(-100, y)))]
        c: FieldWithArgs,
    }

    fn validate_b(b: &str, special_char: char, special_char_limit: usize) -> ValidationNode {
        ValidationNode::error_if(
            b.chars().filter(|c| *c == special_char).count() > special_char_limit,
            || ValidationError::with_code("special_char_count"),
        )
    }

    assert!(StructWithArgs2 {
        a: 200,
        b: "@".repeat(200),
        c: FieldWithArgs(200)
    }
    .validate_args((60, 40, '@'))
    .is_ok());

    // Remember that `args` forwarded to `custom` and `nested` attributes are
    // not checked by the derive macro. If their types or arity don't match the
    // requirements of the field, the generated `ValidateArgs` implementation
    // will not compile.
}
