use parse::*;
use proc_macro2::TokenStream;
use quote::quote;
use syn::*;

mod parse;

#[proc_macro_derive(Validate, attributes(validate))]
pub fn derive_validate_args(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let type_: DeriveInput = syn::parse(input).expect("Input should be valid struct or enum");
    let type_name = type_.ident;
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
    match type_.data {
        Data::Enum(data_enum) => {
            if data_enum.variants.is_empty() {
                quote! {
                    impl ::not_so_fast::ValidateArgs for #type_name {
                        type Args = (#(#arg_types),*);

                        fn validate_args(&self, args: Self::Args) -> ::not_so_fast::ValidationErrors {
                            // TODO apply custom validator on self
                            ::not_so_fast::ValidationErrors::ok()
                        }
                    }
                }.into()
            } else {
                let branches = data_enum.variants.iter().map(|variant| {
                    let variant_name = variant.ident.clone();
                    if variant.fields.is_empty() {
                        return quote! { #type_name::#variant_name => ::not_so_fast::ValidationErrors::ok() };
                    }
                    todo!()
                });
                quote! {
                    impl ::not_so_fast::ValidateArgs for #type_name {
                        type Args = (#(#arg_types),*);

                        fn validate_args(&self, args: Self::Args) -> ::not_so_fast::ValidateErrors {
                            match self {
                                #(#branches),*
                            }
                        }
                    }
                }
                .into()
            }
        }
        Data::Struct(data_struct) => {
            let mut errors = Vec::new();
            for validator in type_custom_validators {
                let function = validator.function;
                let args = validator.args;
                errors.push(quote! { .merge(#function(&self, #(#args),*)) });
            }

            for (i, field) in data_struct.fields.iter().enumerate() {
                let mut field_errors = Vec::new();

                for attr in &field.attrs {
                    if attr.path.get_ident().map_or(false, |i| i == "validate") {
                        let arguments = match attr.parse_args::<FieldValidateArguments>() {
                            Ok(arguments) => arguments.arguments,
                            Err(e) => return e.to_compile_error().into(),
                        };
                        for argument in arguments {
                            if let Some(ident) = &field.ident {
                                field_errors
                                    .push(validate_field(quote! { &self.#ident }, &argument));
                            } else {
                                field_errors.push(validate_field(quote! { &self.#i }, &argument));
                            }
                        }
                    }
                }
                if !field_errors.is_empty() {
                    if let Some(ident) = &field.ident {
                        let field_name = ident.to_string();
                        errors.push(quote!(
                            .and_field(#field_name, ValidationErrors::ok()#(#field_errors),*)
                        ));
                    } else {
                        errors.push(quote!(
                            .and_item(#i, ValidationErrors::ok()#(#field_errors),*)
                        ));
                    }
                }
            }
            quote! {
                impl ::not_so_fast::ValidateArgs for #type_name {
                    type Args = (#(#arg_types),*);

                    fn validate_args(&self, args: Self::Args) -> ::not_so_fast::ValidationErrors {
                        let (#(#arg_names),*) = args;
                        ::not_so_fast::ValidationErrors::ok()
                            #(#errors)*
                    }
                }
            }
            .into()
        }
        _ => panic!("Only structs and enums supported"),
    }
}

fn validate_field(path: TokenStream, argument: &FieldValidateArgument) -> TokenStream {
    match argument {
        FieldValidateArgument::Email { .. } => {
            quote! { .merge(::not_so_fast::validators::email::validate_email(&#path)) }
        }
        _ => todo!("implement other field validators"),
    }
}
