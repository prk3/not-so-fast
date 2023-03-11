use not_so_fast::*;

#[test]
fn struct_custom_basic() {
    #[derive(Validate)]
    #[validate(custom = validate_struct)]
    struct StructCustom {
        a: u8,
    }
    fn validate_struct(value: &StructCustom) -> ValidationNode {
        ValidationNode::error_if(value.a % 3 == 0, || ValidationError::with_code("x"))
    }

    assert_eq!("", StructCustom { a: 2 }.validate().to_string());
    assert_eq!(".: x", StructCustom { a: 3 }.validate().to_string());
}

#[test]
fn struct_custom_alternative_syntax() {
    #[derive(Validate)]
    #[validate(custom(function = validate_struct))]
    struct StructCustom {
        a: u8,
    }
    fn validate_struct(value: &StructCustom) -> ValidationNode {
        ValidationNode::error_if(value.a % 3 == 0, || ValidationError::with_code("x"))
    }

    assert_eq!("", StructCustom { a: 2 }.validate().to_string());
    assert_eq!(".: x", StructCustom { a: 3 }.validate().to_string());
}

#[test]
fn struct_custom_three_custom() {
    #[derive(Validate)]
    #[validate(custom = validate_struct_a, custom = validate_struct_b)]
    #[validate(custom(function = validate_struct_c))]
    struct StructCustom {
        a: u8,
    }
    fn validate_struct_a(value: &StructCustom) -> ValidationNode {
        ValidationNode::error_if(value.a % 3 == 0, || ValidationError::with_code("a"))
    }
    fn validate_struct_b(value: &StructCustom) -> ValidationNode {
        ValidationNode::error_if(value.a % 4 == 0, || ValidationError::with_code("b"))
    }
    fn validate_struct_c(value: &StructCustom) -> ValidationNode {
        ValidationNode::error_if(value.a % 5 == 0, || ValidationError::with_code("c"))
    }

    assert_eq!("", StructCustom { a: 2 }.validate().to_string());
    assert_eq!(".: a", StructCustom { a: 3 }.validate().to_string());
    assert_eq!(".: b", StructCustom { a: 4 }.validate().to_string());
    assert_eq!(".: c", StructCustom { a: 5 }.validate().to_string());
    assert_eq!(
        ".: a\n.: b\n.: c",
        StructCustom { a: 60 }.validate().to_string()
    );
}

#[test]
fn struct_custom_with_one_arg() {
    const X: u32 = 10;

    #[derive(Validate)]
    #[validate(
        args(a: bool, b: &'arg str, c: u64),
        custom(function = validate_struct, args(a)),
    )]
    struct StructCustom {
        a: u8,
    }
    fn validate_struct(value: &StructCustom, a: bool) -> ValidationNode {
        assert!(a);
        ValidationNode::error_if(value.a % 3 == 0, || ValidationError::with_code("x"))
    }

    assert_eq!(
        "",
        StructCustom { a: 2 }
            .validate_args((true, "hello", 50))
            .to_string()
    );
    assert_eq!(
        ".: x",
        StructCustom { a: 3 }
            .validate_args((true, "hello", 50))
            .to_string()
    );
}

#[test]
fn struct_custom_with_multiple_args() {
    const X: u32 = 10;

    #[derive(Validate)]
    #[validate(
        args(a: bool, b: &'arg str, c: u64),
        custom(function = validate_struct, args(b, 100, X)),
    )]
    struct StructCustom {
        a: u8,
    }
    fn validate_struct(value: &StructCustom, b: &str, c: u64, x: u32) -> ValidationNode {
        assert!(b == "hello");
        assert!(c == 100);
        assert!(x == 10);
        ValidationNode::error_if(value.a % 3 == 0, || ValidationError::with_code("x"))
    }

    assert_eq!(
        "",
        StructCustom { a: 2 }
            .validate_args((true, "hello", 50))
            .to_string()
    );
    assert_eq!(
        ".: x",
        StructCustom { a: 3 }
            .validate_args((true, "hello", 50))
            .to_string()
    );
}

#[test]
fn enum_custom_basic() {
    #[derive(Validate)]
    #[validate(custom = validate_enum)]
    enum EnumCustom {
        A,
        B(u8),
        C { x: u16 },
    }
    fn validate_enum(value: &EnumCustom) -> ValidationNode {
        ValidationNode::error_if(matches!(value, EnumCustom::B(..)), || {
            ValidationError::with_code("x")
        })
    }

    assert_eq!("", EnumCustom::A.validate().to_string());
    assert_eq!("", EnumCustom::C { x: 1 }.validate().to_string());
    assert_eq!(".: x", EnumCustom::B(5).validate().to_string());
}

#[test]
fn struct_field_custom_basic() {
    #[derive(Validate)]
    struct StructFieldCustom {
        #[validate(custom = validate_field)]
        a: u8,
    }
    fn validate_field(value: &u8) -> ValidationNode {
        ValidationNode::error_if(value % 3 == 0, || ValidationError::with_code("x"))
    }

    assert_eq!("", StructFieldCustom { a: 2 }.validate().to_string());
    assert_eq!(".a: x", StructFieldCustom { a: 3 }.validate().to_string());
}

#[test]
fn struct_field_custom_alternative_syntax() {
    #[derive(Validate)]
    struct StructFieldCustom {
        #[validate(custom(function = validate_field))]
        a: u8,
    }
    fn validate_field(value: &u8) -> ValidationNode {
        ValidationNode::error_if(value % 3 == 0, || ValidationError::with_code("x"))
    }

    assert_eq!("", StructFieldCustom { a: 2 }.validate().to_string());
    assert_eq!(".a: x", StructFieldCustom { a: 3 }.validate().to_string());
}

#[test]
fn field_custom_three_custom() {
    #[derive(Validate)]
    struct FieldCustom {
        #[validate(custom = validate_field_a, custom(function = validate_field_b))]
        #[validate(custom = validate_field_c)]
        a: u8,
    }
    fn validate_field_a(value: &u8) -> ValidationNode {
        ValidationNode::error_if(value % 3 == 0, || ValidationError::with_code("a"))
    }
    fn validate_field_b(value: &u8) -> ValidationNode {
        ValidationNode::error_if(value % 4 == 0, || ValidationError::with_code("b"))
    }
    fn validate_field_c(value: &u8) -> ValidationNode {
        ValidationNode::error_if(value % 5 == 0, || ValidationError::with_code("c"))
    }

    assert_eq!("", FieldCustom { a: 2 }.validate().to_string());
    assert_eq!(".a: a", FieldCustom { a: 3 }.validate().to_string());
    assert_eq!(".a: b", FieldCustom { a: 4 }.validate().to_string());
    assert_eq!(".a: c", FieldCustom { a: 5 }.validate().to_string());
    assert_eq!(
        ".a: a\n.a: b\n.a: c",
        FieldCustom { a: 60 }.validate().to_string()
    );
}

#[test]
fn enum_field_custom_basic() {
    #[derive(Validate)]
    enum EnumFieldCustom {
        A,
        B(#[validate(custom = validate_enum_field_b)] u8),
        C {
            #[validate(custom = validate_enum_field_c)]
            x: u16,
        },
    }
    fn validate_enum_field_b(value: &u8) -> ValidationNode {
        ValidationNode::error_if(*value == 8, || ValidationError::with_code("x"))
    }
    fn validate_enum_field_c(value: &u16) -> ValidationNode {
        ValidationNode::error_if(*value == 16, || ValidationError::with_code("x"))
    }

    assert_eq!("", EnumFieldCustom::A.validate().to_string());
    assert_eq!(".[0]: x", EnumFieldCustom::B(8).validate().to_string());
    assert_eq!("", EnumFieldCustom::B(16).validate().to_string());
    assert_eq!("", EnumFieldCustom::C { x: 8 }.validate().to_string());
    assert_eq!(".x: x", EnumFieldCustom::C { x: 16 }.validate().to_string());
}
