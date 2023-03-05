#[macro_use]
extern crate pretty_assertions;

use not_so_fast::*;

#[test]
fn simple() {
    let errors = ValidationNode::ok()
        .and_error(
            ValidationError::with_code("one")
                .and_message("Test message one")
                .and_param("param1", "value1"),
        )
        .and_field(
            "field_a",
            ValidationNode::ok()
                .and_error(ValidationError::with_code("two"))
                .and_error(ValidationError::with_code("three")),
        )
        .and_field(
            "field_b",
            ValidationNode::ok()
                .and_item(0, ValidationNode::error(ValidationError::with_code("four")))
                .and_item(1, ValidationNode::error(ValidationError::with_code("five")))
                .and_item(1, ValidationNode::error(ValidationError::with_code("six"))),
        )
        .and_field(
            "field_c_~!@#$%^&*()_+",
            ValidationNode::error(ValidationError::with_code("seven")),
        )
        .and_item(
            0,
            ValidationNode::error(ValidationError::with_code("eight")),
        )
        .and_item(
            1,
            ValidationNode::item(2, ValidationNode::error(ValidationError::with_code("nine"))),
        )
        .and_item(
            2,
            ValidationNode::error(
                ValidationError::with_code("c")
                    .and_param("p01", true)
                    .and_param("p02", 1u8)
                    .and_param("p03", 1u16)
                    .and_param("p04", 1u32)
                    .and_param("p05", 1u64)
                    .and_param("p06", 1u128)
                    .and_param("p07", 1i8)
                    .and_param("p08", 1i16)
                    .and_param("p09", 1i32)
                    .and_param("p10", 1i64)
                    .and_param("p11", 1i128)
                    .and_param("p12", 1usize)
                    .and_param("p13", 1.1f32)
                    .and_param("p14", 1.1f64)
                    .and_param("p15", '\n')
                    .and_param("p16", "one\ntwo")
                    .and_param("p17", String::from("three\nfour"))
                    .and_param("p18", ParamValue::Raw("five\nsix".into())),
            ),
        );

    assert_eq!(
        serde_json::json!([
            [".", "one: Test message one: param1=\"value1\""],
            [".field_a", "two"],
            [".field_a", "three"],
            [".field_b[0]", "four"],
            [".field_b[1]", "five"],
            [".field_b[1]", "six"],
            [".\"field_c_~!@#$%^&*()_+\"", "seven"],
            [".[0]", "eight"],
            [".[1][2]", "nine"],
            [
                ".[2]",
                "c: p01=true, p02=1, p03=1, p04=1, p05=1, p06=1, p07=1, p08=1, p09=1, p10=1, p11=1, p12=1, p13=1.1, p14=1.1, p15='\\n', p16=\"one\\ntwo\", p17=\"three\\nfour\", p18=five\nsix"
            ],
        ]),
        serde_json::to_value(&errors).unwrap()
    );
}
