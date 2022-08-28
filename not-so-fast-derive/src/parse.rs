use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Paren;
use syn::*;

/// Arguments to type-level validate macro.
/// Accepts zero or one `args` and zero or more `custom`.
///
/// ```text
/// #[derive(Validator)]
/// #[validate(args=(a: u64), custom = myfunc)]
///            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
/// struct X {
///     ...
/// }
/// ```
pub struct TypeValidateArguments {
    pub arguments: Vec<TypeValidateArgument>,
}

impl Parse for TypeValidateArguments {
    fn parse(input: ParseStream) -> Result<Self> {
        let arguments = Punctuated::<TypeValidateArgument, Token![,]>::parse_terminated(&input)?
            .into_iter()
            .try_fold(Vec::new(), |mut acc, argument| match argument {
                TypeValidateArgument::Args(ident, _)
                    if acc
                        .iter()
                        .any(|a| matches!(a, TypeValidateArgument::Args(_, _))) =>
                {
                    Err(syn::Error::new_spanned(ident, "\"args\" already defined"))
                }
                _ => {
                    acc.push(argument);
                    Ok(acc)
                }
            })?;
        Ok(Self { arguments })
    }
}

/// Argument to type-level validate attribute.
/// Examples:
/// - `args(a: u64, b: bool)`
/// - `custom = path::to::function`
/// - `custom(function = path::to::function)`
/// - `custom(function = path::to::function, args(100, true))`
#[derive(Debug)]
pub enum TypeValidateArgument {
    Args(Ident, ArgsArguments),
    Custom(Ident, CustomArguments),
}

impl Parse for TypeValidateArgument {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse::<Ident>()?;
        match ident.to_string().as_str() {
            "args" => {
                let args_arguments: ArgsArguments = input.parse()?;
                Ok(Self::Args(ident, args_arguments))
            }
            "custom" => {
                let custom_arguments: CustomArguments = input.parse()?;
                Ok(Self::Custom(ident, custom_arguments))
            }
            _ => Err(syn::Error::new_spanned(
                ident,
                "Unknown argument. Expected \"args\" or \"custom\"",
            )),
        }
    }
}

/// Args arguments, e.g.
/// - `(a: u64, b: bool, c: char)`
#[derive(Debug)]
pub struct ArgsArguments {
    pub arguments: Vec<ArgsArgument>,
}

impl Parse for ArgsArguments {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let _ = parenthesized!(content in input);
        Ok(ArgsArguments {
            arguments: Punctuated::<ArgsArgument, Token![,]>::parse_terminated(&content)?
                .into_iter()
                // Fold arguments into a Vec and check if all names are unique.
                .try_fold(Vec::<ArgsArgument>::new(), |mut acc, argument| {
                    if acc.iter().all(|a| a.name != argument.name) {
                        acc.push(argument);
                        Ok(acc)
                    } else {
                        Err(syn::Error::new_spanned(
                            &argument.name,
                            format!("Argument {:?} already declared", argument.name.to_string()),
                        ))
                    }
                })?,
        })
    }
}

/// Args argument, e.g.
/// - `a: u64`
#[derive(Debug)]
pub struct ArgsArgument {
    pub name: Ident,
    pub type_: Type,
}

impl Parse for ArgsArgument {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        let _: Token![:] = input.parse()?;
        let type_: Type = input.parse()?;
        Ok(Self { name, type_ })
    }
}

/// Parses custom validator arguments, e.g.
/// - `= validator::path`
/// - `(function = validator::path)`
/// - `(function = validator::path, args(a, b, c))`
#[derive(Debug)]
pub struct CustomArguments {
    pub function_ident: Option<Ident>,
    pub function: Path,
    pub args_ident: Option<Ident>,
    pub args: Vec<Arg>,
}

impl Parse for CustomArguments {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![=]) {
            let _: Token![=] = input.parse()?;
            let path: Path = input.parse()?;
            Ok(Self {
                function_ident: None,
                function: path,
                args_ident: None,
                args: Vec::new(),
            })
        } else {
            let content;
            let _ = parenthesized!(content in input);

            let mut function = None;
            let mut args = None;

            let arguments = Punctuated::<CustomArgument, Token![,]>::parse_terminated(&content)?;
            for argument in arguments {
                match argument {
                    CustomArgument::Function(ident, path) if function.is_none() => {
                        function = Some((ident, path));
                    }
                    CustomArgument::Function(ident, path) => {
                        return Err(syn::Error::new_spanned(
                            ident,
                            "\"function\" already defined",
                        ))
                    }
                    CustomArgument::Args(ident, a) if args.is_none() => {
                        args = Some((ident, a));
                    }
                    CustomArgument::Args(ident, a) => {
                        return Err(syn::Error::new_spanned(ident, "\"args\" already defined"))
                    }
                }
            }

            match function {
                Some((ident, path)) => {
                    let (args_ident, args) =
                        args.map_or((None, Vec::new()), |(ident, args)| (None, args));
                    Ok(Self {
                        function_ident: Some(ident),
                        function: path,
                        args_ident,
                        args,
                    })
                }
                None => {
                    // TODO fix
                    panic!("Validation function not defined");
                }
            }
        }
    }
}

/// Parses custom validator argument, e.g.
/// - `function = validator::path`
/// - `args(a, b, c)`
pub enum CustomArgument {
    Function(Ident, Path),
    Args(Ident, Vec<Arg>),
}

impl Parse for CustomArgument {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        if ident == "function" {
            let _: Token![=] = input.parse()?;
            let path: Path = input.parse()?;
            Ok(Self::Function(ident, path))
        } else if ident == "args" {
            let content;
            let _ = parenthesized!(content in input);
            let args = Punctuated::<Arg, Token![,]>::parse_terminated(&content)?;
            Ok(Self::Args(ident, args.into_iter().collect()))
        } else {
            Err(syn::Error::new_spanned(
                ident,
                "Illegal argument for custom argument: expected \"function\" or \"args\"",
            ))
        }
    }
}

/// - `204`
/// - `"hello"`
/// - `path::to::VAR_OR_CONST`
#[derive(Debug)]
pub enum Arg {
    LitBool(LitBool),
    LitByte(LitByte),
    LitByteStr(LitByteStr),
    LitChar(LitChar),
    LitFloat(LitFloat),
    LitInt(LitInt),
    LitStr(LitStr),
    Path(Path),
}

impl Parse for Arg {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        Ok(if lookahead.peek(LitBool) {
            Self::LitBool(input.parse()?)
        } else if lookahead.peek(LitByte) {
            Self::LitByte(input.parse()?)
        } else if lookahead.peek(LitByteStr) {
            Self::LitByteStr(input.parse()?)
        } else if lookahead.peek(LitChar) {
            Self::LitChar(input.parse()?)
        } else if lookahead.peek(LitFloat) {
            Self::LitFloat(input.parse()?)
        } else if lookahead.peek(LitInt) {
            Self::LitInt(input.parse()?)
        } else if lookahead.peek(LitStr) {
            Self::LitStr(input.parse()?)
        } else {
            Self::Path(input.parse()?)
        })
    }
}

impl ToTokens for Arg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::LitBool(v) => v.to_tokens(tokens),
            Self::LitByte(v) => v.to_tokens(tokens),
            Self::LitByteStr(v) => v.to_tokens(tokens),
            Self::LitChar(v) => v.to_tokens(tokens),
            Self::LitFloat(v) => v.to_tokens(tokens),
            Self::LitInt(v) => v.to_tokens(tokens),
            Self::LitStr(v) => v.to_tokens(tokens),
            Self::Path(v) => v.to_tokens(tokens),
        }
    }
}

/// - ``
/// - `(args(a, b, c))`
#[derive(Debug)]
pub struct NestedArguments {
    pub args: Vec<Arg>,
}

impl Parse for NestedArguments {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Paren) {
            let content;
            let _ = parenthesized!(content in input);
            let arguments = Punctuated::<NestedArgument, Token![,]>::parse_terminated(&content)?;
            let mut args = None;
            for argument in arguments {
                match argument {
                    NestedArgument::Args(ident, _) if args.is_some() => {
                        return Err(syn::Error::new_spanned(ident, "args already defined"));
                    }
                    NestedArgument::Args(_, a) => {
                        args = Some(a);
                    }
                }
            }
            Ok(Self {
                args: args.unwrap_or_else(Vec::new),
            })
        } else {
            Ok(Self { args: Vec::new() })
        }
    }
}

/// - `args(a, b, c)`
#[derive(Debug)]
pub enum NestedArgument {
    Args(Ident, Vec<Arg>),
}

impl Parse for NestedArgument {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        if ident == "args" {
            let content;
            let _ = parenthesized!(content in input);
            let args = Punctuated::<Arg, Token![,]>::parse_terminated(&content)?;
            Ok(Self::Args(ident, args.into_iter().collect()))
        } else {
            Err(syn::Error::new_spanned(
                ident,
                "Unsupported argument, expected \"args\"",
            ))
        }
    }
}

/// Arguments to field-level validate macro.
///
/// ```text
/// #[derive(Validator)]
/// struct X {
///     #[validate(length(max = 100))]
///                ^^^^^^^^^^^^^^^^^
///     name: String,
/// }
/// ```
///
/// Examples:
/// - `email, length(min=20, max=100)`
pub struct FieldValidateArguments {
    pub arguments: Vec<FieldValidateArgument>,
}

impl Parse for FieldValidateArguments {
    fn parse(input: ParseStream) -> Result<Self> {
        let arguments = Punctuated::<FieldValidateArgument, Token![,]>::parse_terminated(&input)?
            .into_iter()
            // TODO error on repeated illegal arguments
            .collect();
        Ok(Self { arguments })
    }
}

#[derive(Debug)]
pub enum FieldValidateArgument {
    Custom(Ident, CustomArguments),
    Nested(Ident, NestedArguments),
    Email(Ident),
    Url(Ident),
    Length {
        min: Option<LengthArgument>,
        max: Option<LengthArgument>,
        equal: Option<LengthArgument>,
    },
    ByteLength {
        min: Option<LengthArgument>,
        max: Option<LengthArgument>,
        equal: Option<LengthArgument>,
    },
    Range {
        min: Option<LengthArgument>,
        max: Option<LengthArgument>,
        equal: Option<LengthArgument>,
    },
    Contains {
        pattern: String,
    },
}

impl Parse for FieldValidateArgument {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse::<Ident>()?;
        match ident.to_string().as_str() {
            "email" => Ok(Self::Email(ident)),
            "custom" => Ok(Self::Custom(ident, input.parse()?)),
            "nested" => Ok(Self::Nested(ident, input.parse()?)),
            _ => Err(syn::Error::new_spanned(
                ident,
                "Unknown argument. Expected \"email\"",
            )),
        }
    }
}

/// - `20`
/// - `path::to::VAR_OR_CONST`
#[derive(Debug)]
enum LengthArgument {
    LitInt(LitInt),
    Path(Path),
}
