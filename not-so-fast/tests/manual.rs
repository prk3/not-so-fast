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

    fn validate_user(user: &User) -> ValidationNode {
        ValidationNode::ok()
            .and_field(
                "name",
                ValidationNode::error_if(user.name.len() > 5, || {
                    ValidationError::with_code("length")
                        .and_message("Illegal string length")
                        .and_param("max", 5)
                        .and_param("value", user.name.len())
                }),
            )
            .and_field(
                "age",
                ValidationNode::error_if(user.age < 3, || {
                    ValidationError::with_code("range")
                        .and_message("Number not in range")
                        .and_param("min", 3)
                        .and_param("value", user.age)
                }),
            )
            .and_field(
                "pet_names",
                ValidationNode::ok()
                    .and_error_if(user.pet_names.len() > 2, || {
                        ValidationError::with_code("length")
                            .and_message("Illegal array length")
                            .and_param("max", 2)
                            .and_param("value", user.pet_names.len())
                    })
                    .and_items(user.pet_names.iter(), |_, item| {
                        ValidationNode::error_if(item.len() > 10, || {
                            ValidationError::with_code("length")
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

#[test]
fn stateful_item_validation() {
    fn validate_unique_numbers(numbers: &[i32]) -> ValidationNode {
        let mut numbers_seen = std::collections::HashSet::new();

        ValidationNode::items(numbers.iter(), |index, item| {
            let first_occurrence = numbers_seen.insert(item);
            ValidationNode::error_if(!first_occurrence, || {
                ValidationError::with_code("non_unique")
                    .and_message(format!("Number {item} at position {index} is non-unique"))
            })
        })
    }

    assert!(validate_unique_numbers(&[1, 4, 5, 6, 8, 9]).is_ok());
    assert!(validate_unique_numbers(&[1, 2, 3, 2, 4, 5, 6, 7]).is_err());
}
