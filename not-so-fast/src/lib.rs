//! module docs

use std::borrow::Cow;
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::fmt::Write;
use std::vec;

/// Describes what's invalid about some value. It contains a code, an optional
/// message, and a list of error parameters.
#[derive(Debug)]
pub struct Error {
    /// Tells what feature of the validated value is not ok, e.g. "length",
    /// "range", "invariant_xyz".
    code: Cow<'static, str>,
    /// Optional message explaining the error code, e.g. "Illegal array
    /// length".
    message: Option<Cow<'static, str>>,
    /// A list of params that provide further context about the error, e.g. for
    /// code "range": "min", "max", "value".
    params: BTreeMap<Cow<'static, str>, String>,
}

impl Error {
    /// Creates an error with the provided code. Message and params are
    /// initially empty.
    /// ```
    /// # use not_so_fast::*;
    /// let error = Error::with_code("length");
    /// ```
    pub fn with_code(code: impl Into<Cow<'static, str>>) -> Self {
        Self {
            code: code.into(),
            message: None,
            params: BTreeMap::new(),
        }
    }

    /// Adds a message to the error. If called multiple times, the last message
    /// will be preserved.
    /// ```
    /// # use not_so_fast::*;
    /// let error = Error::with_code("length").and_message("String too long");
    /// ```
    pub fn and_message(mut self, message: impl Into<Cow<'static, str>>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Adds a parameter to the error. If the same parameter (meaning keys are
    /// equal) is added multiple times, the last value will be preserved.
    /// ```
    /// # use not_so_fast::*;
    /// let error = Error::with_code("length").and_param("max", "100".into());
    /// ```
    pub fn and_param(mut self, key: impl Into<Cow<'static, str>>, value: String) -> Self {
        self.params.insert(key.into(), value);
        self
    }
}

/// `ValidationErrors` allows you to build errors that mirror the structure of
/// validated data. It holds errors related to a validated value (value
/// errors), errors of fields (if the value is an object) and errors of items
/// (if the value is a list). You can think of it as a parent node in an error
/// tree, with [Error] type representing leafs.
#[derive(Debug)]
pub struct ValidationErrors {
    /// Errors of the validated value.
    errors: Vec<Error>,
    /// Errors of fields of the validated object.
    fields: BTreeMap<Cow<'static, str>, ValidationErrors>,
    /// Errors of items of the validate list.
    items: BTreeMap<usize, ValidationErrors>,
}

impl ValidationErrors {
    /// Creates `ValidationErrors` with no value errors, no field errors and no
    /// item errors. You'll be able to add errors to the returned value later.
    /// ```
    /// # use not_so_fast::*;
    /// let errors = ValidationErrors::ok();
    /// assert!(errors.is_ok());
    /// assert_eq!("", errors.to_string());
    /// ```
    pub fn ok() -> Self {
        Self {
            errors: Default::default(),
            fields: Default::default(),
            items: Default::default(),
        }
    }

    /// Converts `ValidationErrors` into `Result<(), ValidationErrors>`. It's
    /// useful when you want to propagate errors with `?` operator or transform
    /// the error using `Result`'s methods.
    /// Returns `Ok(())` if `self` has no value errors, no field errors and no
    /// item errors. Otherwise, returns `Err(self)`.
    /// ```
    /// # use not_so_fast::*;
    /// let errors_ok = ValidationErrors::ok();
    /// assert!(matches!(errors_ok.result(), Ok(_)));
    ///
    /// let errors_bad = ValidationErrors::error(Error::with_code("abc"));
    /// assert!(matches!(errors_bad.result(), Err(_)));
    /// ```
    pub fn result(self) -> Result<(), Self> {
        if self.is_ok() {
            Ok(())
        } else {
            Err(self)
        }
    }

    /// Checks if `ValidationError` has no value errors, no field errors, and
    /// no item errors.
    /// ```
    /// # use not_so_fast::*;
    /// let errors_ok = ValidationErrors::ok();
    /// assert!(errors_ok.is_ok());
    ///
    /// let errors_bad = ValidationErrors::error(Error::with_code("abc"));
    /// assert!(!errors_bad.is_ok());
    /// ```
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty() && self.fields.is_empty() && self.items.is_empty()
    }

    /// Checks if `ValidationError` has at least one value error, field error, or
    /// item error.
    /// ```
    /// # use not_so_fast::*;
    /// let errors_bad = ValidationErrors::error(Error::with_code("abc"));
    /// assert!(errors_bad.is_err());
    ///
    /// let errors_ok = ValidationErrors::ok();
    /// assert!(!errors_ok.is_err());
    /// ```
    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }

    /// Recursively adds errors from `other` to `self`.
    /// ```
    /// # use not_so_fast::*;
    /// let errors_a = ValidationErrors::field("a", ValidationErrors::error(Error::with_code("123")));
    /// let errors_b = ValidationErrors::field("b", ValidationErrors::error(Error::with_code("456")));
    /// let errors_c = errors_a.merge(errors_b);
    /// assert!(errors_c.is_err());
    /// assert_eq!(".a: 123\n.b: 456", errors_c.to_string());
    /// ```
    pub fn merge(mut self, other: Self) -> Self {
        self.merge_in_place(other);
        self
    }

    /// Merges `other` info `self` in-place (through `&mut`).
    fn merge_in_place(&mut self, other: ValidationErrors) {
        self.errors.extend(other.errors.into_iter());
        for (key, value) in other.fields {
            match self.fields.entry(key) {
                Entry::Vacant(entry) => {
                    entry.insert(value);
                }
                Entry::Occupied(mut entry) => {
                    entry.get_mut().merge_in_place(value);
                }
            }
        }
        for (key, value) in other.items {
            match self.items.entry(key) {
                Entry::Vacant(entry) => {
                    entry.insert(value);
                }
                Entry::Occupied(mut entry) => {
                    entry.get_mut().merge_in_place(value);
                }
            }
        }
    }

    /// Constructs `ValidationError` with one value error.
    /// ```
    /// # use not_so_fast::*;
    /// let errors = ValidationErrors::error(Error::with_code("abc"));
    /// assert!(errors.is_err());
    /// assert_eq!(".: abc", errors.to_string());
    /// ```
    pub fn error(error: Error) -> Self {
        Self {
            errors: vec![error],
            fields: Default::default(),
            items: Default::default(),
        }
    }

    /// Adds one value error to `self`.
    /// ```
    /// # use not_so_fast::*;
    /// let errors = ValidationErrors::ok().and_error(Error::with_code("abc"));
    /// assert!(errors.is_err());
    /// assert_eq!(".: abc", errors.to_string());
    /// ```
    pub fn and_error(mut self, error: Error) -> Self {
        self.errors.push(error);
        self
    }

    /// Constructs `ValidationError` with the value error returned by function
    /// `f` if `condition` is `true`. Otherwise, returns
    /// `ValidationErrors::ok()`. Function `f` will be called at most once.
    /// ```
    /// # use not_so_fast::*;
    /// let value = 10;
    /// let errors = ValidationErrors::error_if(value >= 20, || Error::with_code("abc"));
    /// assert!(errors.is_ok());
    ///
    /// let errors = ValidationErrors::error_if(value >= 10, || Error::with_code("def"));
    /// assert!(errors.is_err());
    /// assert_eq!(".: def", errors.to_string());
    /// ```
    pub fn error_if(condition: bool, f: impl FnOnce() -> Error) -> Self {
        Self {
            errors: if condition {
                vec![f()]
            } else {
                Default::default()
            },
            fields: Default::default(),
            items: Default::default(),
        }
    }

    /// Adds value error returned by function `f` to `ValidationError` if
    /// `condition` is `true`. Otherwise, returns unchanged `self`. Function
    /// `f` will be called at most once.
    /// ```
    /// # use not_so_fast::*;
    /// let value = 10;
    /// let errors = ValidationErrors::ok().and_error_if(value >= 20, || Error::with_code("abc"));
    /// assert!(errors.is_ok());
    ///
    /// let errors = ValidationErrors::ok().and_error_if(value >= 10, || Error::with_code("def"));
    /// assert!(errors.is_err());
    /// assert_eq!(".: def", errors.to_string());
    /// ```
    pub fn and_error_if(mut self, condition: bool, f: impl FnOnce() -> Error) -> Self {
        if condition {
            self.errors.push(f());
        }
        self
    }

    /// Constructs `ValidationError` from the value error iterator.
    /// ```
    /// # use not_so_fast::*;
    /// let value = 9;
    ///
    /// // error if value is divisible by 3, 5, or 15
    /// let errors_iter = [3, 5, 15]
    ///     .into_iter()
    ///     .filter_map(|divisor| (value % divisor == 0).then(|| {
    ///         Error::with_code("divisible").and_param("by", divisor.to_string())
    ///     }));
    ///
    /// let errors = ValidationErrors::errors(errors_iter);
    /// assert!(errors.is_err());
    /// assert_eq!(".: divisible: by=3", errors.to_string());
    /// ```
    pub fn errors(errors: impl Iterator<Item = Error>) -> ValidationErrors {
        Self {
            errors: errors.collect(),
            fields: Default::default(),
            items: Default::default(),
        }
    }

    /// Adds value errors from `errors` iterator to `self`.
    /// ```
    /// # use not_so_fast::*;
    /// let value = 9;
    ///
    /// // error if value is divisible by 3, 5, or 15
    /// let errors_iter = [3, 5, 15]
    ///     .into_iter()
    ///     .filter_map(|divisor| (value % divisor == 0).then(|| {
    ///         Error::with_code("divisible").and_param("by", divisor.to_string())
    ///     }));
    ///
    /// let errors = ValidationErrors::ok().and_errors(errors_iter);
    /// assert!(errors.is_err());
    /// assert_eq!(".: divisible: by=3", errors.to_string());
    /// ```
    pub fn and_errors(mut self, errors: impl Iterator<Item = Error>) -> ValidationErrors {
        self.errors.extend(errors);
        self
    }

    /// Constructs `ValidationErrors` with errors of one field. If
    /// `validation_errors` is ok, the function also returns ok errors.
    /// ```
    /// # use not_so_fast::*;
    /// let errors = ValidationErrors::field("a", ValidationErrors::ok());
    /// assert!(errors.is_ok());
    ///
    /// let errors = ValidationErrors::field("a", ValidationErrors::error(Error::with_code("abc")));
    /// assert!(errors.is_err());
    /// assert_eq!(".a: abc", errors.to_string());
    /// ```
    pub fn field(name: impl Into<Cow<'static, str>>, validation_errors: ValidationErrors) -> Self {
        Self {
            errors: Default::default(),
            fields: if !validation_errors.is_ok() {
                let mut fields = BTreeMap::default();
                fields.insert(name.into(), validation_errors);
                fields
            } else {
                Default::default()
            },
            items: Default::default(),
        }
    }

    /// Adds errors of one field to self. If self already contains errors for
    /// that field, the errors will be merged. If `validation_errors` is ok,
    /// the function will return self unchanged.
    /// ```
    /// # use not_so_fast::*;
    /// let errors = ValidationErrors::ok().and_field("a", ValidationErrors::ok());
    /// assert!(errors.is_ok());
    ///
    /// let errors = ValidationErrors::ok().and_field("a", ValidationErrors::error(Error::with_code("abc")));
    /// assert!(errors.is_err());
    ///
    /// let errors = ValidationErrors::ok()
    ///     .and_field("a", ValidationErrors::error(Error::with_code("abc")))
    ///     .and_field("a", ValidationErrors::error(Error::with_code("def")))
    ///     .and_field("b", ValidationErrors::error(Error::with_code("ghi")));
    /// assert!(errors.is_err());
    /// assert_eq!(".a: abc\n.a: def\n.b: ghi", errors.to_string());
    /// ```
    pub fn and_field(
        mut self,
        name: impl Into<Cow<'static, str>>,
        validation_errors: ValidationErrors,
    ) -> Self {
        if !validation_errors.is_ok() {
            match self.fields.entry(name.into()) {
                Entry::Vacant(entry) => {
                    entry.insert(validation_errors);
                }
                Entry::Occupied(mut entry) => entry.get_mut().merge_in_place(validation_errors),
            }
        }
        self
    }

    /// Collects field errors from an iterator to (key, value) pairs and a
    /// function transforming key and value references into validation errors.
    /// ```
    /// # use not_so_fast::*;
    /// let map: std::collections::HashMap<String, u32> = [
    ///     ("one".into(), 1),
    ///     ("two".into(), 2),
    ///     ("three".into(), 3),
    /// ].into_iter().collect();
    /// let errors = ValidationErrors::fields(map.iter(), |_key, value| {
    ///     ValidationErrors::error_if(*value > 2, || Error::with_code("abc"))
    /// });
    /// assert!(errors.is_err());
    /// assert_eq!(".three: abc", errors.to_string());
    /// ```
    pub fn fields<'a, K: 'a, V: 'a>(
        iterator: impl Iterator<Item = (&'a K, &'a V)>,
        f: impl Fn(&'a K, &'a V) -> ValidationErrors,
    ) -> Self
    where
        &'a K: Into<Cow<'a, str>>,
    {
        iterator.fold(ValidationErrors::ok(), |acc, (key, value)| {
            let validation_errors = f(key, value);

            // Clone key only if value has errors.
            if !validation_errors.is_ok() {
                let key_owned = Cow::Owned(key.into().to_string());
                acc.and_field(key_owned, validation_errors)
            } else {
                acc
            }
        })
    }

    /// Adds field errors collected the same way as in
    /// [fields](ValidationErrors::fields) method to self.
    /// ```
    /// # use not_so_fast::*;
    /// let map: std::collections::HashMap<String, u32> = [
    ///     ("one".into(), 1),
    ///     ("two".into(), 2),
    ///     ("three".into(), 3),
    /// ].into_iter().collect();
    /// let errors = ValidationErrors::ok().and_fields(map.iter(), |_key, value| {
    ///     ValidationErrors::error_if(*value > 2, || Error::with_code("abc"))
    /// });
    /// assert!(errors.is_err());
    /// assert_eq!(".three: abc", errors.to_string());
    /// ```
    pub fn and_fields<'a, K: 'a, V: 'a>(
        self,
        iterator: impl Iterator<Item = (&'a K, &'a V)>,
        f: impl Fn(&'a K, &'a V) -> ValidationErrors,
    ) -> Self
    where
        &'a K: Into<Cow<'a, str>>,
    {
        self.merge(Self::fields(iterator, f))
    }

    /// Constructs `ValidationErrors` with errors of one item. If
    /// `validation_errors` is ok, the function also returns ok errors.
    /// ```
    /// # use not_so_fast::*;
    /// let errors = ValidationErrors::item(5, ValidationErrors::ok());
    /// assert!(errors.is_ok());
    ///
    /// let errors = ValidationErrors::item(5, ValidationErrors::error(Error::with_code("abc")));
    /// assert!(errors.is_err());
    /// assert_eq!(".[5]: abc", errors.to_string());
    /// ```
    pub fn item(index: usize, validation_errors: ValidationErrors) -> Self {
        Self {
            errors: Default::default(),
            fields: Default::default(),
            items: if !validation_errors.is_ok() {
                let mut items = BTreeMap::default();
                items.insert(index, validation_errors);
                items
            } else {
                Default::default()
            },
        }
    }

    /// Adds errors of one item to self. If self already contains errors for
    /// that item, the errors will be merged. If `validation_errors` is ok,
    /// the function will return self unchanged.
    /// ```
    /// # use not_so_fast::*;
    /// let errors = ValidationErrors::ok().and_item(5, ValidationErrors::ok());
    /// assert!(errors.is_ok());
    ///
    /// let errors = ValidationErrors::ok().and_item(5, ValidationErrors::error(Error::with_code("abc")));
    /// assert!(errors.is_err());
    ///
    /// let errors = ValidationErrors::ok()
    ///     .and_item(5, ValidationErrors::error(Error::with_code("abc")))
    ///     .and_item(5, ValidationErrors::error(Error::with_code("def")))
    ///     .and_item(8, ValidationErrors::error(Error::with_code("ghi")));
    /// assert!(errors.is_err());
    /// assert_eq!(".[5]: abc\n.[5]: def\n.[8]: ghi", errors.to_string());
    /// ```
    pub fn and_item(mut self, index: usize, validation_errors: ValidationErrors) -> Self {
        if !validation_errors.is_ok() {
            match self.items.entry(index) {
                Entry::Vacant(entry) => {
                    entry.insert(validation_errors);
                }
                Entry::Occupied(mut entry) => entry.get_mut().merge_in_place(validation_errors),
            }
        }
        self
    }

    /// Collects item errors from an iterator to (index, value) pairs and a
    /// function transforming index and value references into validation
    /// errors.
    /// ```
    /// # use not_so_fast::*;
    /// let list: Vec<u32> = vec![10, 20, 30];
    ///
    /// let errors = ValidationErrors::items(list.iter(), |_index, value| {
    ///     ValidationErrors::error_if(*value > 25, || Error::with_code("abc"))
    /// });
    /// assert!(errors.is_err());
    /// assert_eq!(".[2]: abc", errors.to_string());
    /// ```
    pub fn items<'a, T: 'a>(
        items: impl Iterator<Item = &'a T>,
        f: impl Fn(usize, &'a T) -> ValidationErrors,
    ) -> Self {
        items
            .enumerate()
            .fold(ValidationErrors::ok(), |acc, (index, item)| {
                acc.and_item(index, f(index, item))
            })
    }

    /// Returns validation errors with only the first error (or None).
    /// ```
    /// # use not_so_fast::*;
    /// let errors = ValidationErrors::ok()
    ///     .and_field("a", ValidationErrors::error(Error::with_code("1")))
    ///     .and_field("a", ValidationErrors::error(Error::with_code("2")))
    ///     .and_field("b", ValidationErrors::error(Error::with_code("3")));
    /// assert_eq!(".a: 1\n.a: 2\n.b: 3", errors.to_string());
    ///
    /// let first = errors.first();
    /// assert_eq!(".a: 1", first.to_string());
    /// ```
    pub fn first(mut self) -> Self {
        if !self.errors.is_empty() {
            Self {
                errors: vec![self.errors.remove(0)],
                fields: Default::default(),
                items: Default::default(),
            }
        } else if !self.fields.is_empty() {
            Self {
                errors: Default::default(),
                fields: self
                    .fields
                    .into_iter()
                    .map(|(key, errors)| (key, errors.first()))
                    .take(1)
                    .collect(),
                items: Default::default(),
            }
        } else if !self.items.is_empty() {
            Self {
                errors: Default::default(),
                fields: Default::default(),
                items: self
                    .items
                    .into_iter()
                    .map(|(index, errors)| (index, errors.first()))
                    .take(1)
                    .collect(),
            }
        } else {
            Self::ok()
        }
    }

    /// Adds item errors collected the same way as in
    /// [items](ValidationErrors::items) method to self.
    /// ```
    /// # use not_so_fast::*;
    /// let list = vec![10, 20, 30];
    ///
    /// let errors = ValidationErrors::ok().and_items(list.iter(), |_index, value| {
    ///     ValidationErrors::error_if(*value > 25, || Error::with_code("abc"))
    /// });
    /// assert!(errors.is_err());
    /// assert_eq!(".[2]: abc", errors.to_string());
    /// ```
    pub fn and_items<'a, T: 'a>(
        self,
        items: impl Iterator<Item = &'a T>,
        f: impl Fn(usize, &'a T) -> ValidationErrors,
    ) -> Self {
        self.merge(Self::items(items, f))
    }

    fn display_fmt<'s, 'p, 'e, 'f>(
        &'s self,
        path: &'p mut Vec<PathElement<'s>>,
        first_printed: &'p mut bool,
        f: &'f mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        for direct in self.errors.iter() {
            if *first_printed {
                f.write_char('\n')?;
                fmt_path_and_error(&direct, path.as_slice(), f)?;
            } else {
                fmt_path_and_error(&direct, path.as_slice(), f)?;
                *first_printed = true;
            }
        }
        for field in self.fields.iter() {
            path.push(PathElement::Name(field.0));
            field.1.display_fmt(path, first_printed, f)?;
            path.pop();
        }
        for item in self.items.iter() {
            path.push(PathElement::Index(*item.0));
            item.1.display_fmt(path, first_printed, f)?;
            path.pop();
        }
        Ok(())
    }

    #[cfg(feature = "serde")]
    fn serialize_elements<'s, S>(
        &'s self,
        path: &mut Vec<PathElement<'s>>,
        buffer: &mut String,
        seq_serializer: &mut S::SerializeSeq,
    ) -> Result<(), S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;

        for direct in self.errors.iter() {
            // TODO Figure out a way to serialize path and error without
            // creating temporary strings or using the buffer.
            buffer.clear();
            write!(buffer, "{}", Path(path.as_slice())).unwrap();
            let path_len = buffer.len();
            write!(buffer, "{}", ErrorDisplay(&direct)).unwrap();

            let path = &buffer[0..path_len];
            let error = &buffer[path_len..buffer.len()];
            seq_serializer.serialize_element(&(path, error))?;
        }
        for field in self.fields.iter() {
            path.push(PathElement::Name(field.0));
            field
                .1
                .serialize_elements::<S>(path, buffer, seq_serializer)?;
            path.pop();
        }
        for item in self.items.iter() {
            path.push(PathElement::Index(*item.0));
            item.1
                .serialize_elements::<S>(path, buffer, seq_serializer)?;
            path.pop();
        }
        Ok(())
    }
}

/// Trait describing types that can be validated without arguments. It is
/// implemented for all types that implement `ValidateArgs<Args=()>`.
pub trait Validate {
    fn validate(&self) -> ValidationErrors;
}

/// Trait describing types that can be validated with arguments.
pub trait ValidateArgs {
    type Args;
    fn validate_args(&self, args: Self::Args) -> ValidationErrors;
}

impl<T> Validate for T
where
    T: ValidateArgs<Args = ()>,
{
    fn validate(&self) -> ValidationErrors {
        self.validate_args(())
    }
}

enum PathElement<'a> {
    Name(&'a str),
    Index(usize),
}

fn fmt_path(path: &[PathElement], f: &mut std::fmt::Formatter) -> std::fmt::Result {
    if path.is_empty() {
        return f.write_char('.');
    }
    for (i, element) in path.iter().enumerate() {
        match element {
            PathElement::Name(_) => {
                f.write_char('.')?;
                fmt_path_element(element, f)?;
            }
            PathElement::Index(_) => {
                if i == 0 {
                    f.write_char('.')?;
                }
                fmt_path_element(element, f)?;
            }
        }
    }
    Ok(())
}

fn fmt_path_element(element: &PathElement, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match element {
        PathElement::Name(name) => {
            if !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                f.write_str(name)?;
            } else {
                f.write_char('"')?;
                for c in name.chars() {
                    if c == '"' {
                        f.write_str("\\\"")?;
                    } else {
                        f.write_char(c)?;
                    }
                }
                f.write_char('"')?;
            }
        }
        PathElement::Index(index) => {
            write!(f, "[{}]", index)?;
        }
    }
    Ok(())
}

fn fmt_error(error: &Error, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    f.write_str(error.code.as_ref())?;
    if let Some(message) = &error.message {
        f.write_str(": ")?;
        f.write_str(message.as_ref())?;
    }
    for (i, param) in error.params.iter().enumerate() {
        if i != 0 {
            f.write_str(", ")?;
        } else {
            f.write_str(": ")?;
        }
        f.write_str(param.0)?;
        f.write_str("=")?;
        f.write_str(param.1)?;
    }
    Ok(())
}

fn fmt_path_and_error(
    error: &Error,
    path: &[PathElement],
    f: &mut std::fmt::Formatter,
) -> std::fmt::Result {
    fmt_path(path, f)?;
    f.write_str(": ")?;
    fmt_error(error, f)
}

struct Path<'a, 'b>(&'a [PathElement<'b>]);

impl<'a, 'b> std::fmt::Display for Path<'a, 'b> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_path(self.0, f)
    }
}

struct ErrorDisplay<'a>(&'a Error);

impl<'a> std::fmt::Display for ErrorDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_error(self.0, f)
    }
}

impl std::fmt::Display for ValidationErrors {
    /// Prints validation errors, one per line with `jq`-like path and an error
    /// description.
    /// ```text
    /// .: invariant_x: property x is not greater than property y
    /// .abc[4]: length: illegal string length: min=10, max=20, value=34
    /// .def.ghi: test
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut path = Vec::new();
        self.display_fmt(&mut path, &mut false, f)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for ValidationErrors {
    /// Serializes validation errors as an array of error tuples, each
    /// containing `jq`-like path and error description, e.g.
    /// ```json
    /// [
    ///     [".", "invariant_x: property x is not greater than property y"],
    ///     [".abc[4]", "length: illegal string length: min=10, max=20, value=34"],
    ///     [".def.ghi", "test"]
    /// ]
    /// ```
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;

        let mut path = Vec::new();
        let mut buffer = String::new();
        let mut seq = serializer.serialize_seq(None)?;
        self.serialize_elements::<S>(&mut path, &mut buffer, &mut seq)?;
        seq.end()
    }
}
