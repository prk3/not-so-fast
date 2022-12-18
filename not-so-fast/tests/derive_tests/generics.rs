use not_so_fast::*;

#[test]
fn struct_generics() {
    #[derive(Validate)]
    struct StructGenerics0 {
        a: u8,
    }
    assert!(StructGenerics0 { a: 0 }.validate().is_ok());

    #[derive(Validate)]
    struct StructGenerics1L<'a> {
        a: &'a str,
    }
    assert!(StructGenerics1L { a: "hello" }.validate().is_ok());

    #[derive(Validate)]
    struct StructGenerics1T<T> {
        a: T,
    }
    assert!(StructGenerics1T {
        a: String::from("hello")
    }
    .validate()
    .is_ok());

    #[derive(Validate)]
    struct StructGenerics1C<const C: usize> {
        a: [u8; C],
    }
    assert!(StructGenerics1C { a: [0; 3] }.validate().is_ok());

    #[derive(Validate)]
    struct StructGenerics1Each<'a, T, const C: usize> {
        a: &'a str,
        b: T,
        c: [u8; C],
    }
    assert!(StructGenerics1Each {
        a: "hello",
        b: String::from("hello"),
        c: [0; 3]
    }
    .validate()
    .is_ok());

    #[derive(Validate)]
    struct StructGenerics2Each<'a, 'b, T1, T2, const C1: usize, const C2: usize> {
        a: &'a str,
        b: &'b str,
        t1: T1,
        t2: T2,
        c1: [u8; C1],
        c2: [bool; C2],
    }
    assert!(StructGenerics2Each {
        a: "a",
        b: "b",
        t1: String::from("t1"),
        t2: vec![0u8],
        c1: [0; 3],
        c2: [true; 1]
    }
    .validate()
    .is_ok());

    #[derive(Validate)]
    struct StructGenericsBounds<'a, 'b: 'a, T: Clone + 'b, const N: usize> {
        a: &'a u8,
        b: &'b u8,
        y: T,
        z: [u8; N],
    }
    assert!(StructGenericsBounds {
        a: &0,
        b: &0,
        y: false,
        z: [0; 10],
    }
    .validate()
    .is_ok());
}

pub fn enum_generics() {
    #[derive(Validate)]
    enum EnumGenericsBounds<'a, 'b: 'a, T: Clone + 'b, const N: usize> {
        A(&'a u8),
        B(&'b u8),
        C(T),
        D([u8; N]),
    }
    assert!(EnumGenericsBounds::<'_, '_, String, 2>::A(&0)
        .validate()
        .is_ok());
    assert!(EnumGenericsBounds::<'_, '_, String, 2>::B(&0)
        .validate()
        .is_ok());
    assert!(EnumGenericsBounds::<'_, '_, String, 2>::C("x".into())
        .validate()
        .is_ok());
    assert!(EnumGenericsBounds::<'_, '_, String, 2>::D([0; 2])
        .validate()
        .is_ok());
}
