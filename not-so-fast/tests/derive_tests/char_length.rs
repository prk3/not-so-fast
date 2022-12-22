use not_so_fast::*;

const USIZE_8: usize = 8;
const USIZE_50: usize = 50;

#[derive(Validate)]
struct S<'a> {
    #[validate(char_length(min = 8))]
    a: String,

    #[validate(char_length(max = 50))]
    b: String,

    #[validate(char_length(min = 8, max = 50))]
    c: String,

    #[validate(char_length(equal = 20))]
    d: String,

    #[validate(char_length(min = 8))]
    e: &'a str,

    #[validate(char_length(min = self::USIZE_8, max = self::USIZE_50))]
    f: String,
}

impl Default for S<'static> {
    fn default() -> Self {
        Self {
            a: "n".repeat(10),
            b: "n".repeat(10),
            c: "n".repeat(10),
            d: "n".repeat(20),
            e: "nnnnnnnnnn", // 10
            f: "n".repeat(10),
        }
    }
}

#[test]
fn valid() {
    assert!(S::default().validate().is_ok());
}

#[test]
fn min() {
    assert!(S {
        a: "ß".repeat(7),
        ..Default::default()
    }
    .validate()
    .is_err());
    assert!(S {
        a: "ß".repeat(8),
        ..Default::default()
    }
    .validate()
    .is_ok());
    assert!(S {
        a: "ß".repeat(9),
        ..Default::default()
    }
    .validate()
    .is_ok());
}

#[test]
fn max() {
    assert!(S {
        b: "ß".repeat(49),
        ..Default::default()
    }
    .validate()
    .is_ok());
    assert!(S {
        b: "ß".repeat(50),
        ..Default::default()
    }
    .validate()
    .is_ok());
    assert!(S {
        b: "ß".repeat(51),
        ..Default::default()
    }
    .validate()
    .is_err());
}

#[test]
fn min_max() {
    assert!(S {
        c: "ß".repeat(7),
        ..Default::default()
    }
    .validate()
    .is_err());
    assert!(S {
        c: "ß".repeat(8),
        ..Default::default()
    }
    .validate()
    .is_ok());
    assert!(S {
        c: "ß".repeat(9),
        ..Default::default()
    }
    .validate()
    .is_ok());
    assert!(S {
        c: "ß".repeat(49),
        ..Default::default()
    }
    .validate()
    .is_ok());
    assert!(S {
        c: "ß".repeat(50),
        ..Default::default()
    }
    .validate()
    .is_ok());
    assert!(S {
        c: "ß".repeat(51),
        ..Default::default()
    }
    .validate()
    .is_err());
}

#[test]
fn equal() {
    assert!(S {
        d: "ß".repeat(19),
        ..Default::default()
    }
    .validate()
    .is_err());
    assert!(S {
        d: "ß".repeat(20),
        ..Default::default()
    }
    .validate()
    .is_ok());
    assert!(S {
        d: "ß".repeat(21),
        ..Default::default()
    }
    .validate()
    .is_err());

    // different char sizes
    assert!(S {
        d: "a".repeat(20),
        ..Default::default()
    }
    .validate()
    .is_ok());
    assert!(S {
        d: "ą".repeat(20),
        ..Default::default()
    }
    .validate()
    .is_ok());
    assert!(S {
        d: "🔥".repeat(20),
        ..Default::default()
    }
    .validate()
    .is_ok());
}

#[test]
fn str() {
    assert!(S {
        e: &"ß".repeat(7),
        ..Default::default()
    }
    .validate()
    .is_err());
    assert!(S {
        e: &"ß".repeat(8),
        ..Default::default()
    }
    .validate()
    .is_ok());
    assert!(S {
        e: &"ß".repeat(9),
        ..Default::default()
    }
    .validate()
    .is_ok());
}

#[test]
fn path_arg() {
    assert!(S {
        f: "ß".repeat(7),
        ..Default::default()
    }
    .validate()
    .is_err());
    assert!(S {
        f: "ß".repeat(8),
        ..Default::default()
    }
    .validate()
    .is_ok());
    assert!(S {
        f: "ß".repeat(9),
        ..Default::default()
    }
    .validate()
    .is_ok());
    assert!(S {
        f: "ß".repeat(49),
        ..Default::default()
    }
    .validate()
    .is_ok());
    assert!(S {
        f: "ß".repeat(50),
        ..Default::default()
    }
    .validate()
    .is_ok());
    assert!(S {
        f: "ß".repeat(51),
        ..Default::default()
    }
    .validate()
    .is_err());
}
