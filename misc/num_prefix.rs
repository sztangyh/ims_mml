extern crate regex;
use std::str;
use regex::Regex;
use std::fmt;
use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq)]
pub struct NumPrefix{
    nums:Vec<String>,
}

#[derive(Debug, PartialEq)]
pub struct NumPrefixRef<'a>(Vec<&'a str>);

#[derive(Debug)]
pub struct NumRange<'a>{
    num1:&'a str,
    num2:&'a str,
    count:u32,
}

#[derive(Debug)]
pub struct NumRange2<'a>{
    num : &'a str,
    base_len : usize,
    idx : usize,
    dgt : u8,
}

impl <'a> NumRange2<'a>{
    pub fn new(num:&'a str, base_len:usize) ->Self{
        NumRange2{
            num,
            base_len,
            idx:base_len,
            dgt:b'0' - 1,
        }
    }
}
impl <'a> Iterator for NumRange2<'a>{
    type Item = String;
    fn next(&mut self) -> Option<String>{
        // if self.idx < self.base_len {
        //     return None;
        // }
        if self.idx == self.base_len && self.dgt == b'9'{
            return None;
        }
        self.dgt += 1;
        while self.dgt == self.num.as_bytes()[self.idx]{
            self.dgt = b'0';
            self.idx += 1;
            if self.idx >= self.num.len() {
                self.idx = self.num.len() -1;
                self.dgt = self.num.as_bytes()[self.idx] + 1;
                break;
            }
        }
        while self.dgt == (b'9' + 1) && self.idx > self.base_len{
            self.idx -= 1;
            self.dgt = self.num.as_bytes()[self.idx] + 1;
        }
        if self.dgt > b'9'{
            None
        }else{
            let s = format!("{}{}",&self.num[..self.idx], self.dgt as char);
            Some(s)    
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>){
        let mut yielded:usize = 0;
        for b in (&self.num[self.base_len..self.idx]).as_bytes(){
            yielded += (*b - b'0') as usize;
        }
        yielded += 
            if self.dgt > self.num.as_bytes()[self.idx]{
                9*(self.num.len()-self.idx-1) + (self.dgt - b'0' )as usize
            }else{
                (self.dgt + 1 - b'0')as usize
            };
        let bound = 9 * (self.num.len() - self.base_len)as usize;
        (bound-yielded, Some(bound - yielded))
    }
}
impl <'a> NumRange<'a>{
    pub fn new(s:&'a str, base_len:usize) -> Self{
        NumRange{
            num1:s,
            num2:&s[base_len..],
            count:0,
        }
    }
}
impl<'a> Iterator for NumRange<'a>{
    type Item = String;
    fn next(&mut self) -> Option<Self::Item>{
        if (self.count as usize) < 9*self.num2.len(){
            let mut c = self.count;
            self.count += 1;
            for (p, b) in self.num2.bytes().enumerate(){
                if c < (b - b'0')as u32{
                    let l = self.num1.len() - self.num2.len()  + p as usize;
                    return Some(format!("{}{}",&self.num1[..l],c));
                }
                c -= (b - b'0')as u32;
            }
            for (p, b) in self.num2.bytes().rev().enumerate(){
                if c < (b'9' - b  )as u32{
                    let l = self.num1.len() - p - 1 as usize;
                    return Some(format!("{}{}",&self.num1[..l], c as u8 + (b-b'0')+ 1));
                }
                c -= (b'9' - b  )as u32;
            }
        }
        return None;
    }
    fn size_hint(&self) -> (usize, Option<usize>){
        let bound = 9*self.num2.len();
        let size = bound - self.count as usize;
        (size, Some(size) )
    }
}
impl  NumPrefix{
    pub fn new(s:&str) ->Self{
        let r1 = Regex::new(r"\d+").unwrap();
        let mut nums = r1.find_iter(s).map(|m|(&s[m.start()..m.end()]).to_owned()).collect::<Vec<_>>();
        Self::_neat(&mut nums);
        NumPrefix{ nums}
    }
    pub fn add_by(&mut self, nn:&Self){        
        self.nums.append(&mut nn.nums.clone());
        Self::_neat(&mut self.nums);
    }
    pub fn add(&self, nn:&Self) ->Self{
        let mut sum = self.clone();
        sum.add_by(nn);
        sum
    }
    pub fn delete_one(&mut self, n:&str){
        while let Ok(idx) = self.nums.binary_search_by(|s|{
            if s.len()<= n.len(){
                (**s).cmp( &n[..s.len()] )
            }else{
                ( &s[..n.len()] ).cmp(n)
            }
        }){
            let l1 = self.nums[idx].len();
            if l1 > n.len(){
                self.nums.remove(idx);
            }else if l1== n.len(){
                self.nums.remove(idx);
                break;
            }else{
                let iter = NumRange2::new( n, self.nums[idx].len(), );
                self.nums.splice(idx..(idx+1), iter);
                break;
            }
        }
    }
    pub fn sub_by(&mut self, nn:&Self){
        for n in &nn.nums{
            self.delete_one(&**n);
        }
    }
    pub fn sub(&self, nn:&Self) ->Self{
        let mut nn2 = self.clone();
        nn2.sub_by(nn);
        nn2
    }
    fn cmp_stub(n1:&str, n2:&str) -> Ordering{
        if n1.len()<= n2.len(){
            n1.cmp( &n2[..n1.len()] )
        }else{
            ( &n1[..n2.len()] ).cmp(n2)
        }
    }
    pub fn intersect<'a>(&'a self, nn:&'a Self) ->NumPrefixRef<'a>{
        let mut rr:Vec<&'a str> = Vec::with_capacity(self.nums.len()+nn.nums.len());
        let (mut iter_a, mut iter_b) = (self.nums.iter(), nn.nums.iter());
        let (mut a, mut b) = (iter_a.next(), iter_b.next());
        'outer: while let Some(n1) = a{
            'inner: while let Some(n2) = b{
                match Self::cmp_stub(n1, n2){
                    Ordering::Less => { a=iter_a.next(); continue 'outer;},
                    Ordering::Greater => { b = iter_b.next(); continue 'inner;},
                    Ordering::Equal =>{
                        if n1.len() == n2.len() {
                            rr.push(&*n1);
                            a = iter_a.next();
                            b = iter_b.next();
                            continue 'outer;
                        }else if n1.len() < n2.len(){
                            rr.push(&*n2);
                            b = iter_b.next();
                            continue 'inner;
                        }else{
                            rr.push(&*n1);
                            a = iter_a.next();
                            continue 'outer;
                        }
                    },
                }
            }
            break;
        }
        NumPrefixRef(rr)
    }
    pub fn includes(&self, n:&str) -> bool{
        if let Ok(_) = self.nums.binary_search_by(|n2|{
            if n2.len() <= n.len() {
                (**n2).cmp(&n[..n2.len()])
            }else{
                (**n2).cmp(n)
            }
        }) {
            true
        }else{
            false
        }
    }
    #[inline]
    pub fn has_snb(&self, num:u64) -> bool{
        self.includes(format!("{}",num).as_str())
    }
    pub fn num_iter(&self) -> std::slice::Iter<String>{
        self.nums.iter()
    }
    fn _check_and_pack(nums: &mut Vec<String>, idx:usize) {
        if nums[idx].len()<2 || (idx+9 >=nums.len()) { return ;}
        let len = nums[idx].len();
        for i in 1..=9{
            if idx + i > nums.len() {return;}
            let cmp = b'0' + i as u8;
            let by_ten = & nums[idx][..len-1];
            let num = & nums[idx + i];
            if  idx + i >= nums.len() ||
                num.len()<len ||
                // num.len()>len+1 ||
                num.as_bytes()[len-1] != cmp ||
                !num.starts_with(by_ten){ 
                return;
            }
            // if num.len() == len+1{
            if num.len() > len{
                // if num.as_bytes()[len] !=b'0' {return;}
                if num.bytes().skip(len).any(|b| b!=b'0'){return;}
                Self::_check_and_pack(nums, idx+i);
                if nums[idx+i].len() !=len{return;}
            }
        }
        nums.drain((idx+1)..(idx+10));
        nums[idx].pop();
        if nums[idx].len()>1 && nums[idx].as_bytes()[nums[idx].len()-1]==b'0'{
            return Self::_check_and_pack(nums, idx);
        }
    }
    fn _neat(nums:&mut Vec<String>){
        nums.sort();
        let mut i = 1;
        while i<nums.len(){
            if nums[i].starts_with(nums[i-1].as_str()){
                nums.remove( i );
            }else{
                i += 1;
            }
        }
        i = 0;
        while i < nums.len(){
            // println!("{}",i);
            // println!("{}[{}] : {}", i,nums.len(), &nums[i]);
            if nums[i].as_bytes()[nums[i].as_bytes().len()-1]==b'0' {
                Self::_check_and_pack(nums, i);
            }
            i += 1;
        }
    }
}

impl From<&NumPrefixRef<'_>> for NumPrefix{
    fn from(nr:&NumPrefixRef) -> Self{
        let nums = nr.0.iter().map(|n|String::from(*n)).collect();
        NumPrefix{nums}
    }
}

impl str::FromStr for NumPrefix{
    type Err = &'static str;
    fn from_str(s: &str)->Result<Self, Self::Err>{
        let r1 = Regex::new(r"[\s,;，；]+").unwrap();
        let mut nums:Vec<String> = 
            r1.split(s).filter_map(|ss|match ss {
                "" => None,
                _ => Some(ss.to_owned()),
            })
            .collect();
        if nums.iter().any(|n|n.contains(|c:char|!c.is_digit(10)) ){
            return Err("Invalid char found"); 
        };
        // nums.sort();
        // println!("{:?}", &nums);
        NumPrefix::_neat(&mut nums);
        Ok(NumPrefix{
            nums,
        })
    }
}

impl fmt::Display for NumPrefix{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result{
        write!(f, "(字冠: {})", self.nums.join(";"))
    }
}
impl IntoIterator for NumPrefix{
    type Item  = String;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> std::vec::IntoIter<Self::Item>{
        self.nums.into_iter()
    }
}
pub struct NumIterater<'a>{
    nn:&'a NumPrefix,
    cur:usize,
}
impl <'a> Iterator for NumIterater<'a>{
    type Item = &'a str;
    fn next(&mut self) -> Option<&'a str>{
        let idx = self.cur;
        if idx < self.nn.nums.len(){
            self.cur += 1;
            Some(&*self.nn.nums[idx])
        }else{
            None
        }
        // None
    }
}
impl <'a> IntoIterator for &'a NumPrefix{
    type Item = &'a str;
    type IntoIter = NumIterater<'a>;
    fn into_iter(self) ->Self::IntoIter{
        NumIterater{
            nn:self,
            cur:0,
        }
    }

}


#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn parse_from_string(){
        let ss1 = r"23400;23402;23403;23404;23405;23406;23407;
        23408;23409;2341;2342;2343;2344;
        2345;2346;2347;2348;2349,23401";
        let np1 = ss1.parse::<NumPrefix>().unwrap();
        println!("parse test1: \"{}\" => {}",ss1, np1);
        assert!(np1==NumPrefix::new("234"));

        let ss2 = r"0,1,2,4,31,39,2,5,6,8,9,7,3";
        let np2 = ss2.parse::<NumPrefix>().unwrap();
        println!("parse test2: \"{}\" => {}",ss2, np2);
        assert!(np2==NumPrefix::new("0,1,3,4,6,7,8,9,2,5,0"));

        let ss2 = r"2001;2002;2003;2004;2005;2006;2007;2008;2009
        ;201;202;203; 204;205;206;207;208;209;
        21;22;23;24  ;25; 26，27;28;290
        ;291;292;293;294;295；296;297;298;   2990;2991;2992;2993;
        2994;2995;2996;2997;2998,2000，2999;";
        let np2 = ss2.parse::<NumPrefix>().unwrap();
        println!("parse test3: \"{}\" => {}",ss2, np2);
        assert!(np2==NumPrefix::new("20，2999；2"));

        let ss2 = r"112;113;115;1100;;1108;1109;1102;1103;1104;1105;1106;
        1107;111;116;117;118;1101";
        let np2 = ss2.parse::<NumPrefix>().unwrap();
        println!("parse test4: \"{}\" => {}",ss2, np2);
        assert!(np2==NumPrefix::new("110;111;112;113;115;116;117;118"));

        let ss2 = r"20;21;22;23;24;25;26;27;28;41;42;
        808;809;81;82;83;84;850;851;852;853;854;8550;8551;8552;
        ;8006;8007;8008;8009;801;802;803;804;805;806;807;
        8553;8554;8556;8557;8558;8559;
        43;44;45;46;47;48;49;8001;8002;8003;8004;8005
        893;894;895;896;897;898;8990;8991;8992;8993;8994;
        856;857;858;859;86;87;88;890;891;892;
        8995;8996;8997;8998,8000,8555,8999";
        let np2 = ss2.parse::<NumPrefix>().unwrap();
        println!("parse test5: \"{}\" => {}",ss2, np2);
        assert!(np2==NumPrefix::new("20;21;22;23;24;25;26;27;28;41;42;43;44;45;46;47;48;49;8"));

        let ss2 = r"21;20;22;23;24;25;26;27;28
        ;2900;311;3225,3220,3221,3223,3222,3224,3225,3226,3227,3228,2902,3229,;
        ;2909,2901,2902,2903,2904,2905,2906,2907,2908,555,2909,292,291,293,294,295,296,297,299,298";
        let np2 = ss2.parse::<NumPrefix>().unwrap();
        println!("parse test6: \"{}\" => {}",ss2, np2);
        assert!(np2==NumPrefix::new("20，2999；2,311,322,555"));

    }

}
