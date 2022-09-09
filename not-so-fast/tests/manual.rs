#[macro_use]
extern crate pretty_assertions;

use not_so_fast::*;

#[test]
fn user() {
    struct User {
        name: String,           // max 5 bytes long
        age: u8,                // min 3
        pet_names: Vec<String>, // at most 2 items, each at most 10 bytes long
    }

    fn validate_user(user: &User) -> ValidationErrors {
        ValidationErrors::ok()
            .and_field(
                "name",
                ValidationErrors::error_if(user.name.len() > 5, || {
                    Error::with_code("length")
                        .and_message("Illegal string length")
                        .and_param("max", 5)
                        .and_param("value", user.name.len())
                }),
            )
            .and_field(
                "age",
                ValidationErrors::error_if(user.age < 3, || {
                    Error::with_code("range")
                        .and_message("Number not in range")
                        .and_param("min", 3)
                        .and_param("value", user.age)
                }),
            )
            .and_field(
                "pet_names",
                ValidationErrors::ok()
                    .and_error_if(user.pet_names.len() > 2, || {
                        Error::with_code("length")
                            .and_message("Illegal array length")
                            .and_param("max", 2)
                            .and_param("value", user.pet_names.len())
                    })
                    .and_items(user.pet_names.iter(), |_, item| {
                        ValidationErrors::error_if(item.len() > 10, || {
                            Error::with_code("length")
                                .and_message("Illegal string length")
                                .and_param("max", 10)
                                .and_param("value", item.len())
                        })
                    }),
            )
    }

    let user = User {
        name: "easdiuasd&&&&&".into(),
        age: 2,
        pet_names: vec![
            "asdjaiu sdhuyags ydgaysd".into(),
            "aisud 8asydahsbdjabsd".into(),
            "a8 7d8diu h788yhkahsd78".into(),
        ],
    };
    let errors = validate_user(&user);

    assert!(errors.is_err());
    assert_eq!(
        ".age: range: Number not in range: min=3, value=2
.name: length: Illegal string length: max=5, value=14
.pet_names: length: Illegal array length: max=2, value=3
.pet_names[0]: length: Illegal string length: max=10, value=24
.pet_names[1]: length: Illegal string length: max=10, value=21
.pet_names[2]: length: Illegal string length: max=10, value=23",
        errors.to_string()
    );
}
