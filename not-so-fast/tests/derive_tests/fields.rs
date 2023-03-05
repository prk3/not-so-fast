use std::borrow::Cow;
use std::collections::HashMap;

use not_so_fast::*;

macro_rules! map {
    ($($key:expr => $value:expr),*) => {
        [$(($key, $value)),*].into_iter().collect()
    };
}

#[test]
fn field_validate_fields() {
    #[derive(Validate)]
    struct Child(#[validate(range(max = 10))] i32);

    #[derive(Validate)]
    struct Parent {
        #[validate(fields)]
        field: HashMap<i32, Child>,
    }
    assert!(Parent { field: map! {} }.validate().is_ok());
    assert!(Parent {
        field: map! { 1 => Child(10) }
    }
    .validate()
    .is_ok());
    assert!(Parent {
        field: map! { 1 => Child(10), 2 => Child(11) }
    }
    .validate()
    .is_err());
}

#[test]
fn field_validate_fields_nested() {
    #[derive(Validate)]
    struct Child(#[validate(range(max = 10))] i32);

    #[derive(Validate)]
    struct Parent {
        #[validate(fields(nested))]
        field: HashMap<i32, Child>,
    }
    assert!(Parent { field: map! {} }.validate().is_ok());
    assert!(Parent {
        field: map! { 1 => Child(10) }
    }
    .validate()
    .is_ok());
    assert!(Parent {
        field: map! { 1 => Child(10), 2 => Child(11) }
    }
    .validate()
    .is_err());
}

#[test]
fn field_validate_fields_range() {
    #[derive(Validate)]
    struct Struct {
        #[validate(fields(range(max = 10)))]
        field: HashMap<i32, i32>,
    }
    assert!(Struct { field: map! {} }.validate().is_ok());
    assert!(Struct {
        field: map! { 1 => 10 }
    }
    .validate()
    .is_ok());
    assert!(Struct {
        field: map! { 1 => 10, 2 => 11 }
    }
    .validate()
    .is_err());
}

#[test]
fn field_validate_fields_fields_range() {
    #[derive(Validate)]
    struct Struct {
        #[validate(fields(fields(range(max = 10))))]
        field: HashMap<i32, HashMap<i32, i32>>,
    }
    assert!(Struct { field: map! {} }.validate().is_ok());
    assert!(Struct {
        field: map! {
            1 => map!{}
        }
    }
    .validate()
    .is_ok());
    assert!(Struct {
        field: map! {
            1 => map! { 1 => 10 },
            2 => map! { 1 => 10 }
        }
    }
    .validate()
    .is_ok());
    assert!(Struct {
        field: map! {
            1 => map! { 1 => 10 },
            2 => map! { 1 => 10, 2 => 11 }
        }
    }
    .validate()
    .is_err());
}

#[test]
fn different_types() {
    use std::collections::*;

    struct CustomKey;

    impl ToString for CustomKey {
        fn to_string(&self) -> String {
            "hello".into()
        }
    }

    #[derive(Validate)]
    struct S<'a> {
        #[validate(fields(range(max = 10)))]
        a: HashMap<i32, i32>,

        #[validate(fields(range(max = 10)))]
        b: BTreeMap<i32, i32>,

        #[validate(fields(range(max = 10)))]
        c: HashMap<String, i32>,

        #[validate(fields(range(max = 10)))]
        d: &'a HashMap<String, i32>,

        #[validate(fields(range(max = 10)))]
        e: HashMap<Cow<'static, str>, i32>,

        #[validate(fields(range(max = 10)))]
        f: HashMap<CustomKey, i32>,
    }

    assert!(S {
        a: Default::default(),
        b: Default::default(),
        c: Default::default(),
        d: &Default::default(),
        e: Default::default(),
        f: Default::default(),
    }
    .validate()
    .is_ok());
}
