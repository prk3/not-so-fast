use not_so_fast::*;

fn main() {
    // - Introduction -

    // This example file shows how to write a validator for any data structure.
    // I recommend you read it from start to end, as we build on what we've
    // learned earlier. You can step thought the code with a debugger, to see
    // errors printed one at a time.

    // - Primitives -

    // We will start simple. Let's write a validator that checks if some age
    // is not greater than 150.

    fn validate_age(age: &u32) -> ValidationErrors {
        ValidationErrors::error_if(*age > 150, || {
            Error::with_code("range")
                .and_message("Number not in range")
                .and_param("max", 150)
                .and_param("value", *age)
        })
    }

    // `ValidationErrors::error_if` accepts two parameters. The first is a bool
    // saying whether the value is valid or not. If it's not, `error_if` method
    // calls the second parameter - function returning Error. Here we pass a
    // closure returning an error with code, message, and two params, though
    // only the code is required.

    // Now let's check if the validator works as expected. `is_ok` method on
    // ValidationErrors tells us whether value is valid or not.

    assert!(validate_age(&0).is_ok());
    assert!(validate_age(&35).is_ok());
    assert!(validate_age(&70).is_ok());
    assert!(!validate_age(&200).is_ok());

    // Check out how validation error is displayed.

    print!("{}\n\n", validate_age(&200));

    // Ok, seems good so far. Now, let's validate username string.

    fn validate_username(username: &str) -> ValidationErrors {
        ValidationErrors::ok()
            .and_error_if(username.len() < 6, || {
                Error::with_code("byte_length")
                    .and_message("String byte length is not allowed")
                    .and_param("min", 6)
                    .and_param("value", username.len())
            })
            .and_error_if(!username.is_ascii(), || {
                Error::with_code("ascii").and_message("String contains non-ASCII characters")
            })
    }

    // This time we started with no error (ValidationErrors::ok()) and added
    // two checks using `and_error_if`.

    assert!(validate_username("user123").is_ok());
    assert!(validate_username("this IS ok TOO").is_ok());
    assert!(!validate_username("short").is_ok());
    assert!(!validate_username("non-ĄŚĆII name").is_ok());

    // One value can trigger many errors. Check out how they are displayed.

    print!("{}\n\n", validate_username("§€¢"));

    // Usually strings are validated as a whole. However, you may want to treat
    // them as a collection of characters and generate an error for specific
    // characters. It's possible with list validation, described in the next
    // chapter.

    // - Lists -

    // not-so-fast allows us to validate lists of items. The following code
    // constructs an error for every float that's not between 0.0 and 1.0,
    // in the 3-elements-long list.

    fn validate_color_rgb(numbers: &[f32; 3]) -> ValidationErrors {
        ValidationErrors::ok()
            .and_item(
                0,
                ValidationErrors::error_if(!(0.0..=1.0).contains(&numbers[0]), || {
                    Error::with_code("range")
                }),
            )
            .and_item(
                1,
                ValidationErrors::error_if(!(0.0..=1.0).contains(&numbers[1]), || {
                    Error::with_code("range")
                }),
            )
            .and_item(
                2,
                ValidationErrors::error_if(!(0.0..=1.0).contains(&numbers[2]), || {
                    Error::with_code("range")
                }),
            )
    }

    assert!(validate_color_rgb(&[0.0, 0.5, 1.0]).is_ok());
    assert!(!validate_color_rgb(&[0.0, 5.0, -1.0]).is_ok());

    // `add_item` works well if the list has a fixed number of elements.
    // Lists of variable length can be validated using `item` and `and_items`
    // methods, as shown below.

    fn validate_username_list(usernames: &[String]) -> ValidationErrors {
        ValidationErrors::items(usernames.iter(), |_index, username| {
            validate_username(username)
        })
    }

    assert!(validate_username_list(&["ann001".into(), "bob002".into()]).is_ok());
    assert!(!validate_username_list(&["short".into(), "≠»²«³¢".into()]).is_ok());

    // Notice how we've reused username validation function.

    // To validate both the length of the list and all items in it, use code
    // like this.

    fn validate_username_list_2(usernames: &[String]) -> ValidationErrors {
        ValidationErrors::ok()
            .and_error_if(usernames.len() > 2, || Error::with_code("length"))
            .and_items(usernames.iter(), |_index, username| {
                validate_username(username)
            })
    }

    assert!(validate_username_list_2(&["ann001".into(), "bob002".into()]).is_ok());
    assert!(
        !validate_username_list_2(&["ann001".into(), "bob002".into(), "chris003".into()]).is_ok()
    );

    // Nice. Let's see how it's displayed.

    print!(
        "{}\n\n",
        validate_username_list_2(&["ok_username".into(), "short".into(), "»»»bad«««".into(),])
    );

    // - Objects -

    // What is we wanted to validate an entire struct? Methods `field` and
    // `and_field` allow us to do just that.

    struct User {
        username: String,
        age: u32,
        favorite_color_rgb: [f32; 3],
        friends: Vec<String>,
    }

    fn validate_user(user: &User) -> ValidationErrors {
        ValidationErrors::ok()
            .and_field("username", validate_username(&user.username))
            .and_field("age", validate_age(&user.age))
            .and_field(
                "favorite_color_rgb",
                validate_color_rgb(&user.favorite_color_rgb),
            )
            .and_field("friends", validate_username_list_2(&user.friends))
    }

    // Because we've defined validators of user properties before, validator of
    // User struct is trivial. Now let's see it in action.

    let ok_user = User {
        username: "hello_world".into(),
        age: 1,
        favorite_color_rgb: [0.0, 1.0, 0.0],
        friends: vec!["foo_bar".into()],
    };

    let bad_user = User {
        username: "¬.¬".into(),
        age: 2000,
        favorite_color_rgb: [-0.5, 5.0, f32::INFINITY],
        friends: vec!["short".into(), "€".into(), "third³".into()],
    };

    assert!(validate_user(&ok_user).is_ok());
    assert!(!validate_user(&bad_user).is_ok());

    print!("{}\n\n", validate_user(&bad_user));

    // Wonderful. What if an object must uphold some invariant, meaning its
    // properties depend on each other? Objects can have direct errors too!

    fn validate_mean_user(user: &User) -> ValidationErrors {
        validate_user(&user).and_error_if(
            user.username.starts_with("mean") && !user.friends.is_empty(),
            || Error::with_code("mean_user_invariant").and_message("Mean users can't have friends"),
        )
    }

    let mean_user = User {
        username: "mean dan".into(),
        age: 30,
        favorite_color_rgb: [0.0, 0.0, 0.0],
        friends: vec!["nice ethan".into()],
    };

    assert!(!validate_mean_user(&mean_user).is_ok());

    print!("{}\n\n", validate_mean_user(&mean_user));

    // The last element of the validation puzzle is checking dynamic objects
    // (also called maps, dictionaries or associative arrays). They can be
    // validated using `fields` and `and_fields` methods.

    fn validate_product_prices(
        product_prices: &std::collections::HashMap<String, f32>,
    ) -> ValidationErrors {
        ValidationErrors::fields(product_prices.iter(), |_key, value| {
            ValidationErrors::error_if(*value < 0.0, || Error::with_code("range"))
        })
    }

    let mut ok_product_prices = std::collections::HashMap::new();
    ok_product_prices.insert("pizza".into(), 10.0);
    ok_product_prices.insert("apple".into(), 1.5);
    ok_product_prices.insert("chocolate".into(), 3.0);

    let mut bad_product_prices = ok_product_prices.clone();
    bad_product_prices.insert("aether".into(), -10.0);

    assert!(!validate_product_prices(&bad_product_prices).is_ok());

    print!("{}\n\n", validate_product_prices(&bad_product_prices));

    // - Keys -

    // Functions you pass to `items`, `and_items`, `fields`, and `and_fields`
    // methods must accept two parameters: the first is index/key and the
    // second is item/value. It's done like this because we sometimes want
    // to check indexes/keys as well. Here is an example.

    #[derive(Clone)]
    struct OrderedProduct {
        id: String,
        #[allow(unused)]
        quantity: u32,
    }

    fn validate_order(
        order: &std::collections::HashMap<String, OrderedProduct>,
    ) -> ValidationErrors {
        ValidationErrors::fields(order.iter(), |id, product| {
            ValidationErrors::error_if(id != &product.id, || {
                Error::with_code("product_id_invariant")
                    .and_message("Product ID key in order object does not match product items's ID")
            })
        })
    }

    // This validator makes sure that product IDs used as keys in order object
    // match with product IDs held in product objects (order object values).

    let mut ok_order = std::collections::HashMap::new();
    ok_order.insert(
        "mug-23".into(),
        OrderedProduct {
            id: "mug-23".into(),
            quantity: 2,
        },
    );

    assert!(validate_order(&ok_order).is_ok());

    let mut bad_order = ok_order.clone();
    bad_order.insert(
        "pencil-3".into(),
        OrderedProduct {
            id: "bucket-39".into(),
            quantity: 1,
        },
    );

    assert!(!validate_order(&bad_order).is_ok());

    // Notice that, despite accepting two parameters, the error function above
    // can return only one error object. If both index/key and item/value can
    // produce a validation error, it's our responsibility to communicate the
    // source of an error clearly.

    // We could report errors on the object that has invalid keys.

    fn validate_product_prices_alt_1(
        product_prices: &std::collections::HashMap<String, f32>,
    ) -> ValidationErrors {
        ValidationErrors::ok()
            .and_errors(
                product_prices
                    .keys()
                    .filter_map(|name| (!name.is_ascii()).then(|| Error::with_code("ascii"))),
            )
            .and_fields(product_prices.iter(), |_name, price| {
                ValidationErrors::error_if(price.is_infinite(), || {
                    Error::with_code("infinite_price")
                })
            })
    }

    // Or mix add "key" errors to "value" error object.

    fn validate_product_prices_alt_2(
        product_prices: &std::collections::HashMap<String, f32>,
    ) -> ValidationErrors {
        ValidationErrors::fields(product_prices.iter(), |name, price| {
            ValidationErrors::ok()
                .and_error_if(!name.is_ascii(), || Error::with_code("ascii"))
                .and_error_if(price.is_infinite(), || Error::with_code("infinite_price"))
        })
    }

    // Or introduce a fictional object with "key" and "value" fields.

    fn validate_product_prices_alt_3(
        product_prices: &std::collections::HashMap<String, f32>,
    ) -> ValidationErrors {
        ValidationErrors::fields(product_prices.iter(), |name, price| {
            ValidationErrors::ok()
                .and_field(
                    "key",
                    ValidationErrors::error_if(!name.is_ascii(), || Error::with_code("ascii")),
                )
                .and_field(
                    "value",
                    ValidationErrors::error_if(price.is_infinite(), || {
                        Error::with_code("infinite_price")
                    }),
                )
        })
    }

    // There are many possibilities, each producing a different error message.
    // You have to choose strategy that makes the most sense for you.

    let mut terrible_product_prices = std::collections::HashMap::new();
    terrible_product_prices.insert("½".into(), f32::INFINITY);

    print!(
        "{}\n\n",
        validate_product_prices_alt_1(&terrible_product_prices)
    );
    print!(
        "{}\n\n",
        validate_product_prices_alt_2(&terrible_product_prices)
    );
    print!(
        "{}\n\n",
        validate_product_prices_alt_3(&terrible_product_prices)
    );

    // - Parametrization -

    // Since manually written validators are regular, unconstrained functions,
    // there is nothing preventing us from parameterizing them.

    struct Profile {
        name: String,
        bio: String,
        images: Vec<String>,
    }

    fn validate_bio(bio: &str, max_char_length: usize) -> ValidationErrors {
        let len = bio.chars().count();
        ValidationErrors::error_if(len > max_char_length, || {
            Error::with_code("length")
                .and_message("Illegal string length")
                .and_param("max", max_char_length)
                .and_param("value", len)
        })
    }

    fn validate_images(images: &[String], multi_image: bool) -> ValidationErrors {
        let limit = if multi_image { 10 } else { 1 };
        ValidationErrors::error_if(images.len() > limit, || {
            Error::with_code("length")
                .and_message("Illegal list length")
                .and_param("max", limit)
                .and_param("value", images.len())
        })
    }

    fn validate_profile(
        profile: &Profile,
        max_bio_char_length: usize,
        multi_image: bool,
    ) -> ValidationErrors {
        ValidationErrors::ok()
            .and_field("name", validate_username(&profile.name))
            .and_field("bio", validate_bio(&profile.bio, max_bio_char_length))
            .and_field("images", validate_images(&profile.images, multi_image))
    }

    let profile = Profile {
        name: "foo_bar_3".into(),
        bio: "x".repeat(1000),
        images: vec!["one.jpg".into(), "two.jpg".into(), "three.jpg".into()],
    };

    assert!(validate_profile(&profile, 2000, true).is_ok());
    assert!(validate_profile(&profile, 2000, false).is_err());
    assert!(validate_profile(&profile, 300, true).is_err());
    assert!(validate_profile(&profile, 300, false).is_err());

    // To make your validators compatible with Validator derive macro, place
    // validator parameters after the reference to value.

    // - Merging -

    // Sometimes it's convenient to split validation logic into two or more
    // functions. In case of comments, one validator could check the length
    // of the comment text, while the other could look for illegal words. To
    // merge two errors together, use `merge` method.

    struct Comment {
        author: String,
        text: String,
    }

    fn validate_text_length(text: &str) -> ValidationErrors {
        ValidationErrors::error_if(text.len() > 500, || {
            Error::with_code("byte_length").and_message("Illegal string byte length")
        })
    }

    fn validate_text_content(text: &str) -> ValidationErrors {
        let illegal_words = ["pineapple", "bash", "truck"];
        let contains_illegal_word = illegal_words.iter().any(|word| text.contains(word));

        ValidationErrors::error_if(contains_illegal_word, || {
            Error::with_code("illegal_word").and_message("Text contains illegal word")
        })
    }

    fn validate_comment(comment: &Comment) -> ValidationErrors {
        ValidationErrors::ok()
            .and_field("author", validate_username(&comment.author))
            .and_field(
                "text",
                ValidationErrors::ok()
                    .merge(validate_text_length(&comment.text))
                    .merge(validate_text_content(&comment.text)),
            )
    }

    let bad_comment_length = Comment {
        author: "ok".into(),
        text: "x".repeat(1000),
    };

    let bad_comment_content = Comment {
        author: "ok".into(),
        text: "I love pineapple pizza".into(),
    };

    assert!(validate_comment(&bad_comment_length).is_err());
    assert!(validate_comment(&bad_comment_content).is_err());

    // `merge` method not only moves all value errors from the error argument
    // to `self`, but also recursively combines field/item errors.
}
