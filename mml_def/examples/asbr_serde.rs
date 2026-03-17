use mml_def::{
    encode_text_value, parse_text_value, MmlBranch, MmlCommand, MmlError, MmlMessage, MmlValue,
    MmlValueEnum,
};

#[derive(Debug, Clone, PartialEq, Eq, MmlValueEnum)]
enum Regtp {
    Single,
    Unknow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Endpoint(String);

impl MmlValue for Endpoint {
    fn from_mml_value(raw: &str) -> Result<Self, MmlError> {
        Ok(Self(parse_text_value(raw)?))
    }

    fn to_mml_value(&self) -> String {
        encode_text_value(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, MmlBranch)]
enum AsbrDid {
    Esl { eid: Endpoint, tid: u32 },
    V5st { v5iid: u32, l3addr: u32 },
}

#[derive(Debug, Clone, PartialEq, MmlMessage)]
#[mml(op = "ADD", object = "ASBR")]
struct AddAsbr {
    pui: String,
    pri: String,
    regtp: Regtp,
    did: AsbrDid,
    hnid: String,
    netid: String,
    netinfo: String,
    phncon: String,
    digmap: u32,
    #[mml(skip)]
    parsed_at: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let line_esl = r#"ADD  ASBR:PUI = "sip:+8675521008574@gd.ctcims.cn",PRI="+8675521008574@gd.ctcims.cn",REGTP=SINGLE,DID=ESL,EID="10.98.254.7:2944",TID="5",HNID="GDHNID",NETID="szvnid",NETINFO="SZBA",PHNCON="SZ",DIGMAP=1;"#;
    let line_v5 = r#"ADD ASBR:PUI="sip:+8675521008866@gd.ctcims.cn",PRI="+8675521008866@gd.ctcims.cn",REGTP=SINGLE,DID=V5ST,V5IID=1330807,L3ADDR=3174,HNID="GDHNID",NETID="szvnid",NETINFO="SZ",PHNCON="SZ",DIGMAP=49;"#;

    let esl = AddAsbr::from_mml_line(line_esl)?;
    let v5 = AddAsbr::from_mml_line(line_v5)?;

    if let AsbrDid::Esl { eid, tid } = &esl.did {
        assert_eq!(eid.0, "10.98.254.7:2944");
        assert_eq!(*tid, 5);
    } else {
        panic!("expected ESL branch");
    }

    if let AsbrDid::V5st { v5iid, l3addr } = &v5.did {
        assert_eq!(*v5iid, 1330807);
        assert_eq!(*l3addr, 3174);
    } else {
        panic!("expected V5ST branch");
    }

    println!("{}", esl.to_mml_line()?);
    println!("{}", v5.to_mml_line()?);
    Ok(())
}
