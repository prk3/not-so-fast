use not_so_fast::*;

#[test]
fn field_validate_some() {
    #[derive(Validate)]
    struct Child(#[validate(range(max = 10))] i32);

    #[derive(Validate)]
    struct Parent {
        #[validate(some)]
        field: Option<Child>,
    }
    assert!(Parent { field: None }.validate().is_ok());
    assert!(Parent {
        field: Some(Child(10))
    }
    .validate()
    .is_ok());
    assert!(Parent {
        field: Some(Child(11))
    }
    .validate()
    .is_err());
}

#[test]
fn field_validate_some_nested() {
    #[derive(Validate)]
    struct Child(#[validate(range(max = 10))] i32);

    #[derive(Validate)]
    struct Parent {
        #[validate(some(nested))]
        field: Option<Child>,
    }
    assert!(Parent { field: None }.validate().is_ok());
    assert!(Parent {
        field: Some(Child(10))
    }
    .validate()
    .is_ok());
    assert!(Parent {
        field: Some(Child(11))
    }
    .validate()
    .is_err());
}

#[test]
fn field_validate_some_range() {
    #[derive(Validate)]
    struct Struct {
        #[validate(some(range(max = 10)))]
        field: Option<i32>,
    }
    assert!(Struct { field: None }.validate().is_ok());
    assert!(Struct { field: Some(10) }.validate().is_ok());
    assert!(Struct { field: Some(11) }.validate().is_err());
}

#[test]
fn field_validate_some_some_range() {
    #[derive(Validate)]
    struct Struct {
        #[validate(some(some(range(max = 10))))]
        field: Option<Option<i32>>,
    }
    assert!(Struct { field: None }.validate().is_ok());
    assert!(Struct { field: Some(None) }.validate().is_ok());
    assert!(Struct {
        field: Some(Some(10))
    }
    .validate()
    .is_ok());
    assert!(Struct {
        field: Some(Some(11))
    }
    .validate()
    .is_err());
}
