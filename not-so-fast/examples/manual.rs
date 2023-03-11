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

    fn validate_age(age: &u32) -> ValidationNode {
        ValidationNode::error_if(*age > 150, || {
            ValidationError::with_code("range")
                .and_message("Number not in range")
                .and_param("max", 150)
                .and_param("value", *age)
        })
    }

    // `ValidationNode::error_if` accepts two parameters. The first is a bool
    // saying whether the value is valid or not. If it's not, `error_if` method
    // calls the second parameter - function returning Error. Here we pass a
    // closure returning an error with code, message, and two params, though
    // only the code is required.

    // Now let's check if the validator works as expected. `is_ok` method on
    // ValidationNode tells us whether value is valid or not.

    assert!(validate_age(&0).is_ok());
    assert!(validate_age(&35).is_ok());
    assert!(validate_age(&70).is_ok());
    assert!(validate_age(&200).is_err());

    // Check out how validation error is displayed.

    print!("{}\n\n", validate_age(&200));

    // Ok, seems good so far. Now, let's validate username string.

    fn validate_username(username: &str) -> ValidationNode {
        ValidationNode::ok()
            .and_error_if(username.len() < 6, || {
                ValidationError::with_code("byte_length")
                    .and_message("String byte length is not allowed")
                    .and_param("min", 6)
                    .and_param("value", username.len())
            })
            .and_error_if(!username.is_ascii(), || {
                ValidationError::with_code("ascii")
                    .and_message("String contains non-ASCII characters")
            })
    }

    // This time we started with no error (ValidationNode::ok()) and added
    // two checks using `and_error_if`.

    assert!(validate_username("user123").is_ok());
    assert!(validate_username("this IS ok TOO").is_ok());
    assert!(validate_username("short").is_err());
    assert!(validate_username("non-ĄŚĆII name").is_err());

    // One value can trigger many errors. Check out how they are displayed.

    print!("{}\n\n", validate_username("§€¢"));

    // Usually strings are validated as a whole. However, you may want to treat
    // them as a collection of characters and generate an error for specific
    // characters. It's possible with list validation, described in the next
    // chapter.

    // - Lists -

    // not-so-fast allows us to validate lists of items. The following code
    // constructs an error for every float that's not between 0.0 and 1.0,
    // in a 3-elements-long list.

    fn validate_color_rgb(numbers: &[f32; 3]) -> ValidationNode {
        ValidationNode::ok()
            .and_item(
                0,
                ValidationNode::error_if(!(0.0..=1.0).contains(&numbers[0]), || {
                    ValidationError::with_code("range")
                }),
            )
            .and_item(
                1,
                ValidationNode::error_if(!(0.0..=1.0).contains(&numbers[1]), || {
                    ValidationError::with_code("range")
                }),
            )
            .and_item(
                2,
                ValidationNode::error_if(!(0.0..=1.0).contains(&numbers[2]), || {
                    ValidationError::with_code("range")
                }),
            )
    }

    assert!(validate_color_rgb(&[0.0, 0.5, 1.0]).is_ok());
    assert!(validate_color_rgb(&[0.0, 5.0, -1.0]).is_err());

    // `add_item` works well if the list has a fixed number of elements.
    // Lists of variable length can be validated using `item` and `and_items`
    // methods, as shown below.

    fn validate_username_list(usernames: &[String]) -> ValidationNode {
        ValidationNode::items(usernames.iter(), |_index, username| {
            validate_username(username)
        })
    }

    assert!(validate_username_list(&["ann001".into(), "bob002".into()]).is_ok());
    assert!(validate_username_list(&["short".into(), "≠»²«³¢".into()]).is_err());

    // Notice how we've reused username validation function.

    // To validate both the length of the list and all items in it, use code
    // like this.

    fn validate_username_list_2(usernames: &[String]) -> ValidationNode {
        ValidationNode::ok()
            .and_error_if(usernames.len() > 2, || ValidationError::with_code("length"))
            .and_items(usernames.iter(), |_index, username| {
                validate_username(username)
            })
    }

    assert!(validate_username_list_2(&["ann001".into(), "bob002".into()]).is_ok());
    assert!(
        validate_username_list_2(&["ann001".into(), "bob002".into(), "chris003".into()]).is_err()
    );

    // Nice. Let's see how it's displayed.

    print!(
        "{}\n\n",
        validate_username_list_2(&["ok_username".into(), "short".into(), "»»»bad«««".into(),])
    );

    // - Objects -

    // What is we wanted to validate an entire struct? Methods `field` and
    // `and_field` allow us to do just that.

    #[derive(Clone)]
    struct User {
        username: String,
        age: u32,
        favorite_color_rgb: [f32; 3],
        friends: Vec<String>,
    }

    fn validate_user(user: &User) -> ValidationNode {
        ValidationNode::ok()
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
    assert!(validate_user(&bad_user).is_err());

    print!("{}\n\n", validate_user(&bad_user));

    // Wonderful. What if an object must uphold some invariant, meaning its
    // properties depend on each other? We can add an error at the object level.

    fn validate_mean_user(user: &User) -> ValidationNode {
        validate_user(user).and_error_if(
            user.username.starts_with("mean") && !user.friends.is_empty(),
            || {
                ValidationError::with_code("mean_user_invariant")
                    .and_message("Mean users can't have friends")
            },
        )
    }

    let mean_user = User {
        username: "mean dan".into(),
        age: 30,
        favorite_color_rgb: [0.0, 0.0, 0.0],
        friends: vec!["nice ethan".into()],
    };

    assert!(validate_mean_user(&mean_user).is_err());

    print!("{}\n\n", validate_mean_user(&mean_user));

    // The last element of the validation puzzle is checking dynamic objects
    // (also called maps, dictionaries or associative arrays). They can be
    // validated using `fields` and `and_fields` methods.

    fn validate_product_prices(
        product_prices: &std::collections::HashMap<String, f32>,
    ) -> ValidationNode {
        ValidationNode::fields(product_prices.iter(), |_key, value| {
            ValidationNode::error_if(*value < 0.0, || ValidationError::with_code("range"))
        })
    }

    let mut ok_product_prices = std::collections::HashMap::new();
    ok_product_prices.insert("pizza".into(), 10.0);
    ok_product_prices.insert("apple".into(), 1.5);
    ok_product_prices.insert("chocolate".into(), 3.0);

    let mut bad_product_prices = ok_product_prices.clone();
    bad_product_prices.insert("aether".into(), -10.0);

    assert!(validate_product_prices(&bad_product_prices).is_err());

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

    fn validate_order(order: &std::collections::HashMap<String, OrderedProduct>) -> ValidationNode {
        ValidationNode::fields(order.iter(), |id, product| {
            ValidationNode::error_if(id != &product.id, || {
                ValidationError::with_code("product_id_invariant")
                    .and_message("Product ID key in order object does not match product's ID")
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

    assert!(validate_order(&bad_order).is_err());

    // Notice that, despite accepting two parameters, the error function above
    // can return only one error object. If both index/key and item/value can
    // produce a validation error, it's our responsibility to communicate the
    // source of an error clearly.

    // We could report errors on the object that has invalid keys.

    fn validate_product_prices_alt_1(
        product_prices: &std::collections::HashMap<String, f32>,
    ) -> ValidationNode {
        ValidationNode::ok()
            .and_errors(
                product_prices.keys().filter_map(|name| {
                    (!name.is_ascii()).then(|| ValidationError::with_code("ascii"))
                }),
            )
            .and_fields(product_prices.iter(), |_name, price| {
                ValidationNode::error_if(price.is_infinite(), || {
                    ValidationError::with_code("infinite_price")
                })
            })
    }

    // Or mix add "key" errors to "value" error object.

    fn validate_product_prices_alt_2(
        product_prices: &std::collections::HashMap<String, f32>,
    ) -> ValidationNode {
        ValidationNode::fields(product_prices.iter(), |name, price| {
            ValidationNode::ok()
                .and_error_if(!name.is_ascii(), || ValidationError::with_code("ascii"))
                .and_error_if(price.is_infinite(), || {
                    ValidationError::with_code("infinite_price")
                })
        })
    }

    // Or introduce a fictional object with "key" and "value" fields.

    fn validate_product_prices_alt_3(
        product_prices: &std::collections::HashMap<String, f32>,
    ) -> ValidationNode {
        ValidationNode::fields(product_prices.iter(), |name, price| {
            ValidationNode::ok()
                .and_field(
                    "key",
                    ValidationNode::error_if(!name.is_ascii(), || {
                        ValidationError::with_code("ascii")
                    }),
                )
                .and_field(
                    "value",
                    ValidationNode::error_if(price.is_infinite(), || {
                        ValidationError::with_code("infinite_price")
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

    fn validate_bio(bio: &str, max_char_length: usize) -> ValidationNode {
        let len = bio.chars().count();
        ValidationNode::error_if(len > max_char_length, || {
            ValidationError::with_code("length")
                .and_message("Illegal string length")
                .and_param("max", max_char_length)
                .and_param("value", len)
        })
    }

    fn validate_images(images: &[String], multi_image: bool) -> ValidationNode {
        let limit = if multi_image { 10 } else { 1 };
        ValidationNode::error_if(images.len() > limit, || {
            ValidationError::with_code("length")
                .and_message("Illegal list length")
                .and_param("max", limit)
                .and_param("value", images.len())
        })
    }

    fn validate_profile(
        profile: &Profile,
        max_bio_char_length: usize,
        multi_image: bool,
    ) -> ValidationNode {
        ValidationNode::ok()
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

    fn validate_text_length(text: &str) -> ValidationNode {
        ValidationNode::error_if(text.len() > 500, || {
            ValidationError::with_code("byte_length").and_message("Illegal string byte length")
        })
    }

    fn validate_text_content(text: &str) -> ValidationNode {
        let illegal_words = ["pineapple", "bash", "truck"];
        let contains_illegal_word = illegal_words.iter().any(|word| text.contains(word));

        ValidationNode::error_if(contains_illegal_word, || {
            ValidationError::with_code("illegal_word").and_message("Text contains illegal word")
        })
    }

    fn validate_comment(comment: &Comment) -> ValidationNode {
        ValidationNode::ok()
            .and_field("author", validate_username(&comment.author))
            .and_field(
                "text",
                ValidationNode::ok()
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

    // - Traits -

    // Until now we've been writing stand-alone validation functions. To
    // associate a validation function with the validated type, implement
    // `ValidateArgs` for that type.

    impl<'arg> ValidateArgs<'arg> for User {
        type Args = ();

        fn validate_args(&self, (): Self::Args) -> ValidationNode {
            validate_user(self)
        }
    }

    // Associated type `Args` is a tuple with validation parameters. In the
    // example above the tuple has zero elements, since user validation
    // does not expect any arguments.

    // `Profile` does have validation parameters, so the implementation of
    // `ValidateArgs` trait would look like this.

    impl<'arg> ValidateArgs<'arg> for Profile {
        type Args = (usize, bool);

        fn validate_args(&self, (max_bio_len, multi_image): Self::Args) -> ValidationNode {
            validate_profile(self, max_bio_len, multi_image)
        }
    }

    // The generic lifetime 'arg can be used to pass expensive-to-copy argument
    // to the validator by reference.

    struct Shape(String);

    impl<'arg> ValidateArgs<'arg> for Shape {
        type Args = (&'arg [&'arg str],);

        fn validate_args(&self, (legal_shapes,): Self::Args) -> ValidationNode {
            ValidationNode::error_if(!legal_shapes.contains(&self.0.as_ref()), || {
                ValidationError::with_code("illegal_shape")
            })
        }
    }

    // To validate data through `ValidateArgs` implementation, just call
    // `validate_args` method providing necessary arguments.

    const LEGAL_SHAPES_2D: [&str; 3] = ["triangle", "square", "circle"];
    const LEGAL_SHAPES_3D: [&str; 3] = ["sphere", "magenta", "yellow"];

    assert!(Shape("square".into())
        .validate_args((&LEGAL_SHAPES_2D[..],))
        .is_ok());

    assert!(Shape("square".into())
        .validate_args((&LEGAL_SHAPES_3D[..],))
        .is_err());

    assert!(ok_user.validate_args(()).is_ok());

    // Because validators usually aren't parameterized, types implementing
    // `ValidateArgs<Args=()>` automatically implement simpler `Validate` trait.
    // Using `Validate` in trait bounds is preferred, as it shortens your code.

    fn clone_and_validate_1<'a, T: Clone + ValidateArgs<'a, Args = ()>>(
        value: &T,
    ) -> Result<T, ValidationNode> {
        let clone: T = value.clone();
        clone.validate_args(()).result()?;
        Ok(clone)
    }

    assert!(clone_and_validate_1(&ok_user).is_ok());

    fn clone_and_validate_2<T: Clone + Validate>(value: &T) -> Result<T, ValidationNode> {
        let clone: T = value.clone();
        clone.validate().result()?;
        Ok(clone)
    }

    assert!(clone_and_validate_2(&bad_user).is_err());
}
