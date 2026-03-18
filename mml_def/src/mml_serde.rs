use crate::{ImsUserNum, MgwId};
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
/// MML 解析与序列化过程中的统一错误类型。
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
/// 拥有型 MML 参数集合，键名大小写不敏感（内部统一为大写）。
pub struct MmlParams {
    inner: BTreeMap<String, String>,
}

impl MmlParams {
    /// 创建空参数集合。
    pub fn new() -> Self {
        Self::default()
    }

    /// 插入参数项，键会规范化，值会自动去除首尾空白。
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.inner
            .insert(normalize_key(&key.into()), value.into().trim().to_string());
    }

    /// 按键名读取参数值（大小写不敏感）。
    pub fn get(&self, key: &str) -> Option<&str> {
        self.inner
            .get(&normalize_key(key))
            .map(std::string::String::as_str)
    }

    /// 判断是否包含指定参数键。
    pub fn contains(&self, key: &str) -> bool {
        self.inner.contains_key(&normalize_key(key))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// 借用型 MML 参数集合，值直接借用原始 MML 文本切片。
pub struct MmlParamsRef<'a> {
    inner: BTreeMap<String, &'a str>,
}

impl<'a> MmlParamsRef<'a> {
    /// 创建空借用参数集合。
    pub fn new() -> Self {
        Self::default()
    }

    /// 插入参数项，键会规范化，值保持借用并自动去除首尾空白。
    pub fn insert(&mut self, key: &str, value: &'a str) {
        self.inner.insert(normalize_key(key), value.trim());
    }

    /// 按键名读取参数值（大小写不敏感）。
    pub fn get(&self, key: &str) -> Option<&'a str> {
        self.inner.get(&normalize_key(key)).copied()
    }

    /// 判断是否包含指定参数键。
    pub fn contains(&self, key: &str) -> bool {
        self.inner.contains_key(&normalize_key(key))
    }
}

/// 统一参数读取接口：支持拥有型与借用型参数集合。
pub trait MmlParamLookup<'de> {
    /// 按键读取参数值。
    fn get(self, key: &str) -> Option<&'de str>;
    /// 判断参数键是否存在。
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

/// 规范化参数键名：去空白并转为 ASCII 大写。
fn normalize_key(key: &str) -> String {
    key.trim().to_ascii_uppercase()
}

/// 校验命令头操作类型和对象。
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

/// 解析完整 MML 行（拥有型参数）：`<OP> <OBJECT>:K=V,...;`。
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

/// 解析完整 MML 行（借用型参数）：`<OP> <OBJECT>:K=V,...;`。
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

/// 解析参数体 `K=V,...` 并构造成拥有型参数集合。
pub fn parse_mml_params(body: &str) -> Result<MmlParams, MmlError> {
    let mut params = MmlParams::new();
    for (k, v) in split_key_values(body)? {
        params.insert(k, v);
    }
    Ok(params)
}

/// 解析参数体 `K=V,...` 并构造成借用型参数集合。
pub fn parse_mml_params_ref<'a>(body: &'a str) -> Result<MmlParamsRef<'a>, MmlError> {
    let mut params = MmlParamsRef::new();
    for (k, v) in split_key_values_ref(body)? {
        params.insert(k, v);
    }
    Ok(params)
}

/// 把操作类型、对象和参数对组装为完整 MML 文本。
pub fn compose_mml_line(op: &str, object: &str, pairs: &[(String, String)]) -> String {
    let body = pairs
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join(",");
    format!("{} {}:{};", op.trim(), object.trim(), body)
}

/// 按键值对切分参数体（拥有型），支持引号与转义场景。
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

/// 按键值对切分参数体（借用型），值切片直接借用原始输入。
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

/// 解析文本参数值，必要时去引号并反转义（拥有型）。
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

/// 解析文本参数值（借用型）：仅去除外围引号，不做反转义。
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

/// 解析普通 token（拥有型，等价于 parse_text_value 后再 trim）。
pub fn parse_plain_token(raw: &str) -> Result<String, MmlError> {
    parse_text_value(raw).map(|s| s.trim().to_string())
}

/// 解析普通 token（借用型，等价于 parse_text_slice 后再 trim）。
pub fn parse_plain_token_ref<'a>(raw: &'a str) -> Result<&'a str, MmlError> {
    Ok(parse_text_slice(raw)?.trim())
}

/// 将文本编码为 MML 字面量（自动加引号与转义）。
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

/// 反转义字符串中的转义序列。
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

/// 定义类型与 MML 参数值之间的双向转换（拥有型）。
pub trait MmlValue: Sized {
    /// 从 MML 参数字符串解析当前类型。
    fn from_mml_value(raw: &str) -> Result<Self, MmlError>;
    /// 将当前类型编码为 MML 参数值字符串。
    fn to_mml_value(&self) -> String;
}

/// 定义支持“借用反序列化”的值类型转换。
pub trait MmlValueBorrowed<'de>: Sized {
    /// 从借用参数值解析当前类型。
    fn from_mml_value_borrowed(raw: &'de str) -> Result<Self, MmlError>;
}

/// 定义值类型序列化接口。
pub trait MmlValueEncode {
    /// 编码为 MML 参数字符串。
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

/// 定义字段级别的解析与编码行为。
pub trait MmlField<'de>: Sized {
    /// 判断字段是否在参数集合中存在。
    fn has_field<P>(field_name: &str, params: P) -> bool
    where
        P: MmlParamLookup<'de> + Copy,
    {
        params.contains(field_name)
    }

    /// 从参数集合读取并解析指定字段。
    fn from_mml_field<P>(field_name: &str, params: P) -> Result<Self, MmlError>
    where
        P: MmlParamLookup<'de> + Copy;

    /// 将字段编码后追加到参数输出列表。
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

/// 定义分支参数组（如 DID 分支）的解析与编码行为。
pub trait MmlBranch<'de>: Sized {
    /// 分支标签字段名；为空时使用外部传入字段名。
    const TAG: Option<&'static str>;

    /// 按标签字段和值解析具体分支。
    fn from_mml_branch<P>(tag_key: &str, params: P) -> Result<Self, MmlError>
    where
        P: MmlParamLookup<'de> + Copy;

    /// 将分支编码为标签和分支字段并写入输出。
    fn append_mml_branch(
        &self,
        tag_key: &str,
        out: &mut Vec<(String, String)>,
    ) -> Result<(), MmlError>;
}

/// 从参数集合反序列化为目标类型。
pub trait MmlDeserialize<'de>: Sized {
    /// 反序列化入口函数。
    fn from_mml_params<P>(params: P) -> Result<Self, MmlError>
    where
        P: MmlParamLookup<'de> + Copy;
}

/// 把目标类型序列化为参数列表。
pub trait MmlSerialize {
    /// 序列化入口函数。
    fn to_mml_params(&self) -> Result<Vec<(String, String)>, MmlError>;
}

/// 完整 MML 命令类型：包含命令头校验与参数读写能力。
pub trait MmlCommand: Sized + MmlSerialize {
    /// 命令操作类型常量（如 `ADD`）。
    const MML_OP: &'static str;
    /// 命令对象常量（如 `ASBR`）。
    const MML_OBJECT: &'static str;

    /// 从完整 MML 行解析当前命令对象（拥有型反序列化）。
    fn from_mml_line(input: &str) -> Result<Self, MmlError>
    where
        for<'de> Self: MmlDeserialize<'de>,
    {
        let (op, object, params) = parse_mml_line(input)?;
        validate_cmd_header(&op, &object, Self::MML_OP, Self::MML_OBJECT)?;
        <Self as MmlDeserialize<'_>>::from_mml_params(&params)
    }

    /// 从完整 MML 行解析当前命令对象（借用型反序列化）。
    fn from_mml_line_borrowed<'de>(input: &'de str) -> Result<Self, MmlError>
    where
        Self: MmlDeserialize<'de>,
    {
        let (op, object, params) = parse_mml_line_ref(input)?;
        validate_cmd_header(op, object, Self::MML_OP, Self::MML_OBJECT)?;
        <Self as MmlDeserialize<'de>>::from_mml_params(&params)
    }

    /// 将当前命令对象编码为完整 MML 行。
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
/// 为实现了 `FromStr + Display` 的类型快速生成 `MmlValue` 实现。
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
