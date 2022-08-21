use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream, Parser, Result as ParseResult};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::*;

struct ValidateArgument {
    ident: Ident,
    value: Option<TokenStream>,
}

impl Parse for ValidateArgument {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let ident = input.parse::<Ident>()?;
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            let value = input.parse::<TokenStream>()?;
            Ok(ValidateArgument {
                ident,
                value: Some(value),
            })
        } else {
            Ok(ValidateArgument { ident, value: None })
        }
    }
}

#[proc_macro_derive(Validate, attributes(validate))]
pub fn derive_validate_args(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let type_: DeriveInput = syn::parse(input).unwrap();
    let type_name = type_.ident;
    for attr in &type_.attrs {
        if attr.path.get_ident().map_or(false, |i| i == "validate") {
            let arguments = Punctuated::<ValidateArgument, Token![,]>::parse_terminated
                .parse2(attr.tokens.clone())
                .unwrap();
        }
    }
    match type_.data {
        Data::Enum(data_enum) => {
            if data_enum.variants.is_empty() {
                quote! {
                    impl ::not_so_fast::ValidateArgs for #type_name {
                        type Args = ();

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
                        type Args = ();

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
        Data::Struct(data_struct) => quote! {
            impl ::not_so_fast::ValidateArgs for #type_name {
                type Args = ();

                fn validate_args(&self, args: Self::Args) -> ::not_so_fast::ValidationErrors {
                    ::not_so_fast::ValidationErrors::ok()
                }
            }
        }
        .into(),
        _ => panic!("Only structs and enums supported"),
    }
}
