use encoding_rs::*;
use mml_def::MmlValueEnum;
use mml_def::*;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Clone, MmlValueEnum, PartialEq, Eq)]
enum MSTYPE {
    Master,
    Slave,
}

#[derive(Debug, Clone, MmlValueEnum, PartialEq, Eq)]
enum GWTP {
    Ag,
    Tg,
    Iad,
    Umgw,
    Mrs,
    Mta,
}

#[derive(Debug, Clone, MmlMessage)]
#[mml(op = "ADD", object = "MGW")]
struct Mgw {
    eid: MgwId,
    gwtp: GWTP,
    mgwdesc: String,
    mstype: Option<MSTYPE>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AGCF {
    AGCF51,
    AGCF52,
    AGCF63,
    AGCF64,
}

impl AGCF {
    pub fn peer(&self) -> Self {
        use AGCF::*;
        match self {
            AGCF51 => AGCF52,
            AGCF52 => AGCF51,
            AGCF63 => AGCF64,
            AGCF64 => AGCF63,
        }
    }
}

fn load_eids(fname: &str) -> Result<Vec<Option<MgwId>>, Box<dyn Error>> {
    let mut file = File::open(fname)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    let eids = buf
        .lines()
        .map(|ln| MgwId::from_str(ln).ok())
        .collect::<Vec<_>>();
    Ok(eids)
}

fn main() {
    let agcfs = [
        (AGCF::AGCF51, "AGCF51_20240422"),
        (AGCF::AGCF52, "AGCF52_20240422"),
        (AGCF::AGCF63, "AGCF63_20240422"),
        (AGCF::AGCF64, "AGCF64_20240422"),
    ]
    .as_slice();

    let mut mgws = vec![];
    let mut eid_all: HashMap<[u8; 4], (Option<AGCF>, Option<AGCF>)> = HashMap::new();

    let _test_ip = [10u8, 98u8, 200u8, 10u8];
    let logfile_root = std::env::var("MML_LOG_DIR").unwrap_or_else(|_| "logfile".to_string());
    for &(agcf, name) in agcfs.iter() {
        let mut buf = Vec::new();
        let mut agcf_file = PathBuf::from(&logfile_root);
        agcf_file.push("agcf");
        agcf_file.push(format!("{name}.txt"));
        let mut file = File::open(agcf_file).unwrap();
        file.read_to_end(&mut buf).unwrap();
        let (buf, _, _) = SHIFT_JIS.decode(&buf);
        let mut lines = buf.lines();
        let mut count = 0u32;
        while let Some(ln) = lines.next() {
            if let Ok(mgw) = Mgw::from_mml_line(ln) {
                if mgw.gwtp == GWTP::Ag {
                    mgws.push((agcf, mgw));
                    count += 1;
                }
            } else if count > 0 {
                break;
            }
        }
        println!("{:?} loaded, records = {}", agcf, count);
    }
    println!("{} Mgws records", mgws.len());

    for (agcf, mgw) in mgws.iter() {
        eid_all
            .entry(mgw.eid.ip.clone())
            .and_modify(|e| {
                if let Some(MSTYPE::Slave) = mgw.mstype {
                    e.1 = Some(*agcf);
                } else {
                    e.0 = Some(*agcf);
                }
            })
            .or_insert(match mgw.mstype {
                Some(MSTYPE::Slave) => (None, Some(*agcf)),
                _ => (Some(*agcf), None),
            });
    }

    println!("{} eids", eid_all.len());
    let eids_file = std::env::var("MML_EIDS_FILE").unwrap_or_else(|_| {
        let mut f = PathBuf::from(&logfile_root);
        f.push("eids.txt");
        f.to_string_lossy().to_string()
    });
    let fttbs = load_eids(&eids_file).unwrap();
    for e in fttbs {
        if let Some(mgw) = e {
            print!("{}\t", mgw);
            match eid_all.get(mgw.ip.as_ref()) {
                Some((Some(a1), Some(a2))) => {
                    println!("{:?}\t{:?}", a1, a2);
                }
                Some((Some(a1), None)) => {
                    println!("{:?}\t", a1);
                }
                Some((None, Some(a2))) => {
                    println!("\t{:?}", a2)
                }
                _ => println!("\t"),
            }
        } else {
            println!("0\t\t");
        }
    }
}
