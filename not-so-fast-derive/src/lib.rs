use parse::*;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{Data, DeriveInput, Fields, Index};

mod parse;

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
            let value_validators = type_custom_validators.into_iter().map(|validator| {
                let function = validator.function;
                let args = validator.args;
                quote! { .merge(#function(self, #(#args),*)) }
            });

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

                let (variant_fields, variant_validators) = match variant.fields {
                    Fields::Named(_) => {
                        let names = variant.fields.iter().map(|field| {
                            field.ident.as_ref().expect("Named field should have ident")
                        });
                        let field_validators =
                            validate_fields(&variant.fields, &variant_name, false)?;
                        (
                            Some(quote! { {#(#names),*} }),
                            Some(quote! { #(#field_validators)* }),
                        )
                    }
                    Fields::Unnamed(_) => {
                        let names = (0..variant.fields.len())
                            .map(|i| Ident::new(&format!("field{i}"), variant_name.span()));
                        let field_validators =
                            validate_fields(&variant.fields, &variant_name, false)?;
                        (
                            Some(quote! { (#(#names),*) }),
                            Some(quote! { #(#field_validators)* }),
                        )
                    }
                    Fields::Unit => (None, None),
                };

                branches.push(quote! {
                    #type_name::#variant_name #variant_fields =>
                        ::not_so_fast::ValidationErrors::ok()
                            #variant_validators
                })
            }

            let variants_validator =
                (!branches.is_empty()).then(|| quote! { .merge(match self { #(#branches),* }) });

            Ok(quote! {
                impl<'arg, #(#generics_full),*> ::not_so_fast::ValidateArgs<'arg> for #type_name<#(#generics_short),*> {
                    type Args = #args_type;

                    fn validate_args(&self, args: Self::Args) -> ::not_so_fast::ValidationErrors {
                        #args_destructure
                        ::not_so_fast::ValidationErrors::ok()
                            #(#value_validators)*
                            #variants_validator
                    }
                }
            })
        }
        Data::Struct(data_struct) => {
            let value_validators = type_custom_validators.into_iter().map(|validator| {
                let function = validator.function;
                let args = validator.args;
                quote! { .merge(#function(&self, #(#args),*)) }
            });

            let field_validators = validate_fields(&data_struct.fields, &type_name, true)?;

            Ok(quote! {
                impl<'arg, #(#generics_full),*> ::not_so_fast::ValidateArgs<'arg> for #type_name<#(#generics_short),*> {
                    type Args = #args_type;

                    fn validate_args(&self, args: Self::Args) -> ::not_so_fast::ValidationErrors {
                        #args_destructure
                        ::not_so_fast::ValidationErrors::ok()
                            #(#value_validators)*
                            #(#field_validators)*
                    }
                }
            })
        }
        _ => panic!("Only structs and enums supported"),
    }
}

fn validate_fields(
    fields: &Fields,
    type_ident: &Ident,
    in_struct: bool,
) -> Result<Vec<TokenStream2>, syn::Error> {
    let mut field_validators = Vec::new();

    for (i, field) in fields.iter().enumerate() {
        let mut value_validators = Vec::new();

        for attr in &field.attrs {
            if attr.path.get_ident().map_or(false, |i| i == "validate") {
                let arguments = attr.parse_args::<FieldValidateArguments>()?;
                for argument in arguments.arguments {
                    let path = match (&field.ident, in_struct) {
                        (Some(ident), true) => quote! { &self.#ident },
                        (None, true) => {
                            let index = Index::from(i);
                            quote! { &self.#index }
                        }
                        (Some(ident), false) => quote! { #ident },
                        (None, false) => {
                            let name = Ident::new(&format!("field{i}"), type_ident.span().clone());
                            quote! { #name }
                        }
                    };
                    value_validators.push(validate_field(path, argument));
                }
            }
        }
        if !value_validators.is_empty() {
            if let Some(ident) = &field.ident {
                let field_name = ident.to_string();
                field_validators.push(quote!(
                    .and_field(#field_name, ValidationErrors::ok()#(#value_validators)*)
                ));
            } else {
                field_validators.push(quote!(
                    .and_item(#i, ValidationErrors::ok()#(#value_validators)*)
                ));
            }
        }
    }

    Ok(field_validators)
}

fn validate_field(path: TokenStream2, argument: FieldValidateArgument) -> TokenStream2 {
    use FieldValidateArgument::*;
    match argument {
        Email(_) => {
            quote! { .merge(::not_so_fast::validators::email::validate_email(#path)) }
        }
        Length(_, LengthArguments { min, max, equal }) => match (&min, &max, &equal) {
            (Some(LengthArgument { value: min, .. }), None, None) => quote! {
                .merge({
                    let notsofast_min = #min;
                    let notsofast_length = (#path).len();
                    ::not_so_fast::ValidationErrors::error_if(
                        notsofast_length < notsofast_min,
                        || ::not_so_fast::Error::with_code("length")
                            .and_message("Invalid length")
                            .and_param("value", notsofast_length)
                            .and_param("min", notsofast_min)
                    )
                })
            },
            (None, Some(LengthArgument { value: max, .. }), None) => quote! {
                .merge({
                    let notsofast_max = #max;
                    let notsofast_length = (#path).len();
                    ::not_so_fast::ValidationErrors::error_if(
                        notsofast_length > notsofast_max,
                        || ::not_so_fast::Error::with_code("length")
                            .and_message("Invalid length")
                            .and_param("value", notsofast_length)
                            .and_param("max", notsofast_max)
                    )
                })
            },
            (
                Some(LengthArgument { value: min, .. }),
                Some(LengthArgument { value: max, .. }),
                None,
            ) => quote! {
                .merge({
                    let notsofast_min = #min;
                    let notsofast_max = #max;
                    let notsofast_length = (#path).len();
                    ::not_so_fast::ValidationErrors::error_if(
                        notsofast_length < notsofast_min || notsofast_length > notsofast_max,
                        || ::not_so_fast::Error::with_code("length")
                            .and_message("Invalid length")
                            .and_param("value", notsofast_length)
                            .and_param("min", notsofast_min)
                            .and_param("max", notsofast_max)
                    )
                })
            },
            (None, None, Some(LengthArgument { value: equal, .. })) => quote! {
                .merge({
                    let notsofast_equal = #equal;
                    let notsofast_length = (#path).len();
                    ::not_so_fast::ValidationErrors::error_if(
                        notsofast_length != notsofast_equal,
                        || ::not_so_fast::Error::with_code("length")
                            .and_message("Invalid length")
                            .and_param("value", notsofast_length)
                            .and_param("equal", notsofast_equal)
                    )
                })
            },
            _ => unreachable!(),
        },
        CharLength(_, LengthArguments { min, max, equal }) => match (&min, &max, &equal) {
            (Some(LengthArgument { value: min, .. }), None, None) => quote! {
                .merge({
                    let notsofast_min = #min;
                    let notsofast_char_length = (#path).chars().count();
                    ::not_so_fast::ValidationErrors::error_if(
                        notsofast_char_length < notsofast_min,
                        || ::not_so_fast::Error::with_code("char_length")
                            .and_message("Invalid character length")
                            .and_param("value", notsofast_char_length)
                            .and_param("min", notsofast_min)
                    )
                })
            },
            (None, Some(LengthArgument { value: max, .. }), None) => quote! {
                .merge({
                    let notsofast_max = #max;
                    let notsofast_char_length = (#path).chars().count();
                    ::not_so_fast::ValidationErrors::error_if(
                        notsofast_char_length > notsofast_max,
                        || ::not_so_fast::Error::with_code("char_length")
                            .and_message("Invalid character length")
                            .and_param("value", notsofast_char_length)
                            .and_param("max", notsofast_max)
                    )
                })
            },
            (
                Some(LengthArgument { value: min, .. }),
                Some(LengthArgument { value: max, .. }),
                None,
            ) => quote! {
                .merge({
                    let notsofast_min = #min;
                    let notsofast_max = #max;
                    let notsofast_char_length = (#path).chars().count();
                    ::not_so_fast::ValidationErrors::error_if(
                        notsofast_char_length < notsofast_min || notsofast_char_length > notsofast_max,
                        || ::not_so_fast::Error::with_code("char_length")
                            .and_message("Invalid character length")
                            .and_param("value", notsofast_char_length)
                            .and_param("min", notsofast_min)
                            .and_param("max", notsofast_max)
                    )
                })
            },
            (None, None, Some(LengthArgument { value: equal, .. })) => quote! {
                .merge({
                    let notsofast_equal = #equal;
                    let notsofast_char_length = (#path).chars().count();
                    ::not_so_fast::ValidationErrors::error_if(
                        notsofast_char_length != notsofast_equal,
                        || ::not_so_fast::Error::with_code("char_length")
                            .and_message("Invalid character length")
                            .and_param("value", notsofast_char_length)
                            .and_param("equal", notsofast_equal)
                    )
                })
            },
            _ => unreachable!(),
        },
        Range(_, RangeArguments { min, max }) => match (min, max) {
            (Some(RangeArgument { value: min, .. }), None) => quote! {
                .merge({
                    let notsofast_min = #min;
                    ::not_so_fast::ValidationErrors::error_if(
                        *(#path) < notsofast_min,
                        || ::not_so_fast::Error::with_code("range")
                            .and_message("Number not in range")
                            .and_param("value", *(#path))
                            .and_param("min", notsofast_min)
                    )
                })
            },
            (None, Some(RangeArgument { value: max, .. })) => quote! {
                .merge({
                    let notsofast_max = #max;
                    ::not_so_fast::ValidationErrors::error_if(
                        *(#path) > notsofast_max,
                        || ::not_so_fast::Error::with_code("range")
                            .and_message("Number not in range")
                            .and_param("value", *(#path))
                            .and_param("max", notsofast_max)
                    )
                })
            },
            (Some(RangeArgument { value: min, .. }), Some(RangeArgument { value: max, .. })) => {
                quote! {
                    .merge({
                        let notsofast_min = #min;
                        let notsofast_max = #max;
                        ::not_so_fast::ValidationErrors::error_if(
                            *(#path) < notsofast_min || *(#path) > notsofast_max,
                            || ::not_so_fast::Error::with_code("range")
                                .and_message("Number not in range")
                                .and_param("value", *(#path))
                                .and_param("min", notsofast_min)
                                .and_param("max", notsofast_max)
                        )
                    })
                }
            }
            _ => unreachable!(),
        },
        Custom(_, arguments) => {
            let function = arguments.function;
            let args = arguments.args;
            quote! { .merge(#function(#path, #(#args),*)) }
        }
        Nested(_, arguments) => {
            let args = arguments.args;
            let args_tuple = make_tuple(args.as_slice());
            quote! { .merge(::not_so_fast::ValidateArgs::validate_args(#path, #args_tuple)) }
        }
        _ => todo!("implement other field validators"),
    }
}

fn make_tuple<T: ToTokens>(elements: &[T]) -> TokenStream2 {
    match elements.len() {
        1 => quote! { (#(#elements),*,) },
        _ => quote! { (#(#elements),*) },
    }
}
