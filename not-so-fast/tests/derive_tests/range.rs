use not_so_fast::*;

const U8_8: u8 = 8;
const U8_50: u8 = 50;

#[derive(Validate)]
struct S {
    #[validate(range(min = 8))]
    a: u8,

    #[validate(range(max = 50))]
    b: i16,

    #[validate(range(min = 8.0, max = 50.0))]
    c: f32,

    #[validate(range(min = 8u8))]
    d: u8,

    #[validate(range(min = U8_8, max = U8_50))]
    e: u8,
}

impl Default for S {
    fn default() -> Self {
        Self {
            a: 10,
            b: 10,
            c: 10.0,
            d: 10,
            e: 10,
        }
    }
}

#[test]
fn valid() {
    dbg!(S::default().validate().to_string());
    assert!(S::default().validate().is_ok());
}

#[test]
fn min() {
    assert!(S {
        a: 7,
        ..Default::default()
    }
    .validate()
    .is_err());
    assert!(S {
        a: 8,
        ..Default::default()
    }
    .validate()
    .is_ok());
    assert!(S {
        a: 9,
        ..Default::default()
    }
    .validate()
    .is_ok());
}

#[test]
fn max() {
    assert!(S {
        b: 49,
        ..Default::default()
    }
    .validate()
    .is_ok());
    assert!(S {
        b: 50,
        ..Default::default()
    }
    .validate()
    .is_ok());
    assert!(S {
        b: 51,
        ..Default::default()
    }
    .validate()
    .is_err());
}

#[test]
fn min_max() {
    assert!(S {
        c: 7.0,
        ..Default::default()
    }
    .validate()
    .is_err());
    assert!(S {
        c: 7.9999,
        ..Default::default()
    }
    .validate()
    .is_err());
    assert!(S {
        c: 8.0,
        ..Default::default()
    }
    .validate()
    .is_ok());
    assert!(S {
        c: 9.0,
        ..Default::default()
    }
    .validate()
    .is_ok());
    assert!(S {
        c: 49.0,
        ..Default::default()
    }
    .validate()
    .is_ok());
    assert!(S {
        c: 50.0,
        ..Default::default()
    }
    .validate()
    .is_ok());
    assert!(S {
        c: 50.0001,
        ..Default::default()
    }
    .validate()
    .is_err());
    assert!(S {
        c: 51.0,
        ..Default::default()
    }
    .validate()
    .is_err());
}

#[test]
fn typed_literal() {
    assert!(S {
        d: 7,
        ..Default::default()
    }
    .validate()
    .is_err());
    assert!(S {
        d: 8,
        ..Default::default()
    }
    .validate()
    .is_ok());
    assert!(S {
        d: 9,
        ..Default::default()
    }
    .validate()
    .is_ok());
}

#[test]
fn path_arg() {
    assert!(S {
        e: 7,
        ..Default::default()
    }
    .validate()
    .is_err());
    assert!(S {
        e: 8,
        ..Default::default()
    }
    .validate()
    .is_ok());
    assert!(S {
        e: 9,
        ..Default::default()
    }
    .validate()
    .is_ok());
}
