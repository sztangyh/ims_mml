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
/// MML 参数集合，键名大小写不敏感（内部统一为大写）。
pub struct MmlParams {
    inner: BTreeMap<String, String>,
}

impl MmlParams {
    /// 闁告帗绋戠紓鎾寸▔閳ь剚绋夐鍡忔晞闁告瑥鍊归弳鐔兼⒖閸℃鍊ら柕?    
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

/// 规范化参数键名：去空白并转为 ASCII 大写。

fn normalize_key(key: &str) -> String {
    key.trim().to_ascii_uppercase()
}

/// 解析完整 MML 行：`<OP> <OBJECT>:K=V,...;`。

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

/// 解析参数体 `K=V,...` 并构造成参数集合。

pub fn parse_mml_params(body: &str) -> Result<MmlParams, MmlError> {
    let mut params = MmlParams::new();
    for (k, v) in split_key_values(body)? {
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

/// 按键值对切分参数体，支持引号与转义场景。

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

/// 解析文本参数值，必要时去引号并反转义。

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

/// 解析普通 token（等价于 parse_text_value 后再 trim）。

pub fn parse_plain_token(raw: &str) -> Result<String, MmlError> {
    parse_text_value(raw).map(|s| s.trim().to_string())
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

/// 定义类型与 MML 参数值之间的双向转换。

pub trait MmlValue: Sized {
    /// 从 MML 参数字符串解析当前类型。
    fn from_mml_value(raw: &str) -> Result<Self, MmlError>;
    /// 将当前类型编码为 MML 参数值字符串。
    fn to_mml_value(&self) -> String;
}

/// 定义字段级别的解析与编码行为。

pub trait MmlField: Sized {
    /// 判断字段是否在参数集合中存在。
    fn has_field(field_name: &str, params: &MmlParams) -> bool {
        params.contains(field_name)
    }

    /// 从参数集合读取并解析指定字段。

    fn from_mml_field(field_name: &str, params: &MmlParams) -> Result<Self, MmlError>;
    /// 将字段编码后追加到参数输出列表。
    fn append_mml_field(
        &self,
        field_name: &str,
        out: &mut Vec<(String, String)>,
    ) -> Result<(), MmlError>;
}

impl<T> MmlField for T
where
    T: MmlValue,
{
    fn from_mml_field(field_name: &str, params: &MmlParams) -> Result<Self, MmlError> {
        let raw = params
            .get(field_name)
            .ok_or_else(|| MmlError::MissingField(field_name.to_string()))?;
        T::from_mml_value(raw).map_err(|e| MmlError::ValueParse {
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
        out.push((field_name.to_string(), self.to_mml_value()));
        Ok(())
    }
}

/// 定义分支参数组（如 DID 分支）的解析与编码行为。

pub trait MmlBranch: Sized {
    /// 分支标签字段名；为空时使用外部传入字段名。
    const TAG: Option<&'static str>;

    /// 按标签字段和值解析具体分支。

    fn from_mml_branch(tag_key: &str, params: &MmlParams) -> Result<Self, MmlError>;
    /// 将分支编码为标签和分支字段并写入输出。
    fn append_mml_branch(
        &self,
        tag_key: &str,
        out: &mut Vec<(String, String)>,
    ) -> Result<(), MmlError>;
}

/// 从参数集合反序列化为目标类型。

pub trait MmlDeserialize: Sized {
    /// 反序列化入口函数。
    fn from_mml_params(params: &MmlParams) -> Result<Self, MmlError>;
}

/// 把目标类型序列化为参数列表。

pub trait MmlSerialize {
    /// 序列化入口函数。
    fn to_mml_params(&self) -> Result<Vec<(String, String)>, MmlError>;
}

/// 完整 MML 命令类型：包含命令头校验与参数读写能力。

pub trait MmlCommand: Sized + MmlDeserialize + MmlSerialize {
    /// 命令操作类型常量（如 `ADD`）。
    const MML_OP: &'static str;
    /// 命令对象常量（如 `ASBR`）。
    const MML_OBJECT: &'static str;

    /// 从完整 MML 行解析当前命令对象。

    fn from_mml_line(input: &str) -> Result<Self, MmlError> {
        let (op, object, params) = parse_mml_line(input)?;
        if !Self::MML_OP.is_empty() && !op.eq_ignore_ascii_case(Self::MML_OP) {
            return Err(MmlError::InvalidLine(format!(
                "expected op {}, got {op}",
                Self::MML_OP
            )));
        }
        if !Self::MML_OBJECT.is_empty() && !object.eq_ignore_ascii_case(Self::MML_OBJECT) {
            return Err(MmlError::InvalidLine(format!(
                "expected object {}, got {object}",
                Self::MML_OBJECT
            )));
        }
        Self::from_mml_params(&params)
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
                    v.parse::<$ty>().map_err(|_e| {
                        $crate::MmlError::InvalidParam(
                            format!("invalid {}: {}", stringify!($ty), v),
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
