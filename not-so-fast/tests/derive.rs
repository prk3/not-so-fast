#![allow(dead_code, unused_variables)]

use derive::Validate;
use not_so_fast::{Error, Validate, ValidateArgs, ValidationErrors};

#[test]
fn struct_a() {
    #[derive(Validate)]
    #[validate(custom = validate_struct_a)]
    struct StructA {
        name: String,
        int: i32,
        vec: Vec<String>,
        array: [String; 3],
        option: Option<String>,
        map: std::collections::HashMap<String, u32>,
    }

    fn validate_struct_a(value: &StructA) -> ValidationErrors {
        ValidationErrors::error(Error::with_code("struct_a"))
    }

    let instance = StructA {
        name: "hello".into(),
        int: 1,
        vec: vec!["foo".into(), "bar".into()],
        array: ["a".into(), "b".into(), "c".into()],
        option: Some("some".into()),
        map: Default::default(),
    };
    let errors = instance.validate();
    assert!(errors.is_err());
    assert_eq!(".: struct_a", errors.to_string());
}

#[test]
fn struct_b() {
    #[derive(Validate)]
    #[validate(
        args(a: u64, b: bool),
        custom(function = validate_struct_b, args(29, b))
    )]
    struct StructB {
        name: String,
        int: i32,
        vec: Vec<String>,
        array: [String; 3],
        option: Option<String>,
        map: std::collections::HashMap<String, u32>,
    }

    fn validate_struct_b(value: &StructB, a: u64, b: bool) -> ValidationErrors {
        ValidationErrors::error(
            Error::with_code("struct_b")
                .and_param("a", a.to_string())
                .and_param("b", b.to_string()),
        )
    }

    let instance = StructB {
        name: "hello".into(),
        int: 1,
        vec: vec!["foo".into(), "bar".into()],
        array: ["a".into(), "b".into(), "c".into()],
        option: Some("some".into()),
        map: Default::default(),
    };
    let errors = instance.validate_args((29, true));
    assert!(errors.is_err());
    assert_eq!(".: struct_b: a=29, b=true", errors.to_string());
}

#[test]
fn struct_c() {
    struct StructC<'a> {
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
