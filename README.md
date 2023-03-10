# not-so-fast

A Rust library for data validation.

## Features

- Builder-pattern API for reporting validation errors
- Easy composition of validators
- Derive macro implementing validation traits for structs and enums
- Error display with `jq`-like paths to bad values
- Error serialization reflecting input data structure

## Installation

```bash
cargo add not-so-fast # --features derive serde
```

Available cargo features:

- `derive` - enables `Validate` derive macro, disabled by default
- `serde` - enables `serde::Serialize` implementation for `ValidationNode`, disabled by default

## Usage

```rust
use not_so_fast::{Validate, ValidationNode, ValidationError};

#[derive(Validate)]
struct User {
    #[validate(custom = alpha_only, char_length(max = 30))]
    nick: String,
    #[validate(range(min = 15, max = 100))]
    age: u8,
    #[validate(length(max = 3), items(char_length(max = 50)))]
    cars: Vec<String>,
}

fn alpha_only(s: &str) -> ValidationNode {
    ValidationNode::error_if(
        s.chars().any(|c| !c.is_alphanumeric()),
        || ValidationError::with_code("alpha_only")
    )
}

let user = User {
    nick: "**tom1980**".into(),
    age: 200,
    cars: vec![
        "first".into(),
        "second".into(),
        "third".repeat(11),
        "fourth".into(),
    ],
};

let node = user.validate();
assert!(node.is_err());
assert_eq!(
    vec![
        ".age: range: Number not in range: max=100, min=15, value=200",
        ".cars: length: Invalid length: max=3, value=4",
        ".cars[2]: char_length: Invalid character length: max=50, value=55",
        ".nick: alpha_only",
    ].join("\n"),
    node.to_string()
);
```

## Guides

[not-so-fast/examples/manual.rs](not-so-fast/examples/manual.rs) explains how to write custom validators.

[not-so-fast/examples/derive.rs](not-so-fast/examples/derive.rs) shows how to use Validator derive macro.

## Compared to validator

`not-so-fast` attempts to fix issues I stumbled upon when working with the popular [https://github.com/Keats/validator](validator) crate. APIs of the two libraries are similar, but not compatible. Here are the differences of `not-so-fast`:

- Validator composition - Composing validators in `not-so-fast` is simple, since all values - numbers, strings, objects, lists - report validation errors using the same type - `ValidationNode`.
- Essential validators - `not-so-fast` comes with only the essential validators. You're expected to write custom validators to test data against your domain's rules.
- Enum support - `Validate` derive macro works with structs and enums.
- Duck typing - `Validate` derive macro does not look at field types whatsoever. To validate data inside containers, you give derive macro hints on how to access the data (`some`, `items`, `fields` attributes). In return, `not-so-fast` will work with third-party container types, as long as they have std-like API.

## TODO

- Add `matches` derive validator for testing strings against regular expressions
- Add `is_some`/`required` derive validator for testing options
- Consider moving derive validators from `not-so-fast-derive` to `not-so-fast` for better code reusability

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
