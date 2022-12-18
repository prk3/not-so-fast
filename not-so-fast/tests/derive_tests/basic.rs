use not_so_fast::*;

#[test]
fn struct_unit() {
    #[derive(Validate)]
    struct StructUnit;
    assert!(StructUnit.validate().is_ok());
}

#[test]
fn struct_tuple() {
    #[derive(Validate)]
    struct StructTuple0();
    assert!(StructTuple0().validate().is_ok());

    #[derive(Validate)]
    struct StructTuple1(String);
    assert!(StructTuple1("1".into()).validate().is_ok());

    #[derive(Validate)]
    struct StructTuple2(String, String);
    assert!(StructTuple2("1".into(), "2".into()).validate().is_ok());
}

#[test]
fn struct_regular() {
    #[derive(Validate)]
    struct StructRegular0 {}
    assert!(StructRegular0 {}.validate().is_ok());

    #[derive(Validate)]
    struct StructRegular1 {
        a: String,
    }
    assert!(StructRegular1 { a: "a".into() }.validate().is_ok());

    #[derive(Validate)]
    struct StructRegular2 {
        a: String,
        b: String,
    }
    assert!(StructRegular2 {
        a: "a".into(),
        b: "b".into(),
    }
    .validate()
    .is_ok());
}

#[test]
fn empty_enum() {
    #[derive(Validate)]
    enum EmptyEnum {}

    // We can't construct EmptyEnum. Let's just check if the code compiles.
}

#[test]
fn empty_different_variants() {
    #[derive(Validate)]
    enum Enum {
        NoFields,
        OneField(String),
        TwoFields(String, String),
        OneNamedField { first: String },
        TwoNamedFields { first: String, second: String },
    }

    assert!(Enum::NoFields.validate().is_ok());
    assert!(Enum::OneField("x".into()).validate().is_ok());
    assert!(Enum::TwoFields("x".into(), "x".into()).validate().is_ok());
    assert!(Enum::OneNamedField { first: "x".into() }.validate().is_ok());
    assert!(Enum::TwoNamedFields {
        first: "x".into(),
        second: "x".into()
    }
    .validate()
    .is_ok());
}
