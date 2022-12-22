use not_so_fast::*;

const USIZE_8: usize = 8;
const USIZE_50: usize = 50;

#[derive(Validate)]
struct S<'a> {
    #[validate(length(min = 8))]
    a: String,

    #[validate(length(max = 50))]
    b: String,

    #[validate(length(min = 8, max = 50))]
    c: String,

    #[validate(length(equal = 20))]
    d: String,

    #[validate(length(min = 8))]
    e: &'a str,

    #[validate(length(min = 8))]
    f: Vec<u8>,

    #[validate(length(min = 8))]
    g: &'a [u8],

    #[validate(length(min = self::USIZE_8, max = self::USIZE_50))]
    h: String,
}

impl Default for S<'static> {
    fn default() -> Self {
        Self {
            a: "n".repeat(10),
            b: "n".repeat(10),
            c: "n".repeat(10),
            d: "n".repeat(20),
            e: "nnnnnnnnnn", // 10
            f: vec![0; 10],
            g: &[0; 20],
            h: "n".repeat(10),
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
        a: "a".repeat(7),
        ..Default::default()
    }
    .validate()
    .is_err());
    assert!(S {
        a: "a".repeat(8),
        ..Default::default()
    }
    .validate()
    .is_ok());
    assert!(S {
        a: "a".repeat(9),
        ..Default::default()
    }
    .validate()
    .is_ok());
}

#[test]
fn max() {
    assert!(S {
        b: "a".repeat(49),
        ..Default::default()
    }
    .validate()
    .is_ok());
    assert!(S {
        b: "a".repeat(50),
        ..Default::default()
    }
    .validate()
    .is_ok());
    assert!(S {
        b: "a".repeat(51),
        ..Default::default()
    }
    .validate()
    .is_err());
}

#[test]
fn min_max() {
    assert!(S {
        c: "a".repeat(7),
        ..Default::default()
    }
    .validate()
    .is_err());
    assert!(S {
        c: "a".repeat(8),
        ..Default::default()
    }
    .validate()
    .is_ok());
    assert!(S {
        c: "a".repeat(9),
        ..Default::default()
    }
    .validate()
    .is_ok());
    assert!(S {
        c: "a".repeat(49),
        ..Default::default()
    }
    .validate()
    .is_ok());
    assert!(S {
        c: "a".repeat(50),
        ..Default::default()
    }
    .validate()
    .is_ok());
    assert!(S {
        c: "a".repeat(51),
        ..Default::default()
    }
    .validate()
    .is_err());
}

#[test]
fn equal() {
    assert!(S {
        d: "a".repeat(19),
        ..Default::default()
    }
    .validate()
    .is_err());
    assert!(S {
        d: "a".repeat(20),
        ..Default::default()
    }
    .validate()
    .is_ok());
    assert!(S {
        d: "a".repeat(21),
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
    .is_err());
    assert!(S {
        d: "🔥".repeat(20),
        ..Default::default()
    }
    .validate()
    .is_err());
}

#[test]
fn str() {
    assert!(S {
        e: &"a".repeat(7),
        ..Default::default()
    }
    .validate()
    .is_err());
    assert!(S {
        e: &"a".repeat(8),
        ..Default::default()
    }
    .validate()
    .is_ok());
    assert!(S {
        e: &"a".repeat(9),
        ..Default::default()
    }
    .validate()
    .is_ok());
}

#[test]
fn vec() {
    assert!(S {
        f: vec![0; 7],
        ..Default::default()
    }
    .validate()
    .is_err());
    assert!(S {
        f: vec![0; 8],
        ..Default::default()
    }
    .validate()
    .is_ok());
    assert!(S {
        f: vec![0; 9],
        ..Default::default()
    }
    .validate()
    .is_ok());
}

#[test]
fn slice() {
    assert!(S {
        g: &[0; 7],
        ..Default::default()
    }
    .validate()
    .is_err());
    assert!(S {
        g: &[0; 8],
        ..Default::default()
    }
    .validate()
    .is_ok());
    assert!(S {
        g: &[0; 9],
        ..Default::default()
    }
    .validate()
    .is_ok());
}

#[test]
fn path_arg() {
    assert!(S {
        h: "a".repeat(7),
        ..Default::default()
    }
    .validate()
    .is_err());
    assert!(S {
        h: "a".repeat(8),
        ..Default::default()
    }
    .validate()
    .is_ok());
    assert!(S {
        h: "a".repeat(9),
        ..Default::default()
    }
    .validate()
    .is_ok());
    assert!(S {
        h: "a".repeat(49),
        ..Default::default()
    }
    .validate()
    .is_ok());
    assert!(S {
        h: "a".repeat(50),
        ..Default::default()
    }
    .validate()
    .is_ok());
    assert!(S {
        h: "a".repeat(51),
        ..Default::default()
    }
    .validate()
    .is_err());
}
