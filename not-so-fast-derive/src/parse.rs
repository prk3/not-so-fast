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
                r#"Unknown argument. Expected "args" or "custom""#,
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
            let input_span = input.span();
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
                    CustomArgument::Function(ident, _) => {
                        return Err(syn::Error::new_spanned(
                            ident,
                            "\"function\" already defined",
                        ))
                    }
                    CustomArgument::Args(ident, a) if args.is_none() => {
                        args = Some((ident, a));
                    }
                    CustomArgument::Args(ident, _) => {
                        return Err(syn::Error::new_spanned(ident, "\"args\" already defined"))
                    }
                }
            }

            match function {
                Some((ident, path)) => {
                    let (args_ident, args) =
                        args.map_or((None, Vec::new()), |(_, args)| (None, args));
                    Ok(Self {
                        function_ident: Some(ident),
                        function: path,
                        args_ident,
                        args,
                    })
                }
                None => Err(syn::Error::new(input_span, "\"function\" not defined")),
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

/// Arguments to field-level validate attribute.
///
/// ```text
/// #[derive(Validator)]
/// struct X {
///     #[validate(custom = myfunc, char_length(max = 100))]
///                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
///     name: String,
/// }
/// ```
///
/// Examples:
/// - `custom = myfunc, length(min=20, max=100)`
/// - `range(min=15)`
#[derive(Debug)]
pub struct FieldValidateArguments {
    pub arguments: Vec<FieldValidateArgument>,
}

impl FieldValidateArguments {
    pub fn empty() -> Self {
        Self {
            arguments: vec![FieldValidateArgument::Nested(
                None,
                NestedArguments { args: vec![] },
            )],
        }
    }
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

// Same as FieldValidateArguments, but optionally wrapped with parentheses.
struct OptParenFieldValidateArguments(FieldValidateArguments);

impl Parse for OptParenFieldValidateArguments {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(token::Paren) {
            let content;
            let _ = parenthesized!(content in input);
            Ok(Self(content.parse()?))
        } else {
            Ok(Self(FieldValidateArguments::empty()))
        }
    }
}

/// Argument to field-level validate attribute.
///
/// Examples:
/// - `custom = myfunc`
/// - `length(min=20, max=100)`
#[derive(Debug)]
pub enum FieldValidateArgument {
    Some(Ident, Box<FieldValidateArguments>),
    Items(Ident, Box<FieldValidateArguments>),
    Fields(Ident, Box<FieldValidateArguments>),
    Nested(Option<Ident>, NestedArguments),
    Custom(Ident, CustomArguments),
    Length(Ident, LengthArguments),
    CharLength(Ident, LengthArguments),
    Range(Ident, RangeArguments),
}

impl Parse for FieldValidateArgument {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse::<Ident>()?;
        match ident.to_string().as_str() {
            "some" => Ok(Self::Some(
                ident,
                Box::new(OptParenFieldValidateArguments::parse(input)?.0),
            )),
            "items" => Ok(Self::Items(
                ident,
                Box::new(OptParenFieldValidateArguments::parse(input)?.0),
            )),
            "fields" => Ok(Self::Fields(
                ident,
                Box::new(OptParenFieldValidateArguments::parse(input)?.0),
            )),
            "nested" => Ok(Self::Nested(Some(ident), input.parse()?)),
            "custom" => Ok(Self::Custom(ident, input.parse()?)),
            "length" => Ok(Self::Length(ident, input.parse()?)),
            "char_length" => Ok(Self::CharLength(ident, input.parse()?)),
            "range" => Ok(Self::Range(ident, input.parse()?)),
            _ => Err(syn::Error::new_spanned(
                ident,
                r#"Unknown argument. Expected "some", "items", "fields", "nested", "custom", "length", "char_length" or "range""#,
            )),
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

/// - `(min = 10)`
/// - `(max = 90)`
/// - `(min = 10, max = 90)`
/// - `(equals = 20)`
/// - `(min = path::to::VAR_OR_CONST)`
#[derive(Debug)]
pub struct LengthArguments {
    pub min: Option<LengthArgument>,
    pub max: Option<LengthArgument>,
    pub equal: Option<LengthArgument>,
}

impl Parse for LengthArguments {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut min = None;
        let mut max = None;
        let mut equal = None;

        let content;
        let _ = parenthesized!(content in input);
        let content_span_start = content.span();
        let args = Punctuated::<LengthArgument, Token![,]>::parse_terminated(&content)?;

        for arg in args {
            if arg.ident == "min" {
                if min.is_none() {
                    min = Some(arg);
                } else {
                    return Err(syn::Error::new(arg.ident.span(), "min already declared"));
                }
            } else if arg.ident == "max" {
                if max.is_none() {
                    max = Some(arg);
                } else {
                    return Err(syn::Error::new(arg.ident.span(), "max already declared"));
                }
            } else if arg.ident == "equal" {
                if equal.is_none() {
                    equal = Some(arg);
                } else {
                    return Err(syn::Error::new(arg.ident.span(), "equal already declared"));
                }
            } else {
                return Err(syn::Error::new(arg.ident.span(), "unknown length argument"));
            }
        }

        let min_or_max = min.is_some() || max.is_some();

        if min_or_max && equal.is_some() {
            return Err(syn::Error::new(
                content_span_start,
                "invalid argument combination: specify either min/max or equal",
            ));
        }
        if !min_or_max && equal.is_none() {
            return Err(syn::Error::new(
                content_span_start,
                "specify min, max, or equal",
            ));
        }

        Ok(Self { min, max, equal })
    }
}

/// - `min = 20`
/// - `max = path::to::VAR_OR_CONST`
#[derive(Debug)]
pub struct LengthArgument {
    pub ident: Ident,
    pub value: LengthArgumentValue,
}

impl Parse for LengthArgument {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        let _: Token![=] = input.parse()?;
        let value: LengthArgumentValue = input.parse()?;
        Ok(Self { ident, value })
    }
}

/// - `20`
/// - `path::to::VAR_OR_CONST`
#[derive(Debug)]
pub enum LengthArgumentValue {
    LitInt(LitInt),
    Path(Path),
}

impl Parse for LengthArgumentValue {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(LitInt) {
            return Ok(Self::LitInt(input.parse()?));
        }
        if let Ok(path) = input.parse::<Path>() {
            return Ok(Self::Path(path));
        }
        Err(syn::Error::new(
            input.span(),
            "Expected integer literal or a path to an integer",
        ))
    }
}

impl ToTokens for LengthArgumentValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::LitInt(lit) => lit.to_tokens(tokens),
            Self::Path(path) => path.to_tokens(tokens),
        }
    }
}

/// - (min = 10)
/// - (max = 90)
/// - (min = 10, max = 90)
/// - (min = path::to::VAR_OR_CONST)
#[derive(Debug)]
pub struct RangeArguments {
    pub min: Option<RangeArgument>,
    pub max: Option<RangeArgument>,
}

impl Parse for RangeArguments {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut min = None;
        let mut max = None;

        let content;
        let _ = parenthesized!(content in input);
        let content_span_start = content.span();
        let args = Punctuated::<RangeArgument, Token![,]>::parse_terminated(&content)?;

        for arg in args {
            if arg.ident == "min" {
                if min.is_none() {
                    min = Some(arg);
                } else {
                    return Err(syn::Error::new(arg.ident.span(), "min already declared"));
                }
            } else if arg.ident == "max" {
                if max.is_none() {
                    max = Some(arg);
                } else {
                    return Err(syn::Error::new(arg.ident.span(), "max already declared"));
                }
            } else {
                return Err(syn::Error::new(arg.ident.span(), "unknown range argument"));
            }
        }

        if min.is_none() && max.is_none() {
            return Err(syn::Error::new(content_span_start, "specify min or max"));
        }

        Ok(Self { min, max })
    }
}

/// - `min = 20`
/// - `min = 20.0`
/// - `max = path::to::VAR_OR_CONST`
#[derive(Debug)]
pub struct RangeArgument {
    pub ident: Ident,
    pub value: RangeArgumentValue,
}

impl Parse for RangeArgument {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        let _: Token![=] = input.parse()?;
        let value: RangeArgumentValue = input.parse()?;
        Ok(Self { ident, value })
    }
}

/// - `20`
/// - `20.0`
/// - `path::to::VAR_OR_CONST`
#[derive(Debug)]
pub enum RangeArgumentValue {
    LitInt(LitInt),
    LitFloat(LitFloat),
    Path(Path),
}

impl Parse for RangeArgumentValue {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(LitInt) {
            return Ok(Self::LitInt(input.parse()?));
        }
        if input.peek(LitFloat) {
            return Ok(Self::LitFloat(input.parse()?));
        }
        if let Ok(path) = input.parse::<Path>() {
            return Ok(Self::Path(path));
        }
        Err(syn::Error::new(
            input.span(),
            "Expected integer literal, float literal, or a path to an integer or float",
        ))
    }
}

impl ToTokens for RangeArgumentValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::LitInt(lit) => lit.to_tokens(tokens),
            Self::LitFloat(lit) => lit.to_tokens(tokens),
            Self::Path(path) => path.to_tokens(tokens),
        }
    }
}
