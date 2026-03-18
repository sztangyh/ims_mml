use mml_def::*;
use mml_def::{MmlBranch, MmlValueEnum};

#[derive(Debug, Clone, MmlValueEnum)]
enum Regtp {
    Single,
    Unknow,
}

#[derive(Debug, Clone, MmlBranch)]
enum AsbrDidBranch {
    Esl { eid: MgwId, tid: u32 },
    Pra { trunkgp: u32 },
    V5st { v5iid: u32, l3addr: u32 },
}

type ImsUser = ImsUserNum<12>;

#[derive(Debug, Clone, MmlMessage)]
#[mml(op = "ADD", object = "ASBR")]
struct AddAsbr<'a> {
    pui: ImsUser,
    pri: ImsUser,
    regtp: Option<Regtp>,
    did: AsbrDidBranch,
    phncon: &'a str,
    digmap: Option<u32>,
    sgn: Option<String>,
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mmls = r#"
ADD ASBR:PUI="sip:+864117154@gd.ctcims.cn",PRI="+864117154@gd.ctcims.cn",REGTP=SINGLE,DID=ESL,MN=1003,TEN=EID,EID="5.154.224.60:2944",TID="40",TIDPFX=Index1,HNID="GDHNID",NETID="zjvnid",NETINFO="ZJ",PHNCON="ZJ",DIGMAP=6,PWD=")))B0E771812269226ADCC35736F546DC1C",EMGCN="110",DP=IMSUB,IFMIMN=1701,CGP=Normal,CXG=65535;
ADD ASBR:PUI="sip:+866682775050@gd.ctcims.cn",PRI="+866682775050@gd.ctcims.cn",REGTP=SINGLE,DID=ESL,MN=1004,TEN=EID,EID="10.101.132.29:2944",TID="127",TIDPFX=Index1,HNID="GDHNID",NETID="mmvnid",NETINFO="MM",PHNCON="MM",DIGMAP=7,PWD=")))B0E771812269226ADCC35736F546DC1C",EMGCN="110",DP=IMSUB,IFMIMN=1700,CGP=Normal,CXG=65535;
ADD ASBR:PUI="sip:+8675521515546@gd.ctcims.cn",PRI="+8675521515546@gd.ctcims.cn",REGTP=SINGLE,DID=V5ST,V5IID=1330704,L3ADDR=445,HNID="GDHNID",NETID="szvnid",NETINFO="SZTQ",PHNCON="SZ",DIGMAP=49,GLOBDMAPIDX=1,PWD=")))19E779E3FB48E5D0D6BC975EB688C9C6",SGN="scc9",EMGCN="110",DP=IMSUB,IFMIMN=1700,CXG=65535;
ADD ASBR:PUI="sip:+8675521515548@gd.ctcims.cn",PRI="+8675521515548@gd.ctcims.cn",REGTP=SINGLE,DID=V5ST,V5IID=1330704,L3ADDR=467,HNID="GDHNID",NETID="szvnid",NETINFO="SZTQ",PHNCON="SZ",DIGMAP=49,GLOBDMAPIDX=1,PWD=")))1E7A0A52F6CF8F78D5A25C942FC640C3",SGN="scc9",EMGCN="110",DP=IMSUB,IFMIMN=1700,CXG=65535;
ADD ASBR:PUI="sip:+8675521513771@gd.ctcims.cn",PRI="+8675521513771@gd.ctcims.cn",REGTP=SINGLE,DID=PRA,MN=1012,ISDN=1,TRUNKGP=28,HNID="GDHNID",NETID="szvnid",NETINFO="SZTQ",PHNCON="SZ",DIGMAP=0,PWD=")))DBE75888DD7B712C729530916344E993",EMGCN="110",IFMIMN=1700,WR=YES,CXG=65535;
ADD ASBR:PUI="sip:+8675521513772@gd.ctcims.cn",PRI="+8675521513772@gd.ctcims.cn",REGTP=SINGLE,DID=PRA,MN=1012,ISDN=1,TRUNKGP=28,HNID="GDHNID",NETID="szvnid",NETINFO="SZTQ",PHNCON="SZ",DIGMAP=0,PWD=")))628C479DE932A5360437990BF8DC6634",EMGCN="110",IFMIMN=1700,WR=YES,CXG=65535;
"#;
    let sz_pfx = U4Number::from(755);
    let mut error_count = 0usize;

    for (idx, m) in mmls.lines().enumerate() {
        let m = m.trim();
        if m.is_empty() {
            continue;
        }

        match AddAsbr::from_mml_line_borrowed(m) {
            Ok(r) => match r.did {
                AsbrDidBranch::Esl { eid, tid } => {
                    println!(
                        "ESL  => {} {} {} {} {:?}",
                        r.pui.strip_prefix(&sz_pfx), eid, tid, r.phncon, r.digmap
                    );
                }
                AsbrDidBranch::V5st { v5iid, l3addr } => {
                    println!(
                        "V5ST => {} {} {} {} {:?}",
                        r.pui.strip_prefix(&sz_pfx), v5iid, l3addr, r.phncon, r.digmap
                    );
                }
                AsbrDidBranch::Pra { trunkgp } => {
                    println!("PRA  => {} {} {} {:?}", r.pui.strip_prefix(&sz_pfx), trunkgp, r.phncon, r.digmap);
                }
            },
            Err(e) => {
                error_count += 1;
                eprintln!("line {} parse failed: {}", idx + 1, e);
            }
        }
    }

    if error_count > 0 {
        return Err(format!("{} line(s) failed to parse", error_count).into());
    }

    Ok(())
}
