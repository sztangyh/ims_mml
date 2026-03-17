//use std::path::Path;
use once_cell::sync::OnceCell;
use regex::Regex;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::str::Split;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug, Clone)]
pub struct SpgOut<'a> {
    pub ne: &'a str,
    pub time: &'a str,
    pub mml: &'a str,
    pub code: &'a str,
    pub rlt: &'a str,
    pub output: &'a str,
}

static RE_MML_OUT: OnceCell<Regex> = OnceCell::new();

pub struct SpgOutSplit<'a> {
    blks: Split<'a, &'a str>,
    skip_fail: bool,
}
impl<'a> Iterator for SpgOutSplit<'a> {
    type Item = SpgOut<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let re = RE_MML_OUT.get().unwrap();
        while let Some(blk) = self.blks.next() {
            if let Some(caps) = re.captures(blk) {
                let code = caps.get(4).map_or("", |m| &blk[m.range()]);
                if self.skip_fail && code != "0" {
                    continue;
                }
                return Some(SpgOut {
                    ne: caps.get(1).map_or("", |m| &blk[m.range()]),
                    time: caps.get(2).map_or("", |m| &blk[m.range()]),
                    mml: caps.get(3).map_or("", |m| &blk[m.range()]),
                    code,
                    rlt: caps.get(5).map_or("", |m| &blk[m.range()]),
                    output: caps.get(6).map_or("", |m| &blk[m.range()]),
                });
            }
        }
        None
    }
}
pub fn spgout_iter<'a>(log: &'a str, skip_fail: bool) -> impl Iterator<Item = SpgOut<'a>> {
    RE_MML_OUT.get_or_init(||{
        Regex::new(r#"(?ms)\A(\w+)\s+([^\r\n]+).*^%%([^%]+)%%\s*^RETCODE = (\d+)\s+([^\r\n]*).*^-{9}(.*)^-{9}"#).unwrap()
    });
    SpgOutSplit {
        blks: log.split("+++    "),
        skip_fail,
    }
}
static RE_SPG_OUT: OnceCell<Regex> = OnceCell::new();
pub fn extract_mml_log<'a, T>(log: &'a str, f: fn(out: SpgOut) -> Option<T>) -> Result<Vec<T>> {
    //r#"(?ms)(\w+)\s+(.*?)\n.*\n%%(.*)%%\nRETCODE = (\d+)\s+(.*)\n.*-{9}(.*)\n-{9}.*"#
    let re = RE_SPG_OUT.get().unwrap();
    let it = log.split("+++    ");
    let mut v = vec![];

    for blk in it {
        if let Some(caps) = re.captures(blk) {
            let out = SpgOut {
                ne: &caps[1],
                time: &caps[2],
                mml: &caps[3],
                code: &caps[4],
                rlt: &caps[5],
                output: &caps[6],
            };
            if let Some(t) = f(out) {
                v.push(t);
            }
        } else {
            println!("Fail Capture:\n{}\n", blk);
        }
    }
    Ok(v)
}

fn main() -> Result<()> {
    let logfile =
        std::env::var("MML_T001_LOG").unwrap_or_else(|_| "logfile/lst_ifcs.txt".to_string());
    let mut file = File::open(&logfile)?;
    let mut log = String::new();
    file.read_to_string(&mut log)?;
    //let txt = "Hi Joc  os 123    ::";
    let re = RE_SPG_OUT.get_or_init(||{
        Regex::new(r#"(?ms)\A(\w+)\s+([^\r\n]+).*^%%([^%]+)%%\s*^RETCODE = (\d+)\s+([^\r\n]*).*^-{9}(.*)^-{9}"#).unwrap()
    });
    let test = r#"HSS53        2023-03-16 16:39:43+08:00
O&M    #30
%%LST HSIFC:IMPU="sip:+8675583551871@gd.ctcims.cn";%%
RETCODE = 0  SUCCESS0001:Operation is successful

结果如下：
---------
SIFCID
0
42
---------(结果个数 = 2)
---    END
"#;
    println!("{:#?}", re.captures(test).unwrap());

    let v: Vec<_> = spgout_iter(&log, false).collect();
    for s in v.iter() {
        println!("{:#?}", s.mml);
    }
    Ok(())
}
