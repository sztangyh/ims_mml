#![allow(unused)]
use std::fmt::Write;

/*
88908001
75588908001
075588908001
+8675588908001
008675588908001
13360096745
013360096745
+8613360096745
sip:+8675588908001@gd.ctcims.cn
tel:+8675588908001
1.0.0.8.9.9.8.8.5.5.7.6.8xxxx
K'xxxxxx
*57#1234445#
26778888-2213

*/
use super::U4Number;
use once_cell::sync::OnceCell;
use regex::Regex;

#[derive(Debug, Clone, Copy)]
/// IMS 用户号码的统一表示（PUI/PRI/RAW/ENUM）。
pub enum ImsUserNum<const NUM_SIZE: usize> {
    PUI(U4Number<NUM_SIZE>),
    PRI(U4Number<NUM_SIZE>),
    TEL(U4Number<NUM_SIZE>),
    RAW(U4Number<NUM_SIZE>),
    ENUM(U4Number<NUM_SIZE>),
}

static RE_PUI: OnceCell<Regex> = OnceCell::new();
static RE_PRI: OnceCell<Regex> = OnceCell::new();
static RE_TEL: OnceCell<Regex> = OnceCell::new();
static RE_ENUM: OnceCell<Regex> = OnceCell::new();

impl<const NUM_SIZE: usize> ImsUserNum<NUM_SIZE> {}

impl<const NUM_SIZE: usize> std::ops::Deref for ImsUserNum<NUM_SIZE> {
    type Target = U4Number<NUM_SIZE>;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::PUI(n) => n,
            Self::PRI(n) => n,
            Self::TEL(n) => n,
            Self::RAW(n) => n,
            Self::ENUM(n) => n,
        }
    }
}

impl<const NUM_SIZE: usize> std::str::FromStr for ImsUserNum<NUM_SIZE> {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re_pui =
            RE_PUI.get_or_init(|| Regex::new(r"(?i)^sip:\+86(\d+)@gd.ctcims.cn$").unwrap());
        let re_pri = RE_PRI.get_or_init(|| Regex::new(r"(?i)^\+86(\d+)@gd.ctcims.cn$").unwrap());
        let re_tel = RE_TEL.get_or_init(|| Regex::new(r"(?i)^tel:\+86(\d+)$").unwrap());
        let re_enum =
            RE_ENUM.get_or_init(|| Regex::new(r"(?i)^((?:\d\.)+)6.8.e164.arpa$").unwrap());

        if let Some(g) = re_pui.captures(s) {
            //dbg!(&g[1]);
            Ok(Self::PUI(g[1].into()))
        } else if let Some(g) = re_pri.captures(s) {
            Ok(Self::PRI(g[1].into()))
        } else if let Some(g) = re_tel.captures(s) {
            //dbg!(&g[1]);
            Ok(Self::TEL(g[1].into()))
        } else if let Some(g) = re_enum.captures(s) {
            //dbg!(&g[1]);
            let buf: Vec<u8> = g[1]
                .as_bytes()
                .iter()
                .enumerate()
                .filter(|(i, c)| *i % 2 == 0)
                .map(|(i, c)| *c)
                .rev()
                .collect();
            Ok(Self::ENUM(buf.as_slice().into()))
        } else {
            //dbg!(s);
            let raw: U4Number<NUM_SIZE> = s.parse()?;
            Ok(Self::RAW(raw))
        }
    }
}

impl<const NUM_SIZE: usize> std::fmt::Display for ImsUserNum<NUM_SIZE> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PUI(n) => write!(f, "sip:+86{}@gd.ctcims.cn", n),
            Self::PRI(n) => write!(f, "+86{}@gd.ctcims.cn", n),
            Self::TEL(n) => write!(f, "tel:+86{}", n),
            Self::RAW(n) => write!(f, "{}", n),
            Self::ENUM(n) => {
                for i in (0..n.len()).rev() {
                    f.write_char((n.get_at(i) + b'0') as char)?;
                    f.write_char('.')?;
                }
                write!(f, "6.8.e164.arpa")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// 前缀类型，文本格式为 `K'xxxx`。
pub struct Pfx(U4Number<16>);

impl std::str::FromStr for Pfx {
    type Err = super::FormatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("K'") || s.starts_with("k'") {
            let s = &s[2..];
            let n = s.parse().map_err(|e| super::FormatError)?;
            Ok(Pfx(n))
        } else {
            Err(super::FormatError)
        }
    }
}

impl std::fmt::Display for Pfx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "K'{}", self.0.to_pfx())
    }
}
