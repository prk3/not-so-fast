#![allow(dead_code, unused_variables)]

use std::collections::HashMap;

use not_so_fast::*;

fn main() {
    // - Introduction -

    // This example file shows how to use `Validator` derive macro to generate
    // validation code for your types from attribute annotations. Keep in
    // mind that `Validator` derive does not look at field types and you'll
    // sometimes have to give it hints on how to reach the data.

    // - Basics -

    // You can apply `Validate` derive on structs and enums, regardless of
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

    assert!(OkStruct { a: "test".into() }.validate().is_ok());
    assert!(OkEnum::Three { a: "test".into() }.validate().is_ok());

    // `Validate` derive implements traits `ValidateArgs` and (indirectly)
    // `Validate` traits. However, the default implementation simply returns
    // `ValidationNode::ok()`. To validate fields inside struct/enums, you have
    // to annotate them with `validate` attribute.

    // `validate` attribute accepts many different arguments for validating
    // different data types. This file shows only simple examples. You'll find
    // the complete documentation in [../../not-so-fast-derive/src/lib.rs].

    // - range -

    // `range` checks if a number (signed integer, unsigned integer, float) is
    // in the specified range. You can define lower and upper bounds with
    // arguments `min` and `max` respectively.

    #[derive(Validate)]
    struct StructRange {
        #[validate(range(min = -100, max = 100))]
        n: i32,
    }

    assert!(StructRange { n: -101 }.validate().is_err());
    assert!(StructRange { n: -100 }.validate().is_ok());
    assert!(StructRange { n: 0 }.validate().is_ok());
    assert!(StructRange { n: 100 }.validate().is_ok());
    assert!(StructRange { n: 101 }.validate().is_err());

    // - length -

    // `length` checks if a list (array, slice, Vec, etc.) has correct length.
    // It can be used with strings, but will then check their byte length, not
    // character length. Like with `range`, you can define valid length range
    // with arguments `min` and `max`. If the list has to have an exact length,
    // use `equals` argument.

    #[derive(Validate)]
    struct StructLength {
        #[validate(length(min = 1, max = 3))]
        l: Vec<u8>,
    }

    assert!(StructLength { l: vec![] }.validate().is_err());
    assert!(StructLength { l: vec![1] }.validate().is_ok());
    assert!(StructLength { l: vec![2, 2] }.validate().is_ok());
    assert!(StructLength { l: vec![3, 3, 3] }.validate().is_ok());
    assert!(StructLength {
        l: vec![4, 4, 4, 4]
    }
    .validate()
    .is_err());

    // - char_length -

    // `char_length` checks if a string (str, String) has correct character
    // length (count of UTF-8 characters). Accepts the same arguments as
    // `length` validator.

    #[derive(Validate)]
    struct StructCharLength {
        #[validate(char_length(min = 5, max = 10))]
        s: String,
    }

    assert!(StructCharLength { s: "hi!".into() }.validate().is_err());
    assert!(StructCharLength { s: "hello!".into() }.validate().is_ok());
    assert!(StructCharLength {
        s: "hello world!".into()
    }
    .validate()
    .is_err());
    assert!(StructCharLength {
        s: "€€€€€€€€€€".into()
    }
    .validate()
    .is_ok());

    // - items -

    // `items` tells `Validate` derive to validate all items of a list-like
    // collection (slice, Vec, VecDeque, etc.). Arguments of the `items`
    // validator are the same as to the `validate` field attribute (`validate`
    // arguments are recursive).

    #[derive(Validate)]
    struct StructItems {
        #[validate(items(range(max = 10)))]
        l: Vec<i32>,
    }

    assert!(StructItems { l: vec![2, 9, 3] }.validate().is_ok());
    assert!(StructItems { l: vec![9, 20, 5] }.validate().is_err());

    // - fields -

    // `fields` works similar to `items`. It validates values of key-value
    // collections, like HashMap or BTreeMap. It accepts the same arguments
    // as field `validate` attribute.

    #[derive(Validate)]
    struct StructFields {
        #[validate(fields(char_length(max = 10)))]
        m: HashMap<String, String>,
    }

    assert!(StructFields {
        m: [("a".into(), "good".into())].into_iter().collect()
    }
    .validate()
    .is_ok());
    assert!(StructFields {
        m: [("a".into(), "bad bad bad".into())].into_iter().collect()
    }
    .validate()
    .is_err());

    // - nested -

    // `nested` validates field with its `ValidateArgs` trait implementation
    // (likely derived with `Validate` macro). When `validate` attribute has no
    // arguments, it defaults to `nested`.

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

    // If field's `ValidateArgs` implementation has non-empty `Args`, you'll
    // have to provide values for these parameters with `args` argument.

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

    // - custom -

    // To cover all your validation needs, you'll likely have to write custom
    // validation functions. `custom` lets you attach these functions to
    // struct/enum field.

    #[derive(Validate)]
    struct StructWithCustom {
        #[validate(custom = validate_name)] // or #[validate(custom(function = validate_name))]
        name: String,
    }

    fn validate_name(name: &str) -> ValidationNode {
        ValidationNode::error_if(name.starts_with('_'), || {
            ValidationError::with_code("underscore")
        })
    }

    assert!(StructWithCustom { name: "n".into() }.validate().is_ok());
    assert!(StructWithCustom { name: "_n".into() }.validate().is_err());

    // Sometimes you need to check values of multiple fields to tell if an
    // object is valid or not. This can be achieved with `custom` validator
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
    // `validate` attribute with `args` argument.

    #[derive(Validate)]
    #[validate(args(x: u64, y: i32, z: char))]
    struct StructWithArgs {}

    // Types with `args` don't implement `Validate` trait. We have to use
    // `ValidateArgs` trait instead. `ValidateArgs` exposes `validate_args`
    // method, which must be supplied with parameter values.

    // This won't work!
    // use not_so_fast::Validate;
    // assert!(StructWithArgs {}.validate().is_ok());

    // This will.
    // use not_so_fast::ValidateArgs;
    assert!(StructWithArgs {}.validate_args((60, 40, '@')).is_ok());

    // The parameters can be forwarded to `custom` and `nested` validators with
    // `args` argument, but also to built-in validators like `range` or
    // `length`. You can put parameter name everywhere a const can go.

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

    // Remember that `args` forwarded to `custom` and `nested` validators are
    // not checked by the derive macro. If their types or arity don't match the
    // requirements of the field, the generated `ValidateArgs` implementation
    // will not compile.
}
