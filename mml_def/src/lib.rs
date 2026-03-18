mod dat_typs;
mod mml_serde;
// Active macro entry points are proc-macros from mml_def_derive.

pub use dat_typs::{
    AgcfIpaddr, FormatError, ImsUserNum, MgwId, Pfx, RangeOfPrefix, U4Number, U4NumberDivided,
    U4NumberVec, N7SPC,
};
pub use mml_serde::{
    compose_mml_line, deserialize_params, deserialize_params_ref, encode_text_value, parse_mml_line, parse_mml_line_ref, parse_mml_params,
    parse_mml_params_ref, parse_text_slice, parse_text_value, MmlError, MmlParams, MmlParamsRef,
    MmlValue,
};

#[doc(hidden)]
pub mod __private {
    pub use crate::mml_serde::{
        parse_plain_token, parse_plain_token_ref, MmlBranch, MmlCommand, MmlDeserialize, MmlField,
        MmlParamLookup, MmlSerialize, MmlValueBorrowed, MmlValueEncode, MmlError,
    };
}

pub use mml_def_derive::{MmlBranch, MmlMessage, MmlValueEnum};

extern crate self as mml_def;