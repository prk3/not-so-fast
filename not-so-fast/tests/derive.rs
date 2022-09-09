#![allow(dead_code, unused_variables)]

#[macro_use]
extern crate pretty_assertions;

use not_so_fast::*;

#[test]
fn struct_no_args() {
    #[derive(Validate)]
    #[validate(custom = validate_struct_no_args)]
    struct StructNoArgs(#[validate(nested)] Nested);

    #[derive(Validate)]
    struct Nested;

    fn validate_struct_no_args(value: &StructNoArgs) -> ValidationErrors {
        ValidationErrors::error(Error::with_code("struct_no_args"))
    }

    let errors = StructNoArgs(Nested).validate();
    assert!(errors.is_err());
    assert_eq!(".: struct_no_args", errors.to_string());
}

#[test]
fn struct_one_arg() {
    #[derive(Validate)]
    #[validate(
        args(a: u64),
        custom(function = validate_struct_one_arg, args(29))
    )]
    struct StructOneArg(#[validate(nested(args(a)))] Nested);

    #[derive(Validate)]
    #[validate(args(a: u64))]
    struct Nested;

    fn validate_struct_one_arg(value: &StructOneArg, a: u64) -> ValidationErrors {
        ValidationErrors::error(Error::with_code("struct_one_arg").and_param("a", a))
    }

    let errors = StructOneArg(Nested).validate_args((29,));
    assert!(errors.is_err());
    assert_eq!(".: struct_one_arg: a=29", errors.to_string());
}

#[test]
fn struct_many_args() {
    #[derive(Validate)]
    #[validate(
        args(a: u64, b: bool),
        custom(function = validate_struct_many_args, args(29, b))
    )]
    struct StructManyArgs(#[validate(nested(args(a, true)))] Nested);

    #[derive(Validate)]
    #[validate(args(a: u64, b: bool))]
    struct Nested;

    fn validate_struct_many_args(value: &StructManyArgs, a: u64, b: bool) -> ValidationErrors {
        ValidationErrors::error(
            Error::with_code("struct_many_args")
                .and_param("a", a)
                .and_param("b", b),
        )
    }

    let errors = StructManyArgs(Nested).validate_args((29, true));
    assert!(errors.is_err());
    assert_eq!(".: struct_many_args: a=29, b=true", errors.to_string());
}

#[test]
fn struct_generics() {
    #[derive(Validate)]
    #[validate(
        args(a: &'arg u64),
        custom(function = validate_struct_generics, args(a))
    )]
    struct StructGenerics<'a, 'b: 'a, T: Copy + 'b, const N: usize> {
        a: &'a u8,
        b: &'b u8,
        y: T,
        z: [u8; N],
    }

    fn validate_struct_generics<'a, 'b: 'a, T: Copy + 'b, const N: usize>(
        value: &StructGenerics<'a, 'b, T, N>,
        a: &u64,
    ) -> ValidationErrors {
        ValidationErrors::error(Error::with_code("struct_args").and_param("a", *a))
    }

    let instance = StructGenerics {
        a: &0,
        b: &0,
        y: false,
        z: [0; 10],
    };
    let errors = instance.validate_args((&29,));
    assert!(errors.is_err());
    assert_eq!(".: struct_args: a=29", errors.to_string());
}

#[test]
fn struct_ref() {
    struct StructRef<'a> {
        name_ref: &'a String,
        int_ref: &'a i32,
        vec_ref: &'a Vec<String>,
        slice: &'a [String],
        array_ref: &'a [String; 3],
        option_ref: &'a Option<String>,
        map_ref: &'a std::collections::HashMap<String, u32>,
    }

    // TODO add support for lifetimes
}

#[test]
fn empty_enum() {
    #[derive(Validate)]
    #[validate(args(a: u64, b: bool), custom = validate_empty_enum)]
    enum EmptyEnum {}

    fn validate_empty_enum(value: &EmptyEnum) -> ValidationErrors {
        ValidationErrors::ok()
    }

    // We can't construct EmptyEnum. Let's just check if the code compiles.
}

#[test]
fn enum_different_variants() {
    #[derive(Validate)]
    #[validate(
        args(a: u64, b: bool),
        custom(function = validate_enum, args(b))
    )]
    enum Enum {
        NoFields,
        OneField(#[validate(custom(function = validate_string, args(a)))] String),
        TwoFields(
            #[validate(custom(function = validate_string, args(a)))] String,
            u64,
        ),
        OneNamedField {
            #[validate(custom(function = validate_string, args(a)))]
            first: String,
        },
        TwoNamedFields {
            #[validate(custom(function = validate_string, args(a)))]
            first: String,
            second: u64,
        },
    }

    fn validate_string(value: &String, a: u64) -> ValidationErrors {
        ValidationErrors::error(Error::with_code("s").and_param("a", a))
    }

    fn validate_enum(value: &Enum, b: bool) -> ValidationErrors {
        ValidationErrors::error(Error::with_code("e").and_param("b", b))
    }

    assert_eq!(
        ".: e: b=false",
        Enum::NoFields.validate_args((100, false)).to_string()
    );
    assert_eq!(
        ".: e: b=false\n.[0]: s: a=100",
        Enum::OneField("x".into())
            .validate_args((100, false))
            .to_string()
    );
    assert_eq!(
        ".: e: b=false\n.[0]: s: a=100",
        Enum::TwoFields("x".into(), 5)
            .validate_args((100, false))
            .to_string()
    );
    assert_eq!(
        ".: e: b=false\n.first: s: a=100",
        Enum::OneNamedField { first: "x".into() }
            .validate_args((100, false))
            .to_string()
    );
    assert_eq!(
        ".: e: b=false\n.first: s: a=100",
        Enum::TwoNamedFields {
            first: "x".into(),
            second: 5
        }
        .validate_args((100, false))
        .to_string()
    );
}
