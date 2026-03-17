
#[derive(Debug, Clone)]
/// 旧版 `mml_enum!` 宏中使用的枚举解析错误类型。
pub struct InvalidEnumValue;

#[allow(unused)]
#[macro_export]
macro_rules! mml_enum{
    ($enum:ident, $($id:ident),+) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        /// 由 `mml_enum!` 生成的大小写不敏感枚举类型。
        pub enum $enum {
            $($id,)+
        }
        impl std::str::FromStr for $enum {
            type Err = InvalidEnumValue;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $(
                        stringify!($id) => Ok(Self::$id),
                    )+
                    _ => Err(InvalidEnumValue)
                }
            }
        }

        impl<'a> FromPara<'a> for $enum {
            type Err = InvalidEnumValue;

            fn from_para(s: &'a str) -> Result<Self, Self::Err> {
                $(
                    if s.eq_ignore_ascii_case(stringify!($id)) { 
                        Ok(Self::$id)
                    } else 
                )+
                { Err(InvalidEnumValue) }
            }
        }
        impl std::fmt::Display for $enum{
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }
    }
}

pub use mml_enum;


