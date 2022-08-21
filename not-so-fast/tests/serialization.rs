use not_so_fast::*;

#[test]
fn simple() {
    let errors = ValidationErrors::ok()
        .and_error(
            Error::with_code("one")
                .and_message("Test message one")
                .and_param("param1", "value1".into()),
        )
        .and_field(
            "field_a",
            ValidationErrors::ok()
                .and_error(Error::with_code("two"))
                .and_error(Error::with_code("three")),
        )
        .and_field(
            "field_b",
            ValidationErrors::ok()
                .and_item(0, ValidationErrors::error(Error::with_code("four")))
                .and_item(1, ValidationErrors::error(Error::with_code("five")))
                .and_item(1, ValidationErrors::error(Error::with_code("six"))),
        )
        .and_field(
            "field_c_~!@#$%^&*()_+",
            ValidationErrors::error(Error::with_code("seven")),
        );

    assert_eq!(
        serde_json::json!([
            [".", "one: Test message one: param1=value1"],
            [".field_a", "two"],
            [".field_a", "three"],
            [".field_b[0]", "four"],
            [".field_b[1]", "five"],
            [".field_b[1]", "six"],
            [".\"field_c_~!@#$%^&*()_+\"", "seven"],
        ]),
        serde_json::to_value(&errors).unwrap()
    );
}
