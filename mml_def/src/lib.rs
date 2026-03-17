mod dat_typs;
mod mml_serde;
// Legacy macro modules in `src/mml_parse*.rs` / `src/mml_enum.rs` are kept as historical
// references; the active macro entry points are proc-macros from `mml_def_derive`.

pub use dat_typs::{
    AgcfIpaddr, FormatError, ImsUserNum, MgwId, Pfx, RangeOfPrefix, U4Number, U4NumberDivided,
    U4NumberVec, N7SPC,
};
pub use mml_serde::{
    compose_mml_line, encode_text_value, parse_mml_line, parse_mml_params, parse_plain_token,
    parse_text_value, MmlBranch, MmlCommand, MmlDeserialize, MmlError, MmlField, MmlParams,
    MmlSerialize, MmlValue,
};

pub use mml_def_derive::{mml_def, mml_easy, mml_enum, MmlBranch, MmlMessage, MmlValueEnum};

extern crate self as mml_def;

#[derive(Debug, Clone)]
/// 历史宏 `mml_enum!` 产生的枚举值解析失败错误。
pub struct InvalidEnumValue;

#[allow(unused)]
/// 解析完整 MML 行并返回 `(操作类型, 操作对象, 参数体)`。
pub fn split_mml<'a>(s: &'a str) -> Option<(&'a str, &'a str, &'a str)> {
    let (hd, bd) = s.split_once(':').map(|(hd, bd)| (hd, bd.trim()))?;
    let (op, sub) = hd
        .split_once(' ')
        .map(|(op, sub)| (op.trim(), sub.trim()))?;
    let body = if bd.ends_with(';') {
        &bd[..bd.len() - 1]
    } else {
        return None;
    };
    Some((op, sub, body))
}

/// 旧版宏体系中的参数值转换 Trait。

pub trait FromPara<'a>: Sized {
    type Err;
    /// 将单个参数值字符串转换为目标类型。
    fn from_para(val: &'a str) -> Result<Self, Self::Err>;
}

macro_rules! auto_trait_FromPara {
    ($($ty:ty),*) => {
        $(
            impl<'a> FromPara<'a> for $ty {
                type Err = <Self as std::str::FromStr>::Err;
                fn from_para(val: &'a str) -> Result<Self, Self::Err> {
                    let val = match val.as_bytes() {
                        &[b'"', .., b'"'] => &val[1..val.len() - 1],
                        _ => val,
                    };
                    <Self as std::str::FromStr>::from_str(val)
                }
            }
        )*
    };
}

impl<'a, const NUM_SIZE: usize> FromPara<'a> for ImsUserNum<NUM_SIZE> {
    type Err = <ImsUserNum<NUM_SIZE> as FromStr>::Err;
    fn from_para(val: &'a str) -> Result<Self, Self::Err> {
        if let &[b'"', ref dq @ .., b'"'] = val.as_bytes() {
            unsafe { std::str::from_utf8_unchecked(dq).parse() }
        } else {
            val.parse()
        }
    }
}

auto_trait_FromPara!(u8, u16, u32, u64, String, MgwId, AgcfIpaddr, N7SPC, Pfx);

mml_enum!(AsbrDid, ESL, PRA, V5ST, SIPPBX);

use std::{convert::Infallible, str::FromStr};
impl<'a> FromPara<'a> for &'a str {
    type Err = Infallible;
    fn from_para(val: &'a str) -> Result<Self, Self::Err> {
        match val.as_bytes() {
            &[b'"', .., b'"'] => Ok(&val[1..val.len() - 1]),
            _ => Ok(val),
        }
    }
}

#[derive(Debug, Clone)]
/// 旧版宏体系中的参数解析错误类型。
pub enum ParaParseError {
    NotFound(String),
    InvalidFormat(String),
    FailMapping(String),
}

impl std::fmt::Display for ParaParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFormat(e) => write!(f, "参数格式错误: {e}"),
            Self::NotFound(e) => write!(f, "参数未找到: {e}"),
            Self::FailMapping(e) => write!(f, "参数映射失败: {e}"),
        }
    }
}

impl std::error::Error for ParaParseError {}

/// 遍历参数体中的 `K=V` 键值项（按逗号分割）。

pub fn mml_paras_iter<'a>(body: &'a str) -> impl Iterator<Item = (&'a str, &'a str)> {
    body.split(',')
        .map(|kv| kv.split_once('=').map(|(k, v)| (k.trim(), v.trim())))
        .filter(|kv| kv.is_some())
        .map(|kv| kv.unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::rc::Rc;

    mml_def!(CSbr, Sbr<'a>, pwd:Ignore, pui:&'a str, vnet:Absent, eid:String, tid:u32, ot:Ignore);

    mml_def!(CSbr2, Sbr2, pui:String, vnet:Mapped, eid:String, tid:u32);
    #[test]
    fn it_works() {
        let mml1 =
            "Pui=8298001, vnet=sz, eid=\"127.0.0.1\", tid=923, ot=xxoo, dd=yyy, pwd=NHDJKO12L";
        let mml2 =
            "Pui=8298444, vnet=gz, eid=\"127.0.0.1\", tid=923, ot=xxoo, dd=yyy, pwd=NHDJKO12L";
        let mut mper: HashMap<Rc<str>, Rc<str>> =
            HashMap::from([(Rc::from("gz"), Rc::from("gz(020)"))]);
        let c1 = Rc::new(CSbr {
            vnet: "sz".to_owned(),
        });
        let s1 = c1.from_mml_body(mml1);
        let s2 = c1.from_mml_body(mml2);
        println!("{:?}", s1);
        assert!(s1.ok().map_or(false, |s| s.pui == "8298001"
            && s.tid == 923
            && s.differs.contains_key("dd")));
        println!("{:?}", s2);
        assert!(s2.ok().map_or(false, |s| s.pui == "8298444"
            && s.tid == 923
            && s.differs.contains_key("vnet")));

        let c1 = Rc::new(CSbr2 {});
        let s1 = c1.from_mml_body(&mut mper, mml1);
        let s2 = c1.from_mml_body(&mut mper, mml2);
        println!("{:?}", s1);
        assert!(s1.ok().map_or(false, |s| s.vnet.as_ref() == "sz"
            && s.tid == 923
            && s.differs.contains_key("dd")));
        println!("{:?}", s2);
        assert!(s2.ok().map_or(false, |s| {
            s.vnet.as_ref() == "gz(020)" && s.tid == 923 && s.differs.contains_key("pwd")
        }));
    }
}
