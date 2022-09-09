#![allow(dead_code, unused_variables)]

use not_so_fast::*;

fn main() {
    // - Introduction -

    // This example file shows how to use Validator derive macro to generate
    // validation code for your types from attribute annotations. Keep in
    // mind that the Validator macro does not understand types in your program
    // and sometimes you'll need to provide it with more context.

    // - Basics -

    // You can apply Validate derive on structs and enums, regardless of
    // whether their fields are named or not.

    #[derive(Validate)]
    struct OkStruct {
        a: String,
    }

    #[derive(Validate)]
    enum OkEnum {
        One,
        Two(String),
        Three { a: String },
    }

    // Validator derive implemented trait `ValidatorArgs` for types `OkStruct`
    // and `OkEnum`. Since we haven't specified validation arguments, both
    // types will also implement `Validator` trait. The first trait adds
    // `validate_args` method, while the second adds `validate` method.

    // use not_so_fast::ValidateArgs;
    assert!(OkStruct { a: "hello".into() }.validate_args(()).is_ok());
    assert!(OkEnum::Three { a: "hello".into() }
        .validate_args(())
        .is_ok());

    // use not_so_fast::Validate;
    assert!(OkStruct { a: "hello".into() }.validate().is_ok());
    assert!(OkEnum::Three { a: "hello".into() }.validate().is_ok());

    // - Args -

    // When validation is dependant on external data, we can parameterize
    // derived validator using `args` argument on the validated type.

    #[derive(Validate)]
    #[validate(args(a: u64, b: bool))]
    struct OkStructParams {
        a: String,
    }

    // In this case, `validate_args` method on `OkStructParams` will require
    // providing parameters `a` of type `u64` and `b` of type `bool`. The names
    // are irrelevant at call site, but necessary for routing arguments to
    // custom and nested validators (explained later).

    assert!(OkStructParams { a: "hello".into() }
        .validate_args((54, true))
        .is_ok());

    // Notice that `OkStructParams` does not implement `Validate` trait.

    // - Nested -

    // Let's say we have a struct or enum with a field that implements
    // `ValidateArgs` trait. How would we tell Validator macro to use that
    // implementation to validate the field? With `nested` argument.

    #[derive(Validate)]
    struct Inner(#[validate(custom = validate_a)] String);

    fn validate_a(a: &String) -> ValidationErrors {
        ValidationErrors::error(Error::with_code("a"))
    }

    #[derive(Validate)]
    struct Outer(#[validate(nested)] Inner);

    assert!(Outer(Inner("invalid".into())).validate().is_err());

    // Without `#[validate(nested)]` validator would not check inner.

    // What if the field has validation args? We can pass them in `args`
    // argument of `nested`. Here we pass a literal and a const name.

    const GLOBAL_FLAG: bool = true;

    #[derive(Validate)]
    #[validate(args(a: u64, b: bool))]
    struct InnerWithArgs;

    #[derive(Validate)]
    struct OuterWithoutArgs(#[validate(nested(args(100, GLOBAL_FLAG)))] InnerWithArgs);

    assert!(OuterWithoutArgs(InnerWithArgs).validate().is_ok());

    // Finally, we could add args to outer type and forward them to field
    // validator. Remember that args are not checked in the macro. If their
    // types or arity don't match requirements of the field, the compiler will
    // warn you about type mismatch.

    #[derive(Validate)]
    #[validate(args(a: u64, b: bool, c: char))]
    struct OuterWithArgs(#[validate(nested(args(a, false)))] InnerWithArgs);

    assert!(OuterWithArgs(InnerWithArgs)
        .validate_args((100, true, 'c'))
        .is_ok());

    // - Custom -

    // not-so-fast comes with a few basic validators. To cover your validation
    // needs, you'll likely have to write custom validation functions. We can
    // attach custom validation logic to the outer type and fields.

    #[derive(Validate)]
    #[validate(custom = validate_struct_with_custom)]
    struct StructWithCustom {
        #[validate(custom = validate_name)]
        name: String,
    }

    fn validate_struct_with_custom(s: &StructWithCustom) -> ValidationErrors {
        ValidationErrors::error(Error::with_code("s"))
    }

    fn validate_name(name: &String) -> ValidationErrors {
        ValidationErrors::error(Error::with_code("name"))
    }

    assert!(StructWithCustom { name: "n".into() }.validate().is_err());

    // If custom validation functions have additional parameters, we can
    // specify with `args` argument to `custom`.

    #[derive(Validate)]
    #[validate(custom(function = validate_struct_with_param, args(true)))]
    struct StructWithParams {
        #[validate(custom(function = validate_name_with_param, args(100)))]
        name: String,
    }

    fn validate_struct_with_param(s: &StructWithParams, param: bool) -> ValidationErrors {
        ValidationErrors::error(Error::with_code("s"))
    }

    fn validate_name_with_param(name: &String, param: u64) -> ValidationErrors {
        ValidationErrors::error(Error::with_code("name"))
    }

    assert!(StructWithCustom { name: "n".into() }.validate().is_err());

    // Just like with `nested`, args in `custom` may come from outer type args
    // or be literals or constants.

    // fn validate_user(_user: &User, a: u64) -> ValidationErrors {
    //     ValidationErrors::error(Error::with_code("user").and_param("a", a))
    // }

    // fn validate_name(_name: &String, a: u64, b: bool) -> ValidationErrors {
    //     ValidationErrors::error(
    //         Error::with_code("name")
    //             .and_param("a", a)
    //             .and_param("b", b),
    //     )
    // }

    // fn validate_age(_age: &u16) -> ValidationErrors {
    //     ValidationErrors::error(Error::with_code("age"))
    // }
}
