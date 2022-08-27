use not_so_fast::*;
use derive::Validate;

#[derive(Validate)]
#[validate(args(a: u64, b: bool), custom(function = validate_user, args(a)))]
struct User {
    #[validate(custom(function = validate_name, args(a, b)))]
    name: String,
    #[validate(custom(function = validate_age))]
    age: u16,
}

fn validate_user(user: &User, a: u64) -> ValidationErrors {
    ValidationErrors::error(Error::with_code("user").and_param("a", a.to_string()))
}

fn validate_name(name: &String, a: u64, b: bool) -> ValidationErrors {
    ValidationErrors::error(
        Error::with_code("name")
            .and_param("a", a.to_string())
            .and_param("b", b.to_string()),
    )
}

fn validate_age(age: u16) -> ValidationErrors {
    ValidationErrors::error(Error::with_code("age"))
}
