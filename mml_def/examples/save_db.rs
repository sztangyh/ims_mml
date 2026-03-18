use mml_def::MmlValueEnum;
use mml_def::*;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::Read;

use encoding_rs::*;

#[derive(Debug, Clone, MmlValueEnum)]
enum ModuleType {
    ACU,
    IFM,
    CDB,
    BSG,
}

#[derive(Debug, Clone, MmlValueEnum, PartialEq, Eq)]
enum MSTYPE {
    MASTER,
    SLAVE,
}

#[derive(Debug, Clone, MmlValueEnum, PartialEq, Eq)]
enum GWTP {
    AG,
    TG,
    IAD,
    UMGW,
    MRS,
    MTA,
}

#[derive(Debug, Clone, MmlValueEnum)]
enum DOT {
    PBX,
    CC,
    CMPX,
    NATT,
    INTT,
    MANT,
    IPTMSC,
    PTMSC,
    MSC,
    BSC,
    GMSC,
    IN,
    TOLL,
    TS,
    LS,
    OTHER,
}

#[derive(Debug, Clone, MmlValueEnum)]
enum DOL {
    HIGH,
    SAME,
    LOW,
}

#[derive(Debug, Clone, MmlValueEnum)]
enum YesNo {
    YES,
    NO,
}

#[derive(Debug, Clone, MmlValueEnum)]
enum SPCHG {
    SPCHG,
    NOSPCHG,
}

#[derive(Debug, Clone, MmlValueEnum)]
enum PraTkcCS {
    USE,
    UNU,
}

#[derive(Debug, Clone, MmlValueEnum)]
enum ClrdsnFunc {
    NIN,
    DSG,
    ATT,
    DSGATT,
    CLRTYP,
    NOPROC,
}

#[derive(Debug, Clone, MmlMessage)]
#[mml(op = "ADD", object = "MODULE")]
struct Module {
    mid: u32,
    mt: ModuleType,
}

#[derive(Debug, Clone, MmlMessage)]
#[mml(op = "ADD", object = "MGW")]
struct Mgw {
    eid: MgwId,
    gwtp: GWTP,
    mgwdesc: String,
    mstype: Option<MSTYPE>,
}

#[derive(Debug, Clone, MmlMessage)]
#[mml(op = "ADD", object = "OFC")]
struct Ofc {
    o: u32,
    on: String,
    dot: DOT,
    dol: DOL,
    dpc1: Option<String>,
}

#[derive(Debug, Clone, MmlMessage)]
#[mml(op = "ADD", object = "SRT")]
struct Srt {
    src: u32,
    o: u32,
}

#[derive(Debug, Clone, MmlMessage)]
#[mml(op = "ADD", object = "RT")]
struct Rt {
    r: u32,
    sr1: u32,
    sr2: Option<u32>,
}

#[derive(Debug, Clone, MmlMessage)]
#[mml(op = "ADD", object = "RTANA")]
struct Rtana {
    rsc: u32,
    rssc: u32,
    r: u32,
}

#[derive(Debug, Clone, MmlMessage)]
#[mml(op = "ADD", object = "CNACLD")]
struct Cnacld {
    lp: u32,
    pfx: String,
    rsc: u32,
    minl: u8,
    maxl: u8,
    clraf: Option<YesNo>,
    spchg: Option<SPCHG>,
}

#[derive(Debug, Clone, MmlMessage)]
#[mml(op = "MOD", object = "CNACLD")]
struct CnacldM {
    lp: u32,
    pfx: String,
    spchg: SPCHG,
}

#[derive(Debug, Clone, MmlMessage)]
#[mml(op = "ADD", object = "SIPTG")]
struct Siptg {
    tg: u32,
    csc: u32,
    srt: u32,
    tgn: String,
    hcic: u32,
    lcic: u32,
    si: u32,
    icr: Option<String>,
    ocr: Option<String>,
}

#[derive(Debug, Clone, MmlMessage)]
#[mml(op = "ADD", object = "PRATG")]
struct Pratg {
    tg: u32,
    tgn: String,
    mgw: MgwId,
    src: u32,
    link: u32,
    cdef: String,
    icr: Option<String>,
    ocr: Option<String>,
    maxi: Option<u32>,
    tgms: Option<MSTYPE>,
    mtg: Option<u32>,
}

#[derive(Debug, Clone, MmlMessage)]
#[mml(op = "ADD", object = "PRALNK")]
struct Pralnk {
    pln: u32,
    scn: u32,
    mn: u32,
    lks: u32,
    binifid: u32,
}

#[derive(Debug, Clone, MmlMessage)]
#[mml(op = "ADD", object = "PRATKC")]
struct Pratkc {
    mn: u32,
    tg: u32,
    sc: u32,
    tid: u32,
    cs: PraTkcCS,
}

#[derive(Debug, Clone, MmlMessage)]
#[mml(op = "ADD", object = "TGDSG")]
struct Tgdsg {
    tg: u32,
    dsg: u32,
}

#[derive(Debug, Clone, MmlMessage)]
#[mml(op = "MOD", object = "TGSRT")]
struct TgSrt {
    tg: u32,
    src: u32,
}

#[derive(Debug, Clone, MmlMessage)]
#[mml(op = "ADD", object = "CLRDSN")]
struct Clrdsn {
    dsp: u32,
    cli: String,
    func: ClrdsnFunc,
    min: Option<u8>,
    max: Option<u8>,
    rdcx: Option<u32>,
}

#[derive(Debug, Clone, MmlMessage)]
#[mml(op = "ADD", object = "TGAP")]
struct Tgap {
    tg: u32,
    cdfp: String,
    name: Option<String>,
    asvrcf: Option<String>,
}

type UserNum = ImsUserNum<12>;

#[derive(Debug, Clone, MmlMessage)]
#[mml(op = "ADD", object = "ASBR")]
struct AddAsbr {
    pui: UserNum,
    did: String,
    trunkgp: Option<u32>,
    eid: Option<String>,
    tid: Option<u32>,
    v5iid: Option<u32>,
    l3addr: Option<u32>,
    netinfo: Option<String>,
    phncon: Option<String>,
    digmap: Option<u32>,
    ifmimn: Option<u32>,
    sgn: Option<String>,
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct PraUser {
    tg: u32,
    tgn: String,
    mgw: MgwId,
    route: (u32, u32, u32),
    lnk: u32,
    icr: Option<String>,
    ocr: String,
    cdef: String,
    diff: BTreeMap<String, String>,
    mn: u32,
    tkc_sc: u32,
    tkc_tid: u32,
    lnk_scn: u32,
    lks: u32,
    binifid: u32,
    ofc: u32,
    dsg: Option<u32>,
    maxi: Option<u32>,
    area_code: Option<String>,
    mtg: Option<u32>,
    cdfp: Option<String>,
}

#[derive(Debug, Default)]
struct AgcfData {
    modules: BTreeMap<u32, ModuleType>,
    mgw: Vec<Mgw>,
    pratg: Vec<PraUser>,
    siptg: Vec<Siptg>,
    cnacld: Vec<Cnacld>,
    asbr_esl: Vec<AddAsbr>,
    asbr_pra: Vec<AddAsbr>,
    asbr_sip: Vec<AddAsbr>,
    asbr_v5: Vec<AddAsbr>,
}
fn required_env(key: &str) -> Result<String, std::io::Error> {
    std::env::var(key).map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("missing env var: {key}"),
        )
    })
}

fn main() -> Result<(), std::io::Error> {
    let fname = required_env("MML_INPUT_FILE")?;
    let mut agcf = AgcfData::default();

    let mut ofc_tbl: BTreeMap<u32, Ofc> = BTreeMap::new();
    let mut srt_tbl: BTreeMap<u32, u32> = BTreeMap::new();
    let mut rt_tbl: BTreeMap<u32, u32> = BTreeMap::new();
    let mut rtana_tbl: BTreeMap<u32, (u32, u32)> = BTreeMap::new();
    let mut cnacld_spchg: BTreeMap<(u32, String), SPCHG> = BTreeMap::new();
    let mut tgsrt_tbl: BTreeMap<u32, u32> = BTreeMap::new();
    let mut pratg_list: Vec<Pratg> = vec![];
    let mut pralnk_list: BTreeMap<u32, Pralnk> = BTreeMap::new();
    let mut pratkc_tbl: BTreeMap<u32, (u32, u32, u32)> = BTreeMap::new();
    let mut tgdsg_tbl: BTreeMap<u32, u32> = BTreeMap::new();
    let mut tgap_tbl: BTreeMap<u32, Tgap> = BTreeMap::new();

    let mut buf = Vec::new();
    let mut file = File::open(&fname)?;
    file.read_to_end(&mut buf)?;
    let (bb, _, _) = GBK.decode(&buf);
    let mut lines = bb.lines();
    while let Some(line) = lines.next() {
        let Ok((op, obj, params)) = parse_mml_line(line) else {
            continue;
        };

        match (op.as_str(), obj.as_str()) {
            ("ADD", "MODULE") => {
                if let Some(m) = deserialize_params::<Module>(&params).ok() {
                    agcf.modules.insert(m.mid, m.mt);
                }
            }
            ("ADD", "MGW") => {
                if let Some(mgw) = deserialize_params::<Mgw>(&params).ok() {
                    if mgw.gwtp == GWTP::AG {
                        agcf.mgw.push(mgw);
                    }
                }
            }
            ("ADD", "PRALNK") => {
                if let Some(lnk) = deserialize_params::<Pralnk>(&params).ok() {
                    pralnk_list.insert(lnk.pln, lnk);
                }
            }
            ("ADD", "OFC") => {
                if line.contains("=LOW") {
                    if let Some(o) = deserialize_params::<Ofc>(&params).ok() {
                        ofc_tbl.insert(o.o, o);
                    }
                }
            }
            ("ADD", "SRT") => {
                if let Some(sr) = deserialize_params::<Srt>(&params).ok() {
                    srt_tbl.insert(sr.src, sr.o);
                }
            }
            ("ADD", "RT") => {
                if let Some(r) = deserialize_params::<Rt>(&params).ok() {
                    if r.sr2.is_none() {
                        rt_tbl.insert(r.sr1, r.r);
                    }
                }
            }
            ("ADD", "RTANA") => {
                if let Some(ana) = deserialize_params::<Rtana>(&params).ok() {
                    rtana_tbl.insert(ana.r, (ana.rsc, ana.rssc));
                }
            }
            ("ADD", "CNACLD") => {
                if let Some(cna) = deserialize_params::<Cnacld>(&params).ok() {
                    agcf.cnacld.push(cna);
                }
            }
            ("ADD", "SIPTG") => {
                if let Some(sipt) = deserialize_params::<Siptg>(&params).ok() {
                    agcf.siptg.push(sipt);
                }
            }
            ("ADD", "PRATG") => {
                if let Some(prat) = deserialize_params::<Pratg>(&params).ok() {
                    pratg_list.push(prat);
                }
            }
            ("ADD", "PRATKC") => {
                if let Some(tkc) = deserialize_params::<Pratkc>(&params).ok() {
                    pratkc_tbl
                        .entry(tkc.tg)
                        .or_insert((tkc.mn, tkc.sc, tkc.tid));
                }
            }
            ("ADD", "TGDSG") => {
                if let Some(dsg) = deserialize_params::<Tgdsg>(&params).ok() {
                    tgdsg_tbl.insert(dsg.tg, dsg.dsg);
                }
            }
            ("ADD", "ASBR") => {
                if let Some(asbr) = deserialize_params::<AddAsbr>(&params).ok() {
                    match asbr.did.to_ascii_uppercase().as_str() {
                        "ESL" => agcf.asbr_esl.push(asbr),
                        "V5ST" => agcf.asbr_v5.push(asbr),
                        "PRA" => agcf.asbr_pra.push(asbr),
                        "SIPPBX" => agcf.asbr_sip.push(asbr),
                        _ => {}
                    }
                }
            }
            ("MOD", "TGSRT") => {
                if let Some(tr) = deserialize_params::<TgSrt>(&params).ok() {
                    tgsrt_tbl.insert(tr.tg, tr.src);
                }
            }
            ("MOD", "CNACLD") => {
                if let Some(cna) = deserialize_params::<CnacldM>(&params).ok() {
                    cnacld_spchg.insert((cna.lp, cna.pfx), cna.spchg);
                }
            }
            ("ADD", "CLRDSN") => {
                let _ = deserialize_params::<Clrdsn>(&params).ok();
            }
            ("ADD", "TGAP") => {
                if let Some(tgap) = deserialize_params::<Tgap>(&params).ok() {
                    tgap_tbl.entry(tgap.tg).or_insert(tgap);
                }
            }
            _ => {}
        }
    }

    let siptg_tbl: HashMap<u32, Siptg> = agcf.siptg.iter().map(|s| (s.tg, s.clone())).collect();
    for tg in pratg_list {
        let src = match tgsrt_tbl.get(&tg.tg).copied() {
            Some(src) => src,
            None => {
                println!("\"{}\" tg={} No TGSRT data", tg.tgn, tg.tg);
                continue;
            }
        };
        let sipt = match siptg_tbl.get(&tg.tg) {
            Some(s) => s,
            None => {
                println!("\"{}\" tg={} No SIPTG data", tg.tgn, tg.tg);
                continue;
            }
        };
        let rt = sipt.srt;
        let o = match srt_tbl.get(&src).copied() {
            Some(o) => o,
            None => {
                println!("\"{}\" src={}", tg.tgn, src);
                continue;
            }
        };
        let rt = match rt_tbl.get(&rt).copied() {
            Some(rt) => rt,
            None => {
                println!("\"{}\" srt={}", tg.tgn, rt);
                continue;
            }
        };
        let rsc = match rtana_tbl.get(&rt).map(|x| x.0) {
            Some(rsc) => rsc,
            None => {
                println!("\"{}\" rt={}", tg.tgn, rt);
                continue;
            }
        };
        let (mn, sc, tid) = match pratkc_tbl.get(&tg.tg) {
            Some(d) => *d,
            None => {
                println!("\"{}\" tg={} No PRATKC data", tg.tgn, tg.tg);
                continue;
            }
        };
        let dsg = tgdsg_tbl.get(&tg.tg).copied();
        let lnk = match pralnk_list.get(&tg.link) {
            Some(lnk) => lnk,
            None => {
                println!("\"{}\" tg={} No PRALNK data", tg.tgn, tg.link);
                continue;
            }
        };

        let (cdfp, area_code) = match tgap_tbl.remove(&tg.tg) {
            Some(tgap) => (Some(tgap.cdfp), tgap.name),
            None => (None, None),
        };

        let mut diff = BTreeMap::new();
        if let Some(icr) = tg.icr.clone() {
            diff.insert("ICR".into(), icr);
        }
        if let Some(ocr) = tg.ocr.clone() {
            diff.insert("OCR".into(), ocr);
        }

        agcf.pratg.push(PraUser {
            tg: tg.tg,
            tgn: tg.tgn,
            mgw: tg.mgw,
            route: (rsc, rt, src),
            lnk: tg.link,
            icr: tg.icr,
            ocr: tg.ocr.unwrap_or_default(),
            cdef: tg.cdef,
            diff,
            mn,
            tkc_sc: sc,
            tkc_tid: tid,
            lnk_scn: lnk.scn,
            lks: lnk.lks,
            binifid: lnk.binifid,
            ofc: o,
            dsg,
            maxi: tg.maxi,
            mtg: tg.mtg,
            area_code,
            cdfp,
        });
    }
    for cna in agcf.cnacld.iter_mut() {
        if let Some(spchp) = cnacld_spchg.get(&(cna.lp, cna.pfx.clone())) {
            cna.spchg = Some(spchp.clone());
        }
    }

    println!("modules : {}", agcf.modules.len());
    println!("mgws: {}\n\t{:?}\n", agcf.mgw.len(), agcf.mgw.first());
    println!("pratg: {}\n\t{:?}\n", agcf.pratg.len(), agcf.pratg.first());
    println!("default icr = \"N/A(derive version)\"");
    println!("pratg: {}\n\t{:?}\n", agcf.pratg.len(), agcf.pratg.last());
    println!("siptg: {}\n\t{:?}\n", agcf.siptg.len(), agcf.siptg.first());
    println!(
        "cnacld: {}\n\t{:?}\n",
        agcf.cnacld.len(),
        agcf.cnacld.last()
    );
    println!(
        "esl: {}  v5: {}  pra: {}  sip: {}\n",
        agcf.asbr_esl.len(),
        agcf.asbr_v5.len(),
        agcf.asbr_pra.len(),
        agcf.asbr_sip.len()
    );
    println!("{:?}\n", agcf.asbr_esl.first());
    println!("{:?}\n", agcf.asbr_sip.first());
    println!("{:?}\n", agcf.asbr_pra.first());
    println!("{:?}\n", agcf.asbr_v5.first());

    Ok(())
}
