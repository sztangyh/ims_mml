use mml_def::*;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use encoding_rs::*;

type IMSUB = ImsUserNum<12>;
type SZSUB = U4Number<12>;

#[derive(Debug, Clone, MmlMessage)]
#[mml(op = "ADD", object = "ASBR")]
struct Snb {
    pui: IMSUB,
    did: String,
    trunkgp: Option<u32>,
}

fn read_sipusers(fname: &str) -> Vec<SZSUB> {
    let sz_ac: SZSUB = "755".into();
    let mut snbs = vec![];

    let mut buf = Vec::new();
    let mut file = File::open(fname).unwrap();
    file.read_to_end(&mut buf).unwrap();
    let (bb, _, _) = SHIFT_JIS.decode(&buf);
    let mut lines = bb.lines();
    while let Some(line) = lines.next() {
        if let Ok(snb) = Snb::from_mml_line(line) {
            if snb.trunkgp.is_some() {
                snbs.push(snb.pui.strip_prefix(&sz_ac));
            }
        }
    }
    snbs
}

fn main() {
    let pfx = "866060;8660610;8660611;866080;866081;866082;866083;866084";
    let fname = std::env::var("MML_SIPUSERS_FILE").unwrap_or_else(|_| {
        let mut f = PathBuf::from("logfile");
        f.push("agcf");
        f.push("AGCF65_20240506.txt");
        f.to_string_lossy().to_string()
    });

    let snbs = read_sipusers(&fname);
    let pfxs: U4NumberVec<12> = U4NumberVec::new(pfx);
    let r = snbs.iter().filter(|&n| pfxs.include(n)).collect::<Vec<_>>();

    println!(
        "sip涓户涓?255鐨勫彿鐮?={}\t绗﹀悎瀛楀啝鏉′欢鐨勫彿鐮?= {}",
        snbs.len(),
        r.len()
    );
    for snb in r {
        println!("{}", snb);
    }
}
