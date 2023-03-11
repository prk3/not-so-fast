use parse::*;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{Data, DeriveInput, Field, Fields, Index};

mod parse;

/// Implements `ValidateArgs` for structs and enums.
///
/// ## Supported type attributes
///
/// ### args
///
/// Defines `Args` of the `ValidateArgs` implementation.
///
/// ```text
/// #[validate(args(a: i32, b: bool, ...))]
/// ```
///
/// Example:
/// ```
/// # use ::not_so_fast::*;
/// # use ::not_so_fast_derive::Validate;
/// #[derive(Validate)]
/// #[validate(args(max_len: usize))]
/// struct Comment {
///     author_id: u64,
///     #[validate(char_length(max = max_len))]
///     content: String,
/// }
///
/// let comment = Comment {
///     author_id: 1,
///     content: "Great video! ".repeat(10),
/// };
/// assert!(comment.validate_args((150,)).is_ok());
/// assert!(comment.validate_args((100,)).is_err());
/// ```
///
/// ### custom
///
/// Validates the entire struct/enum with a custom validation function.
/// The signature of the function must be `fn(data: &T, args: (A, B, C, ...))
///  -> ValidationNode` if it has validation parameters, or `fn(data: &T, args:
/// (A, B, C, ...)) -> ValidationNode` if it doesn't.
///
/// ```text
/// #[validate(custom = func::path)]
/// #[validate(custom(function = func::path))]
/// #[validate(custom(function = func::path, args=(...)))]
/// ```
///
/// Example:
///
/// ```
/// # use ::not_so_fast::*;
/// # use ::not_so_fast_derive::Validate;
/// #[derive(Validate)]
/// #[validate(custom = validate_comment)]
/// struct Comment {
///     author_id: u64,
///     content: String,
/// }
///
/// fn validate_comment(comment: &Comment) -> ValidationNode {
///     let max_len = if comment.author_id == 0 { 200 } else { 100 };
///     ValidationNode::field("content", ValidationNode::error_if(
///         comment.content.len() > max_len,
///         || ValidationError::with_code("length")
///     ))
/// }
///
/// let super_comment = Comment {
///     author_id: 0,
///     content: "Great video! ".repeat(10),
/// };
///
/// let regular_comment = Comment {
///     author_id: 1,
///     content: "Great video! ".repeat(10),
/// };
///
/// assert!(super_comment.validate().is_ok());
/// assert!(regular_comment.validate().is_err());
/// ```
///
/// ## Supported field attributes
///
/// ### some
///
/// Validates data in `Some` variant of `Option`. Accepts all field arguments.
///
/// ```text
/// #[validate(some)]
/// #[validate(some(...))]
/// ```
///
/// Example:
///
/// ```
/// # use ::not_so_fast::*;
/// # use ::not_so_fast_derive::Validate;
/// #[derive(Validate)]
/// struct Input {
///     #[validate(some(range(max = 10)))]
///     maybe_number: Option<u32>,
/// }
///
/// assert!(Input { maybe_number: None }.validate().is_ok());
/// assert!(Input { maybe_number: Some(5) }.validate().is_ok());
/// assert!(Input { maybe_number: Some(20) }.validate().is_err());
/// ```
///
/// ### items
///
/// Validates all items in a list-like collection. Works with arrays, slices,
/// `Vec`, `VecDeque`, `HashSet`, `BTreeSet`, `LinkedList`.
///
/// ```text
/// #[validate(items)]
/// #[validate(items(...))]
/// ```
///
/// Example:
///
/// ```
/// # use ::not_so_fast::*;
/// # use ::not_so_fast_derive::Validate;
/// #[derive(Validate)]
/// struct Input {
///     #[validate(items(range(max = 10)))]
///     numbers: Vec<u32>,
/// }
///
/// assert!(Input { numbers: vec![] }.validate().is_ok());
/// assert!(Input { numbers: vec![1, 2, 3] }.validate().is_ok());
/// assert!(Input { numbers: vec![6, 1, 50] }.validate().is_err());
/// ```
///
/// ### fields
///
/// Validates all values in a key-value collection. Works with HashMap and
/// BTreeMap.
///
/// ```text
/// #[validate(fields)]
/// #[validate(fields(...))]
/// ```
///
/// Example:
///
/// ```
/// # use ::not_so_fast::*;
/// # use ::not_so_fast_derive::Validate;
/// use std::collections::HashMap;
///
/// #[derive(Validate)]
/// struct Input {
///     #[validate(fields(char_length(max = 10)))]
///     map: HashMap<u32, String>,
/// }
///
/// assert!(Input { map: [].into_iter().collect() }.validate().is_ok());
/// assert!(Input { map: [(1, "hello".into())].into_iter().collect() }.validate().is_ok());
/// assert!(Input { map: [(1, "x".repeat(100))].into_iter().collect() }.validate().is_err());
/// ```
///
/// ### nested
///
/// Validates field using its `ValidateArgs` implementation.
///
/// ```text
/// #[validate]
/// #[validate(nested)]
/// #[validate(nested(args(...)))]
/// ```
///
/// Example:
/// ```
/// # use ::not_so_fast::*;
/// # use ::not_so_fast_derive::Validate;
/// #[derive(Validate)]
/// struct Child {
///     #[validate(range(max = 10))]
///     number: u32,
/// }
///
/// #[derive(Validate)]
/// struct Input {
///     #[validate]
///     child: Child,
/// }
///
/// assert!(Input { child: Child { number: 5 }}.validate().is_ok());
/// assert!(Input { child: Child { number: 20 }}.validate().is_err());
/// ```
///
/// ### custom
///
/// Validates field using a custom validation function. The signature of the
/// function must be `fn(data: &T, args: (A, B, C, ...)) -> ValidationNode` if
/// it has validation parameters, or `fn(data: &T, args: (A, B, C, ...)) ->
/// ValidationNode` if it doesn't.
///
/// ```text
/// #[validate(custom = func::path)]
/// #[validate(custom(function = func::path))]
/// #[validate(custom(function = func::path, args=(...)))]
/// ```
///
/// Example:
///
/// ```
/// # use ::not_so_fast::*;
/// # use ::not_so_fast_derive::Validate;
/// #[derive(Validate)]
/// struct Input {
///     #[validate(custom = validate_username)]
///     username: String,
/// }
///
/// fn validate_username(username: &str) -> ValidationNode {
///     ValidationNode::error_if(
///         !username.chars().all(|a| a.is_alphanumeric()),
///         || ValidationError::with_code("non_alpha"),
///     )
/// }
///
/// assert!(Input { username: "Alex1990".into() }.validate().is_ok());
/// assert!(Input { username: "Bob!!!".into() }.validate().is_err());
/// ```
///
/// ### range
///
/// Checks if a number is in the specified range. Works with all integer and
/// float types.
///
/// ```text
/// #[validate(range(min = expr))]
/// #[validate(range(max = expr))]
/// #[validate(range(min = expr, max = expr))]
/// ```
///
/// Example:
///
/// ```
/// # use ::not_so_fast::*;
/// # use ::not_so_fast_derive::Validate;
/// #[derive(Validate)]
/// struct Input {
///     #[validate(range(min = 1, max = 100))]
///     number: u32,
/// }
///
/// assert!(Input { number: 0 }.validate().is_err());
/// assert!(Input { number: 4 }.validate().is_ok());
/// assert!(Input { number: 110 }.validate().is_err());
/// ```
///
/// ### length
///
/// Validates size of a container. Works with arrays, strings, slices, and all
/// standard container types. String length is measures **in bytes**, not UTF-8
/// characters.
///
/// ```text
/// #[validate(length(min = expr))]
/// #[validate(length(max = expr))]
/// #[validate(length(min = expr, max = expr))]
/// #[validate(length(equal = expr))]
/// ```
///
/// Example:
///
/// ```
/// # use ::not_so_fast::*;
/// # use ::not_so_fast_derive::Validate;
/// #[derive(Validate)]
/// struct Input {
///     #[validate(length(max = 2))]
///     numbers: Vec<u32>,
/// }
///
/// assert!(Input { numbers: vec![1] }.validate().is_ok());
/// assert!(Input { numbers: vec![1, 1] }.validate().is_ok());
/// assert!(Input { numbers: vec![1, 1, 1] }.validate().is_err());
/// ```
///
/// ### char_length
///
/// Validates size of a string measured in UTF-8 characters. Works with strings
/// and string slices.
///
/// ```text
/// #[validate(char_length(min = expr))]
/// #[validate(char_length(max = expr))]
/// #[validate(char_length(min = expr, max = expr))]
/// #[validate(char_length(equal = expr))]
/// ```
///
/// Example:
///
/// ```
/// # use ::not_so_fast::*;
/// # use ::not_so_fast_derive::Validate;
/// #[derive(Validate)]
/// struct Input {
///     #[validate(char_length(max = 5))]
///     username: String,
/// }
///
/// assert!(Input { username: "Chris".into() }.validate().is_ok());
/// assert!(Input { username: "María".into() }.validate().is_ok());
/// assert!(Input { username: "Isabela".into() }.validate().is_err());
/// ```
#[proc_macro_derive(Validate, attributes(validate))]
pub fn derive_validate_args(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let type_: DeriveInput = syn::parse(input).expect("Input should be valid struct or enum");
    expand_validate(type_)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn expand_validate(type_: DeriveInput) -> Result<TokenStream2, syn::Error> {
    let type_name = &type_.ident;

    let lifetimes_full = type_.generics.lifetimes().map(|l| l as &dyn ToTokens);
    let types_full = type_.generics.type_params().map(|t| t as &dyn ToTokens);
    let consts_full = type_.generics.const_params().map(|t| t as &dyn ToTokens);
    let generics_full = lifetimes_full.chain(types_full).chain(consts_full);

    let lifetimes_short = type_
        .generics
        .lifetimes()
        .map(|l| &l.lifetime as &dyn ToTokens);
    let types_short = type_
        .generics
        .type_params()
        .map(|t| &t.ident as &dyn ToTokens);
    let consts_short = type_
        .generics
        .const_params()
        .map(|c| &c.ident as &dyn ToTokens);
    let generics_short = lifetimes_short.chain(types_short).chain(consts_short);

    let mut arg_types = Vec::new();
    let mut arg_names = Vec::new();
    let mut type_custom_validators = Vec::new();

    for attr in &type_.attrs {
        if attr.path.get_ident().map_or(false, |i| i == "validate") {
            let arguments = attr.parse_args::<TypeValidateArguments>()?.arguments;
            for argument in arguments {
                match argument {
                    TypeValidateArgument::Args(_, args) => {
                        arg_names.extend(args.arguments.iter().map(|arg| arg.name.clone()));
                        arg_types.extend(args.arguments.iter().map(|arg| arg.type_.clone()));
                    }
                    TypeValidateArgument::Custom(_, custom) => {
                        type_custom_validators.push(custom);
                    }
                }
            }
        }
    }

    let args_type = make_tuple(arg_types.as_slice());
    let args_destructure = (!arg_names.is_empty()).then(|| {
        let tuple = make_tuple(arg_names.as_slice());
        quote! { let #tuple = args; }
    });

    match &type_.data {
        Data::Enum(data_enum) => {
            let mut branches = Vec::new();

            for variant in &data_enum.variants {
                let variant_name = &variant.ident;

                for attr in &variant.attrs {
                    if attr.path.get_ident().map_or(false, |i| i == "validate") {
                        return Err(syn::Error::new_spanned(
                            attr,
                            "validate attribute can not be applied to enum variants",
                        ));
                    }
                }

                let (variant_fields, variant_field_modifiers) = match variant.fields {
                    Fields::Named(_) => {
                        let names = variant.fields.iter().map(|field| {
                            field.ident.as_ref().expect("Named field should have ident")
                        });
                        (
                            Some(quote! { {#(#names),*} }),
                            modifiers_for_fields(&variant.fields, variant_name, false)?,
                        )
                    }
                    Fields::Unnamed(_) => {
                        let names = (0..variant.fields.len())
                            .map(|i| Ident::new(&format!("field{i}"), variant_name.span()));
                        (
                            Some(quote! { (#(#names),*) }),
                            modifiers_for_fields(&variant.fields, variant_name, false)?,
                        )
                    }
                    Fields::Unit => (None, Vec::new()),
                };

                branches.push(quote! {
                    #type_name::#variant_name #variant_fields =>
                        ::not_so_fast::ValidationNode::ok()
                            #(#variant_field_modifiers)*
                })
            }

            let node_from_custom = |validator: CustomArguments| {
                let function = validator.function;
                let args = validator.args;
                quote! { #function(self, #(#args),*) }
            };

            let combined_node = match (type_custom_validators.is_empty(), branches.is_empty()) {
                (false, false) => {
                    let value_node =
                        merge_nodes(type_custom_validators.into_iter().map(node_from_custom));
                    quote! {
                        #value_node.merge(match self { #(#branches),* })
                    }
                }
                (false, true) => {
                    merge_nodes(type_custom_validators.into_iter().map(node_from_custom))
                }
                (true, false) => {
                    quote! { match self { #(#branches),* } }
                }
                (true, true) => {
                    quote! { ::not_so_fast::ValidationNode::ok() }
                }
            };

            Ok(quote! {
                impl<'arg, #(#generics_full),*> ::not_so_fast::ValidateArgs<'arg> for #type_name<#(#generics_short),*> {
                    type Args = #args_type;

                    fn validate_args(&self, args: Self::Args) -> ::not_so_fast::ValidationNode {
                        #args_destructure
                        #combined_node
                    }
                }
            })
        }
        Data::Struct(data_struct) => {
            let value_node = merge_nodes(type_custom_validators.into_iter().map(|validator| {
                let function = validator.function;
                let args = validator.args;
                quote! { #function(&self, #(#args),*) }
            }));
            let field_modifiers = modifiers_for_fields(&data_struct.fields, type_name, true)?;

            Ok(quote! {
                impl<'arg, #(#generics_full),*> ::not_so_fast::ValidateArgs<'arg> for #type_name<#(#generics_short),*> {
                    type Args = #args_type;

                    fn validate_args(&self, args: Self::Args) -> ::not_so_fast::ValidationNode {
                        #args_destructure
                        #value_node
                            #(#field_modifiers)*
                    }
                }
            })
        }
        _ => panic!("Only structs and enums supported"),
    }
}

fn modifiers_for_fields(
    fields: &Fields,
    type_ident: &Ident,
    in_struct: bool,
) -> Result<Vec<TokenStream2>, syn::Error> {
    match fields {
        Fields::Named(fields) => {
            let mut modifiers = Vec::new();
            for (i, field) in fields.named.iter().enumerate() {
                let ident = field.ident.as_ref().unwrap().to_string();
                if let Some(node) = node_for_field(field, i, type_ident, in_struct)? {
                    modifiers.push(quote! { .and_field(#ident, #node) });
                }
            }
            Ok(modifiers)
        }
        Fields::Unnamed(fields) => {
            let mut modifiers = Vec::new();
            for (i, field) in fields.unnamed.iter().enumerate() {
                if let Some(node) = node_for_field(field, i, type_ident, in_struct)? {
                    modifiers.push(quote! { .and_item(#i, #node) });
                }
            }
            Ok(modifiers)
        }
        Fields::Unit => Ok(Vec::new()),
    }
}

fn node_for_field(
    field: &Field,
    field_index: usize,
    type_ident: &Ident,
    in_struct: bool,
) -> Result<Option<TokenStream2>, syn::Error> {
    let mut nodes = Vec::new();

    for attr in &field.attrs {
        if attr.path.get_ident().map_or(false, |i| i == "validate") {
            let arguments = if attr.tokens.is_empty() {
                FieldValidateArguments::empty()
            } else {
                attr.parse_args::<FieldValidateArguments>()?
            };

            for argument in arguments.arguments {
                let path = match (&field.ident, in_struct) {
                    (Some(ident), true) => quote! { &self.#ident },
                    (None, true) => {
                        let index = Index::from(field_index);
                        quote! { &self.#index }
                    }
                    (Some(ident), false) => quote! { #ident },
                    (None, false) => {
                        let name = Ident::new(&format!("field{field_index}"), type_ident.span());
                        quote! { #name }
                    }
                };
                nodes.push(node_for_field_argument(path, argument));
            }
        }
    }

    Ok((!nodes.is_empty()).then(|| merge_nodes(nodes.into_iter())))
}

fn node_for_field_argument(path: TokenStream2, argument: FieldValidateArgument) -> TokenStream2 {
    use FieldValidateArgument as A;
    match argument {
        A::Some(_, arguments) => {
            let node = merge_nodes(
                arguments
                    .arguments
                    .into_iter()
                    .map(|node| node_for_field_argument(quote! { value }, node)),
            );
            quote! {
                if let Some(value) = #path {
                    #node
                } else {
                    ::not_so_fast::ValidationNode::ok()
                }
            }
        }
        A::Items(_, arguments) => {
            let node = merge_nodes(
                arguments
                    .arguments
                    .into_iter()
                    .map(|node| node_for_field_argument(quote! { item }, node)),
            );
            quote! {
                ::not_so_fast::ValidationNode::items((#path).iter(), |_index, item| {
                    #node
                })
            }
        }
        A::Fields(_, arguments) => {
            let node = merge_nodes(
                arguments
                    .arguments
                    .into_iter()
                    .map(|node| node_for_field_argument(quote! { value }, node)),
            );
            quote! {
                ::not_so_fast::ValidationNode::fields((#path).iter(), |_key, value| {
                    #node
                })
            }
        }
        A::Nested(_, arguments) => {
            let args = arguments.args;
            let args_tuple = make_tuple(args.as_slice());
            quote! { ::not_so_fast::ValidateArgs::validate_args(#path, #args_tuple) }
        }
        A::Custom(_, arguments) => {
            let function = arguments.function;
            let args = arguments.args;
            quote! { #function(#path, #(#args),*) }
        }
        A::Length(_, LengthArguments { min, max, equal }) => match (&min, &max, &equal) {
            (Some(LengthArgument { value: min, .. }), None, None) => quote! {{
                let notsofast_length = (#path).len();
                ::not_so_fast::ValidationNode::error_if(
                    notsofast_length < #min,
                    || ::not_so_fast::ValidationError::with_code("length")
                        .and_message("Invalid length")
                        .and_param("value", notsofast_length)
                        .and_param("min", #min)
                )
            }},
            (None, Some(LengthArgument { value: max, .. }), None) => quote! {{
                let notsofast_length = (#path).len();
                ::not_so_fast::ValidationNode::error_if(
                    notsofast_length > #max,
                    || ::not_so_fast::ValidationError::with_code("length")
                        .and_message("Invalid length")
                        .and_param("value", notsofast_length)
                        .and_param("max", #max)
                )
            }},
            (
                Some(LengthArgument { value: min, .. }),
                Some(LengthArgument { value: max, .. }),
                None,
            ) => quote! {{
                let notsofast_length = (#path).len();
                ::not_so_fast::ValidationNode::error_if(
                    notsofast_length < #min || notsofast_length > #max,
                    || ::not_so_fast::ValidationError::with_code("length")
                        .and_message("Invalid length")
                        .and_param("value", notsofast_length)
                        .and_param("min", #min)
                        .and_param("max", #max)
                )
            }},
            (None, None, Some(LengthArgument { value: equal, .. })) => quote! {{
                let notsofast_length = (#path).len();
                ::not_so_fast::ValidationNode::error_if(
                    notsofast_length != #equal,
                    || ::not_so_fast::ValidationError::with_code("length")
                        .and_message("Invalid length")
                        .and_param("value", notsofast_length)
                        .and_param("equal", #equal)
                )
            }},
            _ => unreachable!(),
        },
        A::CharLength(_, LengthArguments { min, max, equal }) => match (&min, &max, &equal) {
            (Some(LengthArgument { value: min, .. }), None, None) => quote! {{
                let notsofast_char_length = (#path).chars().count();
                ::not_so_fast::ValidationNode::error_if(
                    notsofast_char_length < #min,
                    || ::not_so_fast::ValidationError::with_code("char_length")
                        .and_message("Invalid character length")
                        .and_param("value", notsofast_char_length)
                        .and_param("min", #min)
                )
            }},
            (None, Some(LengthArgument { value: max, .. }), None) => quote! {{
                let notsofast_char_length = (#path).chars().count();
                ::not_so_fast::ValidationNode::error_if(
                    notsofast_char_length > #max,
                    || ::not_so_fast::ValidationError::with_code("char_length")
                        .and_message("Invalid character length")
                        .and_param("value", notsofast_char_length)
                        .and_param("max", #max)
                )
            }},
            (
                Some(LengthArgument { value: min, .. }),
                Some(LengthArgument { value: max, .. }),
                None,
            ) => quote! {{
                let notsofast_char_length = (#path).chars().count();
                ::not_so_fast::ValidationNode::error_if(
                    notsofast_char_length < #min || notsofast_char_length > #max,
                    || ::not_so_fast::ValidationError::with_code("char_length")
                        .and_message("Invalid character length")
                        .and_param("value", notsofast_char_length)
                        .and_param("min", #min)
                        .and_param("max", #max)
                )
            }},
            (None, None, Some(LengthArgument { value: equal, .. })) => quote! {{
                let notsofast_char_length = (#path).chars().count();
                ::not_so_fast::ValidationNode::error_if(
                    notsofast_char_length != #equal,
                    || ::not_so_fast::ValidationError::with_code("char_length")
                        .and_message("Invalid character length")
                        .and_param("value", notsofast_char_length)
                        .and_param("equal", #equal)
                )
            }},
            _ => unreachable!(),
        },
        A::Range(_, RangeArguments { min, max }) => match (min, max) {
            (Some(RangeArgument { value: min, .. }), None) => quote! {
                ::not_so_fast::ValidationNode::error_if(
                    *(#path) < #min,
                    || ::not_so_fast::ValidationError::with_code("range")
                        .and_message("Number not in range")
                        .and_param("value", *(#path))
                        .and_param("min", #min)
                )
            },
            (None, Some(RangeArgument { value: max, .. })) => quote! {
                ::not_so_fast::ValidationNode::error_if(
                    *(#path) > #max,
                    || ::not_so_fast::ValidationError::with_code("range")
                        .and_message("Number not in range")
                        .and_param("value", *(#path))
                        .and_param("max", #max)
                )
            },
            (Some(RangeArgument { value: min, .. }), Some(RangeArgument { value: max, .. })) => {
                quote! {
                    ::not_so_fast::ValidationNode::error_if(
                        *(#path) < #min || *(#path) > #max,
                        || ::not_so_fast::ValidationError::with_code("range")
                            .and_message("Number not in range")
                            .and_param("value", *(#path))
                            .and_param("min", #min)
                            .and_param("max", #max)
                    )
                }
            }
            _ => unreachable!(),
        },
    }
}

fn merge_nodes(mut nodes: impl Iterator<Item = TokenStream2>) -> TokenStream2 {
    if let Some(first_node) = nodes.next() {
        let merges = nodes.map(|node| quote! { .merge(#node) });
        quote! {
            #first_node #(#merges)*
        }
    } else {
        quote! { ::not_so_fast::ValidationNode::ok() }
    }
}

fn make_tuple<T: ToTokens>(elements: &[T]) -> TokenStream2 {
    match elements.len() {
        1 => quote! { (#(#elements),*,) },
        _ => quote! { (#(#elements),*) },
    }
}
