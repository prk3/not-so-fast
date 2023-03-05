use not_so_fast::*;

#[test]
fn field_validate_items() {
    #[derive(Validate)]
    struct Child(#[validate(range(max = 10))] i32);

    #[derive(Validate)]
    struct Parent {
        #[validate(items)]
        field: Vec<Child>,
    }
    assert!(Parent { field: vec![] }.validate().is_ok());
    assert!(Parent {
        field: vec![Child(10)]
    }
    .validate()
    .is_ok());
    assert!(Parent {
        field: vec![Child(10), Child(11)]
    }
    .validate()
    .is_err());
}

#[test]
fn field_validate_items_nested() {
    #[derive(Validate)]
    struct Child(#[validate(range(max = 10))] i32);

    #[derive(Validate)]
    struct Parent {
        #[validate(items(nested))]
        field: Vec<Child>,
    }
    assert!(Parent { field: vec![] }.validate().is_ok());
    assert!(Parent {
        field: vec![Child(10)]
    }
    .validate()
    .is_ok());
    assert!(Parent {
        field: vec![Child(10), Child(11)]
    }
    .validate()
    .is_err());
}

#[test]
fn field_validate_items_range() {
    #[derive(Validate)]
    struct Struct {
        #[validate(items(range(max = 10)))]
        field: Vec<i32>,
    }
    assert!(Struct { field: vec![] }.validate().is_ok());
    assert!(Struct { field: vec![10] }.validate().is_ok());
    assert!(Struct {
        field: vec![10, 11]
    }
    .validate()
    .is_err());
}

#[test]
fn field_validate_items_items_range() {
    #[derive(Validate)]
    struct Struct {
        #[validate(items(items(range(max = 10))))]
        field: Vec<Vec<i32>>,
    }
    assert!(Struct { field: vec![] }.validate().is_ok());
    assert!(Struct {
        field: vec![vec![]]
    }
    .validate()
    .is_ok());
    assert!(Struct {
        field: vec![vec![10], vec![10]]
    }
    .validate()
    .is_ok());
    assert!(Struct {
        field: vec![vec![10], vec![10, 11]]
    }
    .validate()
    .is_err());
}

#[test]
fn different_types() {
    use std::collections::*;

    #[derive(Validate)]
    struct S<'a> {
        #[validate(items(range(max = 10)))]
        a: Vec<i32>,
        #[validate(items(range(max = 10)))]
        b: VecDeque<i32>,
        #[validate(items(range(max = 10)))]
        c: LinkedList<i32>,
        #[validate(items(range(max = 10)))]
        d: &'a [i32],
        #[validate(items(range(max = 10)))]
        e: [i32; 10],
    }

    assert!(S {
        a: Default::default(),
        b: Default::default(),
        c: Default::default(),
        d: &[],
        e: Default::default(),
    }
    .validate()
    .is_ok());
}
