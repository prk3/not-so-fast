use not_so_fast::*;

#[test]
fn struct_no_args() {
    #[derive(Validate)]
    struct StructNoArgs;
    assert!(StructNoArgs.validate().is_ok());
}

#[test]
fn struct_zero_args() {
    #[derive(Validate)]
    #[validate(args())]
    struct StructZeroArgs;
    assert!(StructZeroArgs.validate().is_ok());
}

#[test]
fn struct_one_arg() {
    #[derive(Validate)]
    #[validate(args(a: u64))]
    struct StructOneArg;
    assert!(StructOneArg.validate_args((2,)).is_ok());
}

#[test]
fn struct_two_args() {
    #[derive(Validate)]
    #[validate(args(a: u64, b: bool))]
    struct StructTwoArgs;
    assert!(StructTwoArgs.validate_args((2, true)).is_ok());
}

#[test]
fn enum_two_args() {
    #[derive(Validate)]
    #[validate(args(a: u64, b: bool))]
    enum EnumTwoArgs {
        A,
    }
    assert!(EnumTwoArgs::A.validate_args((2, true)).is_ok());
}

#[test]
fn struct_arg_lifetime() {
    #[derive(Validate)]
    #[validate(args(a: &'arg str, b: &'arg [u8]))]
    enum EnumTwoArgs {
        A,
    }
    assert!(EnumTwoArgs::A.validate_args(("a", &[0])).is_ok());
}

#[test]
fn struct_routing_args() {
    #[derive(Validate)]
    #[validate(args(a: u64, b: &'arg str, c: bool))]
    struct Struct {
        #[validate(custom(function = validate_a, args(a, b)))]
        a: String,
        #[validate(nested(args(b, c)))]
        b: Nested,
    }

    fn validate_a(value: &String, a: u64, b: &str) -> ValidationNode {
        assert!(a == 10);
        assert!(b == "x");
        ValidationNode::ok()
    }

    struct Nested;

    impl<'arg> ValidateArgs<'arg> for Nested {
        type Args = (&'arg str, bool);
        fn validate_args(&self, (b, c): Self::Args) -> ValidationNode {
            assert!(b == "x");
            assert!(c == false);
            ValidationNode::ok()
        }
    }

    assert!(Struct {
        a: "abc".into(),
        b: Nested
    }
    .validate_args((10, "x", false))
    .is_ok());
}
