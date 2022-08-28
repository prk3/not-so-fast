use parse::*;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{Data, DeriveInput, Fields, Index};

mod parse;

#[proc_macro_derive(Validate, attributes(validate))]
pub fn derive_validate_args(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let type_: DeriveInput = syn::parse(input).expect("Input should be valid struct or enum");
    let type_name = type_.ident;

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
            let arguments = match attr.parse_args::<TypeValidateArguments>() {
                Ok(arguments) => arguments.arguments,
                Err(e) => return e.to_compile_error().into(),
            };
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

    match type_.data {
        Data::Enum(data_enum) => {
            let value_validators = type_custom_validators.into_iter().map(|validator| {
                let function = validator.function;
                let args = validator.args;
                quote! { .merge(#function(self, #(#args),*)) }
            });

            let variants_validator = (!data_enum.variants.is_empty()).then(|| {
                let mut branches = Vec::new();
                for variant in data_enum.variants {
                    let variant_name = variant.ident;
                    let (variant_fields, variant_validators) = match variant.fields {
                        Fields::Named(_) => {
                            let names = variant.fields.iter().map(|field| {
                                field.ident.as_ref().expect("Named field should have ident")
                            });
                            let field_validators =
                                match validate_fields(&variant.fields, &variant_name, false) {
                                    Ok(validators) => validators,
                                    Err(error) => return error.to_compile_error().into(),
                                };
                            (
                                Some(quote! { {#(#names),*} }),
                                Some(quote! { #(#field_validators)* }),
                            )
                        }
                        Fields::Unnamed(_) => {
                            let names = (0..variant.fields.len())
                                .map(|i| Ident::new(&format!("field{i}"), variant_name.span()));
                            let field_validators =
                                match validate_fields(&variant.fields, &variant_name, false) {
                                    Ok(validators) => validators,
                                    Err(error) => return error.to_compile_error().into(),
                                };
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

                quote! { .merge(match self { #(#branches),* }) }
            });

            quote! {
                impl<'arg, #(#generics_full),*> ::not_so_fast::ValidateArgs<'arg> for #type_name<#(#generics_short),*> {
                    type Args = #args_type;

                    fn validate_args(&self, args: Self::Args) -> ::not_so_fast::ValidationErrors {
                        #args_destructure
                        ::not_so_fast::ValidationErrors::ok()
                            #(#value_validators)*
                            #variants_validator
                    }
                }
            }
            .into()
        }
        Data::Struct(data_struct) => {
            let value_validators = type_custom_validators.into_iter().map(|validator| {
                let function = validator.function;
                let args = validator.args;
                quote! { .merge(#function(&self, #(#args),*)) }
            });

            let field_validators = match validate_fields(&data_struct.fields, &type_name, true) {
                Ok(validators) => validators,
                Err(error) => return error.to_compile_error().into(),
            };

            quote! {
                impl<'arg, #(#generics_full),*> ::not_so_fast::ValidateArgs<'arg> for #type_name<#(#generics_short),*> {
                    type Args = #args_type;

                    fn validate_args(&self, args: Self::Args) -> ::not_so_fast::ValidationErrors {
                        #args_destructure
                        ::not_so_fast::ValidationErrors::ok()
                            #(#value_validators)*
                            #(#field_validators)*
                    }
                }
            }
            .into()
        }
        _ => panic!("Only structs and enums supported"),
    }
}

fn validate_fields(
    fields: &Fields,
    type_ident: &Ident,
    in_struct: bool,
) -> Result<Vec<TokenStream>, syn::Error> {
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
                    .and_field(#field_name, ValidationErrors::ok()#(#value_validators),*)
                ));
            } else {
                field_validators.push(quote!(
                    .and_item(#i, ValidationErrors::ok()#(#value_validators),*)
                ));
            }
        }
    }

    Ok(field_validators)
}

fn validate_field(path: TokenStream, argument: FieldValidateArgument) -> TokenStream {
    use FieldValidateArgument::*;
    match argument {
        Email(_) => {
            quote! { .merge(::not_so_fast::validators::email::validate_email(#path)) }
        }
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

fn make_tuple<T: ToTokens>(elements: &[T]) -> TokenStream {
    match elements.len() {
        1 => quote! { (#(#elements),*,) },
        _ => quote! { (#(#elements),*) },
    }
}
