use not_so_fast::*;

#[test]
fn field_validate() {
    #[derive(Validate)]
    struct Child(#[validate(range(max = 10))] i32);

    #[derive(Validate)]
    struct Parent {
        #[validate]
        field: Child,
    }
    assert!(Parent { field: Child(10) }.validate().is_ok());
    assert!(Parent { field: Child(11) }.validate().is_err());
}

#[test]
fn field_validate_nested() {
    #[derive(Validate)]
    struct Child(#[validate(range(max = 10))] i32);

    #[derive(Validate)]
    struct Parent {
        #[validate(nested)]
        field: Child,
    }
    assert!(Parent { field: Child(10) }.validate().is_ok());
    assert!(Parent { field: Child(11) }.validate().is_err());
}

#[test]
fn field_validate_nested_args() {
    #[derive(Validate)]
    #[validate(args(m: i32))]
    struct Child(#[validate(range(max=m))] i32);

    #[derive(Validate)]
    struct Parent {
        #[validate(nested(args(10)))]
        field: Child,
    }
    assert!(Parent { field: Child(10) }.validate().is_ok());
    assert!(Parent { field: Child(11) }.validate().is_err());
}
