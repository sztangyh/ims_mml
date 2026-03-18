use crate::{ImsUserNum, MgwId};
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Errors returned by MML parsing and serialization routines.
pub enum MmlError {
    InvalidLine(String),
    InvalidParam(String),
    MissingField(String),
    ValueParse {
        field: String,
        value: String,
        reason: String,
    },
    UnknownBranchTag {
        tag: String,
        value: String,
    },
}

impl Display for MmlError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLine(msg) => write!(f, "invalid mml line: {msg}"),
            Self::InvalidParam(msg) => write!(f, "invalid mml parameter: {msg}"),
            Self::MissingField(field) => write!(f, "missing field: {field}"),
            Self::ValueParse {
                field,
                value,
                reason,
            } => write!(f, "failed to parse field {field} from {value}: {reason}"),
            Self::UnknownBranchTag { tag, value } => {
                write!(f, "unknown branch tag value: {tag}={value}")
            }
        }
    }
}

impl std::error::Error for MmlError {}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Owned MML parameter map with case-insensitive keys.
pub struct MmlParams {
    inner: BTreeMap<String, String>,
}

impl MmlParams {
    /// Creates a new empty value.
pub fn new() -> Self {
        Self::default()
    }

    /// Inserts or overwrites a parameter value by key.
pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.inner
            .insert(normalize_key(&key.into()), value.into().trim().to_string());
    }

    /// Gets a parameter value by key.
pub fn get(&self, key: &str) -> Option<&str> {
        self.inner
            .get(&normalize_key(key))
            .map(std::string::String::as_str)
    }

    /// Returns whether a key exists in the parameter map.
pub fn contains(&self, key: &str) -> bool {
        self.inner.contains_key(&normalize_key(key))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Borrowed MML parameter map that references input slices.
pub struct MmlParamsRef<'a> {
    inner: BTreeMap<String, &'a str>,
}

impl<'a> MmlParamsRef<'a> {
    /// Creates a new empty value.
pub fn new() -> Self {
        Self::default()
    }

    /// Inserts or overwrites a parameter value by key.
pub fn insert(&mut self, key: &str, value: &'a str) {
        self.inner.insert(normalize_key(key), value.trim());
    }

    /// Gets a parameter value by key.
pub fn get(&self, key: &str) -> Option<&'a str> {
        self.inner.get(&normalize_key(key)).copied()
    }

    /// Returns whether a key exists in the parameter map.
pub fn contains(&self, key: &str) -> bool {
        self.inner.contains_key(&normalize_key(key))
    }
}

/// Lookup abstraction shared by owned and borrowed parameter maps.
pub trait MmlParamLookup<'de> {
    /// Gets a parameter value by key.
fn get(self, key: &str) -> Option<&'de str>;
    /// Returns whether a key exists in the parameter map.
fn contains(self, key: &str) -> bool;
}

impl<'de, 'p: 'de> MmlParamLookup<'de> for &'p MmlParams {
    fn get(self, key: &str) -> Option<&'de str> {
        MmlParams::get(self, key)
    }

    fn contains(self, key: &str) -> bool {
        MmlParams::contains(self, key)
    }
}

impl<'de, 'p> MmlParamLookup<'de> for &'p MmlParamsRef<'de> {
    fn get(self, key: &str) -> Option<&'de str> {
        MmlParamsRef::get(self, key)
    }

    fn contains(self, key: &str) -> bool {
        MmlParamsRef::contains(self, key)
    }
}

/// Normalizes a parameter key for case-insensitive matching.
fn normalize_key(key: &str) -> String {
    key.trim().to_ascii_uppercase()
}

/// Validates parsed command header fields.
fn validate_cmd_header(
    op: &str,
    object: &str,
    expected_op: &str,
    expected_object: &str,
) -> Result<(), MmlError> {
    if !expected_op.is_empty() && !op.eq_ignore_ascii_case(expected_op) {
        return Err(MmlError::InvalidLine(format!(
            "expected op {}, got {op}",
            expected_op
        )));
    }
    if !expected_object.is_empty() && !object.eq_ignore_ascii_case(expected_object) {
        return Err(MmlError::InvalidLine(format!(
            "expected object {}, got {object}",
            expected_object
        )));
    }
    Ok(())
}

/// Parses a full MML command line into op, object, and owned parameters.
pub fn parse_mml_line(input: &str) -> Result<(String, String, MmlParams), MmlError> {
    let trimmed = input.trim();
    let (header, body_with_tail) = trimmed
        .split_once(':')
        .ok_or_else(|| MmlError::InvalidLine("missing ':' separator".into()))?;
    let body = body_with_tail
        .trim()
        .strip_suffix(';')
        .ok_or_else(|| MmlError::InvalidLine("missing ';' tail".into()))?;

    let mut hs = header.split_whitespace();
    let op = hs
        .next()
        .ok_or_else(|| MmlError::InvalidLine("missing operation type".into()))?
        .to_string();
    let object = hs
        .next()
        .ok_or_else(|| MmlError::InvalidLine("missing operation object".into()))?
        .to_string();

    if hs.next().is_some() {
        return Err(MmlError::InvalidLine(
            "header must be '<OP> <OBJECT>'".into(),
        ));
    }

    let params = parse_mml_params(body)?;
    Ok((op, object, params))
}

/// Parses a full MML command line into op, object, and borrowed parameters.
pub fn parse_mml_line_ref<'a>(input: &'a str) -> Result<(&'a str, &'a str, MmlParamsRef<'a>), MmlError> {
    let trimmed = input.trim();
    let (header, body_with_tail) = trimmed
        .split_once(':')
        .ok_or_else(|| MmlError::InvalidLine("missing ':' separator".into()))?;
    let body = body_with_tail
        .trim()
        .strip_suffix(';')
        .ok_or_else(|| MmlError::InvalidLine("missing ';' tail".into()))?;

    let mut hs = header.split_whitespace();
    let op = hs
        .next()
        .ok_or_else(|| MmlError::InvalidLine("missing operation type".into()))?;
    let object = hs
        .next()
        .ok_or_else(|| MmlError::InvalidLine("missing operation object".into()))?;

    if hs.next().is_some() {
        return Err(MmlError::InvalidLine(
            "header must be '<OP> <OBJECT>'".into(),
        ));
    }

    let params = parse_mml_params_ref(body)?;
    Ok((op, object, params))
}

/// Parses `K=V,...` into an owned parameter map.
pub fn parse_mml_params(body: &str) -> Result<MmlParams, MmlError> {
    let mut params = MmlParams::new();
    for (k, v) in split_key_values(body)? {
        params.insert(k, v);
    }
    Ok(params)
}

/// Parse `K=V,...` into a borrowed parameter map.
pub fn parse_mml_params_ref<'a>(body: &'a str) -> Result<MmlParamsRef<'a>, MmlError> {
    let mut params = MmlParamsRef::new();
    for (k, v) in split_key_values_ref(body)? {
        params.insert(k, v);
    }
    Ok(params)
}

/// Deserialize a target type from an owned parameter map.
pub fn deserialize_params<T>(params: &MmlParams) -> Result<T, MmlError>
where
    for<'de> T: MmlDeserialize<'de>,
{
    <T as MmlDeserialize<'_>>::from_mml_params(params)
}

/// Deserialize a target type from a borrowed parameter map.
pub fn deserialize_params_ref<'de, T>(params: &'de MmlParamsRef<'de>) -> Result<T, MmlError>
where
    T: MmlDeserialize<'de>,
{
    <T as MmlDeserialize<'de>>::from_mml_params(params)
}

/// Compose a full MML line from operation, object, and parameters.
pub fn compose_mml_line(op: &str, object: &str, pairs: &[(String, String)]) -> String {
    let body = pairs
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join(",");
    format!("{} {}:{};", op.trim(), object.trim(), body)
}

/// Splits a parameter body into owned key-value pairs.
fn split_key_values(body: &str) -> Result<Vec<(String, String)>, MmlError> {
    let mut out = Vec::new();
    let mut item = String::new();
    let mut in_quotes = false;
    let mut escaped = false;

    let push_item = |item: &mut String, out: &mut Vec<(String, String)>| -> Result<(), MmlError> {
        let trimmed = item.trim();
        if trimmed.is_empty() {
            item.clear();
            return Ok(());
        }
        let (k, v) = trimmed
            .split_once('=')
            .ok_or_else(|| MmlError::InvalidParam(format!("missing '=': {trimmed}")))?;
        let key = k.trim();
        if key.is_empty() {
            return Err(MmlError::InvalidParam(format!("empty key: {trimmed}")));
        }
        out.push((key.to_string(), v.trim().to_string()));
        item.clear();
        Ok(())
    };

    for ch in body.chars() {
        if ch == '"' && !escaped {
            in_quotes = !in_quotes;
        }
        if ch == ',' && !in_quotes {
            push_item(&mut item, &mut out)?;
            escaped = false;
            continue;
        }
        escaped = ch == '\\' && !escaped;
        item.push(ch);
    }

    if in_quotes {
        return Err(MmlError::InvalidParam("unclosed quote in body".into()));
    }

    push_item(&mut item, &mut out)?;
    Ok(out)
}

/// Splits a parameter body into borrowed key-value pairs.
fn split_key_values_ref<'a>(body: &'a str) -> Result<Vec<(&'a str, &'a str)>, MmlError> {
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut in_quotes = false;
    let mut escaped = false;

    for (idx, ch) in body.char_indices() {
        if ch == '"' && !escaped {
            in_quotes = !in_quotes;
        }
        if ch == ',' && !in_quotes {
            push_item_ref(&body[start..idx], &mut out)?;
            start = idx + 1;
            escaped = false;
            continue;
        }
        escaped = ch == '\\' && !escaped;
    }

    if in_quotes {
        return Err(MmlError::InvalidParam("unclosed quote in body".into()));
    }

    push_item_ref(&body[start..], &mut out)?;
    Ok(out)
}

fn push_item_ref<'a>(item: &'a str, out: &mut Vec<(&'a str, &'a str)>) -> Result<(), MmlError> {
    let trimmed = item.trim();
    if trimmed.is_empty() {
        return Ok(());
    }
    let (k, v) = trimmed
        .split_once('=')
        .ok_or_else(|| MmlError::InvalidParam(format!("missing '=': {trimmed}")))?;
    let key = k.trim();
    if key.is_empty() {
        return Err(MmlError::InvalidParam(format!("empty key: {trimmed}")));
    }
    out.push((key, v.trim()));
    Ok(())
}

/// Parses a text parameter and returns an owned string value.
pub fn parse_text_value(raw: &str) -> Result<String, MmlError> {
    let trimmed = raw.trim();
    if trimmed.starts_with('"') {
        if !trimmed.ends_with('"') || trimmed.len() < 2 {
            return Err(MmlError::InvalidParam(format!(
                "quoted value not closed: {trimmed}"
            )));
        }
        let inner = &trimmed[1..trimmed.len() - 1];
        return Ok(unescape_text(inner));
    }
    Ok(trimmed.to_string())
}

/// Parses a text parameter and returns a borrowed string slice.
pub fn parse_text_slice<'a>(raw: &'a str) -> Result<&'a str, MmlError> {
    let trimmed = raw.trim();
    if trimmed.starts_with('"') {
        if !trimmed.ends_with('"') || trimmed.len() < 2 {
            return Err(MmlError::InvalidParam(format!(
                "quoted value not closed: {trimmed}"
            )));
        }
        return Ok(&trimmed[1..trimmed.len() - 1]);
    }
    Ok(trimmed)
}

/// Parses a plain token as an owned string.
pub fn parse_plain_token(raw: &str) -> Result<String, MmlError> {
    parse_text_value(raw).map(|s| s.trim().to_string())
}

/// Parses a plain token as a borrowed string slice.
pub fn parse_plain_token_ref<'a>(raw: &'a str) -> Result<&'a str, MmlError> {
    Ok(parse_text_slice(raw)?.trim())
}

/// Encodes plain text into an MML string literal.
pub fn encode_text_value(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len() + 2);
    escaped.push('"');
    for ch in text.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }
    escaped.push('"');
    escaped
}

/// Unescapes backslash escape sequences inside quoted text.
fn unescape_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut escaped = false;
    for ch in text.chars() {
        if escaped {
            match ch {
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                'n' => out.push('\n'),
                'r' => out.push('\r'),
                't' => out.push('\t'),
                _ => {
                    out.push('\\');
                    out.push(ch);
                }
            }
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        out.push(ch);
    }
    if escaped {
        out.push('\\');
    }
    out
}

/// Bidirectional conversion between a Rust type and one MML value.
pub trait MmlValue: Sized {
    /// Parses a typed value from one raw MML value token.
fn from_mml_value(raw: &str) -> Result<Self, MmlError>;
    /// Encodes a typed value as one MML value token.
fn to_mml_value(&self) -> String;
}

/// Borrowing variant of `MmlValue` for zero-copy parsing.
pub trait MmlValueBorrowed<'de>: Sized {
    /// Parses a typed value by borrowing from the source input.
fn from_mml_value_borrowed(raw: &'de str) -> Result<Self, MmlError>;
}

/// Encodes a field value into an MML parameter literal.
pub trait MmlValueEncode {
    /// Encodes the value into an MML token string.
fn encode_mml_value(&self) -> String;
}

impl<'de, T> MmlValueBorrowed<'de> for T
where
    T: MmlValue,
{
    fn from_mml_value_borrowed(raw: &'de str) -> Result<Self, MmlError> {
        T::from_mml_value(raw)
    }
}

impl<T> MmlValueEncode for T
where
    T: MmlValue,
{
    fn encode_mml_value(&self) -> String {
        self.to_mml_value()
    }
}

impl<'de, 'a> MmlValueBorrowed<'de> for &'a str
where
    'de: 'a,
{
    fn from_mml_value_borrowed(raw: &'de str) -> Result<Self, MmlError> {
        parse_text_slice(raw)
    }
}

impl MmlValueEncode for &str {
    fn encode_mml_value(&self) -> String {
        encode_text_value(self)
    }
}

/// Field-level decode/encode behavior used by derive macros.
pub trait MmlField<'de>: Sized {
    /// Returns whether the field is present in the parameter map.
fn has_field<P>(field_name: &str, params: P) -> bool
    where
        P: MmlParamLookup<'de> + Copy,
    {
        params.contains(field_name)
    }

    /// Parses one field from the parameter map.
fn from_mml_field<P>(field_name: &str, params: P) -> Result<Self, MmlError>
    where
        P: MmlParamLookup<'de> + Copy;

    /// Appends one serialized field into the output list.
fn append_mml_field(
        &self,
        field_name: &str,
        out: &mut Vec<(String, String)>,
    ) -> Result<(), MmlError>;
}

impl<'de, T> MmlField<'de> for T
where
    T: MmlValueBorrowed<'de> + MmlValueEncode,
{
    fn from_mml_field<P>(field_name: &str, params: P) -> Result<Self, MmlError>
    where
        P: MmlParamLookup<'de> + Copy,
    {
        let raw = params
            .get(field_name)
            .ok_or_else(|| MmlError::MissingField(field_name.to_string()))?;
        T::from_mml_value_borrowed(raw).map_err(|e| MmlError::ValueParse {
            field: field_name.to_string(),
            value: raw.to_string(),
            reason: e.to_string(),
        })
    }

    fn append_mml_field(
        &self,
        field_name: &str,
        out: &mut Vec<(String, String)>,
    ) -> Result<(), MmlError> {
        out.push((field_name.to_string(), self.encode_mml_value()));
        Ok(())
    }
}

/// Branch decode/encode behavior for tagged parameter groups.
pub trait MmlBranch<'de>: Sized {
    /// Branch tag key override; use field name when `None`.
const TAG: Option<&'static str>;

    /// Parses a tagged branch from the parameter map.
fn from_mml_branch<P>(tag_key: &str, params: P) -> Result<Self, MmlError>
    where
        P: MmlParamLookup<'de> + Copy;

    /// Appends a tagged branch into serialized parameters.
fn append_mml_branch(
        &self,
        tag_key: &str,
        out: &mut Vec<(String, String)>,
    ) -> Result<(), MmlError>;
}

/// Deserializes a Rust type from MML parameters.
pub trait MmlDeserialize<'de>: Sized {
    /// Builds a value from MML parameters.
fn from_mml_params<P>(params: P) -> Result<Self, MmlError>
    where
        P: MmlParamLookup<'de> + Copy;
}

/// Serializes a Rust type into MML key-value parameters.
pub trait MmlSerialize {
    /// Serializes the value into MML parameters.
fn to_mml_params(&self) -> Result<Vec<(String, String)>, MmlError>;
}

/// Command-level API that validates op/object headers.
pub trait MmlCommand: Sized + MmlSerialize {
    /// Expected MML operation token, such as `ADD`.
const MML_OP: &'static str;
    /// Expected MML object token, such as `ASBR`.
const MML_OBJECT: &'static str;

    /// Parses and validates a full MML command line for this command type.
fn from_mml_line(input: &str) -> Result<Self, MmlError>
    where
        for<'de> Self: MmlDeserialize<'de>,
    {
        let (op, object, params) = parse_mml_line(input)?;
        validate_cmd_header(&op, &object, Self::MML_OP, Self::MML_OBJECT)?;
        <Self as MmlDeserialize<'_>>::from_mml_params(&params)
    }

    /// Parses a full command line with borrowed fields when possible.
fn from_mml_line_borrowed<'de>(input: &'de str) -> Result<Self, MmlError>
    where
        Self: MmlDeserialize<'de>,
    {
        let (op, object, params) = parse_mml_line_ref(input)?;
        validate_cmd_header(op, object, Self::MML_OP, Self::MML_OBJECT)?;
        <Self as MmlDeserialize<'de>>::from_mml_params(&params)
    }

    /// Serializes a command into a full MML line.
fn to_mml_line(&self) -> Result<String, MmlError> {
        let params = self.to_mml_params()?;
        Ok(compose_mml_line(Self::MML_OP, Self::MML_OBJECT, &params))
    }
}

macro_rules! impl_number_value {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl MmlValue for $ty {
                fn from_mml_value(raw: &str) -> Result<Self, MmlError> {
                    let v = parse_plain_token(raw)?;
                    v.parse::<$ty>().map_err(|e| MmlError::InvalidParam(e.to_string()))
                }

                fn to_mml_value(&self) -> String {
                    self.to_string()
                }
            }
        )+
    };
}

impl_number_value!(u8, u16, u32, u64, u128, usize);
impl_number_value!(i8, i16, i32, i64, i128, isize);
impl_number_value!(f32, f64);

impl MmlValue for bool {
    fn from_mml_value(raw: &str) -> Result<Self, MmlError> {
        let v = parse_plain_token(raw)?;
        match v.to_ascii_lowercase().as_str() {
            "true" | "1" | "yes" | "y" => Ok(true),
            "false" | "0" | "no" | "n" => Ok(false),
            _ => Err(MmlError::InvalidParam(format!("invalid bool: {v}"))),
        }
    }

    fn to_mml_value(&self) -> String {
        if *self {
            "TRUE".to_string()
        } else {
            "FALSE".to_string()
        }
    }
}

impl MmlValue for String {
    fn from_mml_value(raw: &str) -> Result<Self, MmlError> {
        parse_text_value(raw)
    }

    fn to_mml_value(&self) -> String {
        encode_text_value(self)
    }
}

impl MmlValue for char {
    fn from_mml_value(raw: &str) -> Result<Self, MmlError> {
        let v = parse_text_value(raw)?;
        let mut chars = v.chars();
        let ch = chars
            .next()
            .ok_or_else(|| MmlError::InvalidParam("char value cannot be empty".into()))?;
        if chars.next().is_some() {
            return Err(MmlError::InvalidParam(format!("invalid char: {v}")));
        }
        Ok(ch)
    }

    fn to_mml_value(&self) -> String {
        encode_text_value(&self.to_string())
    }
}

impl MmlValue for MgwId {
    fn from_mml_value(raw: &str) -> Result<Self, MmlError> {
        let v = parse_plain_token(raw)?;
        v.parse::<MgwId>()
            .map_err(|e| MmlError::InvalidParam(e.to_string()))
    }

    fn to_mml_value(&self) -> String {
        encode_text_value(&self.to_string())
    }
}

impl<const NUM_SIZE: usize> MmlValue for ImsUserNum<NUM_SIZE> {
    fn from_mml_value(raw: &str) -> Result<Self, MmlError> {
        let v = parse_text_value(raw)?;
        v.parse::<ImsUserNum<NUM_SIZE>>()
            .map_err(|e| MmlError::InvalidParam(e.to_string()))
    }

    fn to_mml_value(&self) -> String {
        encode_text_value(&self.to_string())
    }
}

#[macro_export]
/// Implements `MmlValue` via `FromStr` and `Display`.
macro_rules! impl_mml_value_via_fromstr_display {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl $crate::MmlValue for $ty {
                fn from_mml_value(raw: &str) -> Result<Self, $crate::MmlError> {
                    let v = $crate::parse_plain_token(raw)?;
                    v.parse::<$ty>().map_err(|e| {
                        $crate::MmlError::InvalidParam(
                            format!("invalid {}: {} ({})", stringify!($ty), v, e),
                        )
                    })
                }

                fn to_mml_value(&self) -> String {
                    self.to_string()
                }
            }
        )+
    };
}
