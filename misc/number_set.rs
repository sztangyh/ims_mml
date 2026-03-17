extern crate regex;
use regex::Regex;
use std::fmt;
use std::str;
use std::cmp::Ordering;

#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord)]
pub struct NumStem(u64);


const STUB_MAX_LEN:u8  = 15;
const STUB_MAX_BITS:u64 = STUB_MAX_LEN as u64 * 4;
#[allow(dead_code)]
#[inline]
fn min<T>(a:T,b:T)->T where T:PartialOrd{ if a>b{b}else{a}}
#[allow(dead_code)]  
#[inline]
fn max<T>(a:T,b:T)->T where T:PartialOrd{ if a>b{a}else{b}}

impl NumStem {
    pub fn new(num:&str) -> Self{
        let mut n = 0u64;
        let mut len = 0;
        for b in num.bytes(){
            let d = (b - b'0') & 0xF;
            // println!("{} <- {}",len, d);
            n |= (d as u64) << (STUB_MAX_BITS - 4*len);
            len += 1;
        }
        n |= len &0xF;
        NumStem(n)
    }

    #[allow(dead_code)]
    #[inline]
    pub fn len(&self) ->u8{
        (self.0 & 0xF) as u8
    }
    
    #[allow(dead_code)]
    pub fn set_len(&mut self, len:u8){
        // if len==0 {panic!("set len=0 to NumStem")}
        // 原是在len比原先大时，才将新加入的bit清0，len比原来小时则直接设置STUB_MAX_LEN位置为新值
        // 但发现在比较两个NumStem时，会出现同一个字冠，其u64值不同，而导致比较结果错误的问题
        // 所以改为只有len改变，则所有无用bit的值皆清0
        let l = min( self.len(), len) as u64;
        if l > 0 {
            self.0 >>= STUB_MAX_BITS + 4 - 4*l as u64;
            self.0 <<= STUB_MAX_BITS + 4 - 4*l as u64;    
        }else {
            self.0 = 0;
            // 因为 NumStemRange iter时需要用len==0来标志遍历结束，简单起见，这里允许设置len为0，
            // 但对于一般的NumStem而言, len为0是无意义和非法的，会在一些情况如cmp_stup()时产生panic
            // panic!("Setting length of <NumStem> to '0' is not allowing");
        }
        self.set_at(STUB_MAX_LEN, len as u8);
    }

    #[allow(dead_code)]
    #[inline]
    pub fn copy_set_len(&self, len:u8) -> Self{
        let mut n2 = *self;
        n2.set_len(len);
        n2
    }
    #[allow(dead_code)]
    #[inline]
    pub fn expand(&self, e:u8) -> Self{
        self.copy_set_len(self.len() + e)
    }

    #[allow(dead_code)]
    #[inline]
    pub fn get_at(&self, pos:u8) -> u8 {
        let shift = STUB_MAX_BITS - 4*pos as u64;
        ((self.0 >> shift ) & 0xF) as u8
    }
    #[allow(dead_code)]
    #[inline]
    pub fn get_tail(&self) -> u8 {
        let l = self.len();
        if l > 0 {
            self.get_at(l-1)
        }else{
            255
        }
    }

    #[allow(dead_code)]
    #[inline]
    pub fn set_at(&mut self, pos:u8, d:u8){
        let shift = STUB_MAX_BITS - 4*pos as u64;
        let m:u64 = !(0xF << shift );
        self.0 &= m;
        self.0 |= (d as u64) << shift;
    }

    #[allow(dead_code)]
    #[inline]
    pub fn cmp_snb(&self, n2:&NumStem) -> Ordering{
        let l = min(self.len(), n2.len()) as u64;
        //二者在前部分[..l]相同时，如果n2.len()>=self.len()则返回Equal，否则返回Less --(例:self="pqrxxxx",n2="pqr"时)
        let a = ( self.0 >> (STUB_MAX_BITS + 4 - 4*l) , n2.len() >= self.len() );
        let b = ( n2.0 >> (STUB_MAX_BITS + 4 - 4*l) , true );
        a.cmp(&b)
    }

    #[allow(dead_code)]
    #[inline]
    pub fn includes(&self, n2:&NumStem) -> bool{
        let (l1, l2) = (self.len() as u64, n2.len() as u64);
        let shift = STUB_MAX_BITS + 4 - 4*l1;
        if (l1>0) && (l2 >= l1) && (self.0 >> shift)==(n2.0 >> shift){
            true
        }else{
            false
        }
    }

    #[allow(dead_code)]
    #[inline]
    pub fn overlaped(&self, n2:&NumStem) -> bool{
        let l = if self.len() < n2.len() {self.len() as u64} else {n2.len() as u64};
        // println!("cmp {} {}", self, n2);
        if l >0 {
            let (a, b) = (self.0 >> (STUB_MAX_BITS + 4 - 4*l) , n2.0 >> (STUB_MAX_BITS + 4 - 4*l));
            a == b
        }else{
            false
        }
    }

    #[allow(dead_code)]
    #[inline]
    pub fn cmp_stub(&self, n2:&NumStem) -> Ordering{
        let l = min(self.len(), n2.len()) as u64;
        // println!("cmp {} {}", self, n2);
        // l 如果为0 下一行将会panic, 正常情况下不应该存在 len()==0 的 NumStem
        (self.0 >> (STUB_MAX_BITS + 4 - 4*l) ).cmp( &(n2.0 >> (STUB_MAX_BITS + 4 - 4*l)) )
    }

    #[allow(dead_code)]
    #[inline]
    pub fn get_bcd(&self) -> (u64, u8){
        let l = self.len();
        if l==0 {
            (0, 0)
        }else{
            (self.0 >> (STUB_MAX_BITS - 4*(l-1) as u64),l)
        }
    }
    #[allow(dead_code)]
    #[inline]
    pub fn raw(&self) ->u64{
        self.0
    }
}

#[derive(Debug,Clone,PartialEq)]
pub struct NumStemSet(Vec<NumStem>);

impl NumStemSet{
    fn _neat(nums:&mut Vec<NumStem>){
        nums.sort();
        let mut i = 1;
        while i<nums.len(){
            if nums[i-1].includes(&nums[i]){
                nums.remove( i );
            }else{
                i += 1;
            }
        }
        i = 0;
        while i < nums.len(){
            // println!("{}",i);
            // println!("{}[{}] : {}", i,nums.len(), &nums[i]);
            if nums[i].get_tail() == 0 {
                // println!("(:: {}) chk-{} LEN:{}", i, nums[i], nums.len());
                Self::_check_and_pack(nums, i);
            }
            i += 1;
        }
    }


    fn _check_and_pack(nums: &mut Vec<NumStem>, idx:usize) {
        if nums[idx].len()<2 || (idx+9 >=nums.len()) { return ;}
        let cmp = nums[idx].get_bcd();
        for i in 1..=9{
            let idx2 = idx + i as usize;
            if idx2 > nums.len() {return;}
            let cmp2 = nums[idx2].get_bcd();
            if cmp2.1 == cmp.1 {
                if  cmp.0 + i  == cmp2.0 { continue;}
            }else if cmp2.1 > cmp.1 {
                // println!("\t>> {}[{}] / {}[{}]",nums[idx],cmp.1, nums[idx2],cmp2.1);
                if (cmp.0 + i ) << ( 4*( cmp2.1 - cmp.1 ) ) == cmp2.0 {
                    // println!("\t>> Yes!");
                    Self::_check_and_pack(nums, idx2);
                    if nums[idx2].len() == cmp.1 {continue;}
                    // continue;
                }
            }
            return;
        }
        nums.drain( (idx+1)..(idx+10) );
        nums[idx].set_len( cmp.1 - 1 );
        if (cmp.1-1) >1 && ( (cmp.0 >> 4) & 0xf ) == 0 {
            return Self::_check_and_pack(nums, idx);
        }
    }

    fn _from_str(s:& str) -> Vec<NumStem>{
        let r1 = Regex::new(r"\d+").unwrap();
        let mut nums = r1.find_iter(s).map(|m|(NumStem::new(&s[m.start()..m.end()]))).collect::<Vec<_>>();
        Self::_neat(&mut nums);
        nums
    }
    pub fn new( s:&str) -> Self{
        let nums = Self::_from_str(s);
        NumStemSet(nums)
    }

    pub fn add_by(&mut self, nn:&Self){        
        self.0.append(&mut nn.0.clone());
        Self::_neat(&mut self.0);
    }

    pub fn add(&self, nn:&Self) ->Self{
        let mut sum = self.clone();
        sum.add_by(nn);
        sum
    }

    pub fn delete_one(&mut self, n:NumStem){
        while let Ok(idx) = self.0.binary_search_by(|n2|{
            // if n2.overlaped(&n) {Ordering::Equal}
            // else if n2.0 > n.0 {Ordering::Greater}
            // else {Ordering::Less}
            n2.cmp_stub(&n)
        }){
            let l1 = self.0[idx].len();
            // println!("find {}", self.0[idx]);
            if l1 > n.len(){
                self.0.remove(idx);
            }else if l1== n.len(){
                self.0.remove(idx);
                break;
            }else{
                let iter = NumStemRange::new( n, l1, );
                // println!("{}/{} {:?}", n, self.0[idx], &iter);
                self.0.splice(idx..(idx+1), iter);
                break;
            }
        }
    }
    pub fn sub_by(&mut self, nn:&Self){
        for n in &nn.0{
            self.delete_one(*n);
        }
    }
    pub fn sub(&self, nn:&Self) ->Self{
        let mut nn2 = self.clone();
        nn2.sub_by(nn);
        nn2
    }

    pub fn get_stems(&self) -> &[NumStem]{
        &self.0
    }

    fn _intersect(nn1:&[NumStem], nn2:&[NumStem] ) -> Vec<NumStem>{
        let mut rr:Vec<NumStem> = Vec::with_capacity( max(nn1.len(),nn2.len()) );
        let (mut iter_a, mut iter_b) = (nn1.iter(), nn2.iter());
        let (mut a, mut b) = (iter_a.next(), iter_b.next());
        'outer: while let Some(n1) = a{
            'inner: while let Some(n2) = b{
                match n1.cmp_stub(&n2){
                    Ordering::Less => { a=iter_a.next(); continue 'outer;},
                    Ordering::Greater => { b = iter_b.next(); continue 'inner;},
                    Ordering::Equal =>{
                        if n1.len() == n2.len() {
                            rr.push(n1.clone());
                            a = iter_a.next();
                            b = iter_b.next();
                            continue 'outer;
                        }else if n1.len() < n2.len(){
                            rr.push(n2.clone());
                            b = iter_b.next();
                            continue 'inner;
                        }else{
                            rr.push(n1.clone());
                            a = iter_a.next();
                            continue 'outer;
                        }
                    },
                }
            }
            break;
        }
        rr
    }

    #[allow(dead_code)]
    #[inline]
    pub fn intersect(&self, nn:&Self) ->NumStemSet {
        let rr = Self::_intersect(&self.0, &nn.0);
        NumStemSet(rr)
    }

    #[allow(dead_code)]
    pub fn intersect_str(self, s:&str) -> NumStemSet{
        let nn2 = Self::_from_str(s);
        let rr = Self::_intersect(&self.0, &nn2);
        NumStemSet(rr)
    }

    #[allow(dead_code)]
    pub fn has_str_snb(&self, n:&str) -> bool{
        let n = &n[..min(15,n.len())];
        let snb = NumStem::new(n);
        if let Ok(idx) = self.0.binary_search_by(|n2|n2.cmp_stub(&snb)) {
            // cmp_stub()找到的只是存在相互覆盖的字冠，返回字冠的长度如果比snb长，则不包含snb，
            //这时按理数组中也不应存在字根相同而长度比当前返回字冠更短的其它字冠，所以不需要再binary search，立即返回false
            // 之后又添加了cmp_snb(),其对查找对象的长度做了判断，只在snb长度比字冠长的时候，返回Ordering::Equal
            snb.len() >= self.0[idx].len() 
        }else{
            false
        }
    }

    #[allow(dead_code)]
    #[inline]
    pub fn has_snb(&self, num:u64) -> bool{
        self.has_str_snb(format!("{}",num).as_str())
    }

    #[allow(dead_code)]
    pub fn iter_snbs(&self, snb_len:u8, filter:&str) -> NumStemSnbs{
        NumStemSnbs::new(self, snb_len, filter)
    }
}

impl fmt::Display for NumStemSet{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result{
        // write!(f, "(字冠: {})", self.0.iter().map(|n|n.to_string()).collect::<Vec<_>>().join(" "))
        let mut iter = self.0.iter();
        if let Some(n) = iter.next(){
            write!(f, "{}", n.to_string())?;
        }
        while let Some(n) = iter.next() {
            write!(f," {}", n.to_string())?;
        }
        write!(f,"")
    }
}
#[derive(Debug)]
pub struct NumStemSnbs{
    stems    :NumStemSet,
    snb_len :u8,
    idx :usize,
    start :u64,
    end :u64,
    cur :u64,
}
impl NumStemSnbs{
    fn stem2u64(d:NumStem) -> u64{
        let mut u:u64 = 0;
        for i in 0..d.len() {u = u*10 + d.get_at(i) as u64;}
        u
    }

    #[allow(dead_code)]
    pub fn new(nums:&NumStemSet, snb_len:u8, filter:&str) -> Self{
        let n2  = NumStemSet::new(filter);
        let stems = if n2.0.len()==0{nums.clone()}else{nums.intersect(&n2)};
        let r = if snb_len > stems.0[0].len() { (10u64).pow( (snb_len - stems.0[0].len() )as u32) }else {1};
        let start:u64 = if stems.0[0].len()>0{ Self::stem2u64(stems.0[0]) * r }else { 0 };
        NumStemSnbs{
            stems,
            snb_len,
            idx : 0,
            start,
            end :start + r -1,
            cur : start,
        }
    }
    pub fn go_next(&mut self){
        if self.cur < self.end {
            self.cur += 1;
        }else {
            self.idx += 1;
            if self.idx < self.stems.0.len() {
                let r = if self.snb_len > self.stems.0[self.idx].len() {
                    (10u64).pow( (self.snb_len - self.stems.0[self.idx].len() )as u32)
                }else { 1 };
                self.start = Self::stem2u64(self.stems.0[self.idx]) * r ;
                self.end = self.start + r - 1;
                self.cur = self.start;                   
            }
        }
    }
    pub fn get_size(&self) ->(usize, Option<usize>){
        let mut size:usize = (self.end - self.start) as usize;
        for i in (self.idx+1)..self.stems.0.len(){
            let l = self.stems.0[i].len();
            if l > self.snb_len {
                size += 10usize.pow((l - self.snb_len)as u32);
            }else{
                size += 1;
            }
        }
        (size, Some(size))
    }
}
impl Iterator for NumStemSnbs{
    type Item = u64;
    fn next(&mut self) ->Option<u64>{
        if self.idx >= self.stems.0.len(){
            return None;
        }
        let snb = self.cur;
        self.go_next();
        Some(snb)
    }

    fn size_hint(&self) -> (usize, Option<usize>){
        self.get_size()
    }
}

#[derive(Debug)]
pub struct NumStemRange{
    num     :NumStem,
    iter    :NumStem,
    base_len:u8,
}

impl NumStemRange {
    pub fn new(num:NumStem, base_len:u8) ->Self{
        let mut iter = num;
        //如果num=234000 start=3 按字冠234来遍历 则iter第一个值应该是234001 而不是2340 以下确保如此
        let (mut l1, l2) = (base_len, num.len() );
        if l1 >= l2 {
            iter.set_at(STUB_MAX_LEN, 0);
            return NumStemRange{num, iter, base_len};
        }
        iter.set_len(l1); //base_len 之后的位置0， 后继用iter.set_at(STUB_MAX_LEN,len)来置位实际长度
        while l1 < l2 && 0 == num.get_at(l1) {
            l1 += 1;
        }
        if l1 < num.len(){
            iter.set_at(l1, 0);
            iter.set_at(STUB_MAX_LEN, l1+1);
        }else{
            iter.set_at( l2-1, 1);
            iter.set_at(STUB_MAX_LEN, l2);
        }
        // println!("new() num={}, iter={}",num, iter);
        NumStemRange{
            num,
            iter,
            base_len,
        } 
    }
    #[inline]
    pub fn iter_next(&mut self) -> u8{
        let mut l = self.iter.len();
        // let mut d:u8 = 0;
        while l > 0{
            let d = self.iter.get_at(l-1);
            if d < 9 {
                self.iter.set_at(l-1, d+1);
                break;
            }
            self.iter.set_at(l-1, 0);
            l -= 1;
        }
        self.iter.set_at(STUB_MAX_LEN, l); //遍历结束时l会小于base_len，如果base_len为1，l会为0
        l
    }    

}
impl Iterator for NumStemRange{
    type Item = NumStem;
    fn next(&mut self) -> Option<NumStem>{
        if self.iter.len() <= self.base_len { return None;}
        let iter = self.iter;
        let mut len = self.iter_next(); 
        // println!("{} len={:?}", self.iter, len);
        while len >= self.base_len && self.iter.includes(&self.num){
            if len == self.num.len(){
                self.iter_next();
                break;
            }
            len += 1;
            self.iter = self.iter.expand(1);
        }
        Some(iter)
    }
    fn size_hint(&self) -> (usize, Option<usize>){
        let mut yielded:usize = 0;
        let (l1, l2) = (self.iter.len(), self.num.len());
        if l1 <= self.base_len {return (0, Some(0));}
        let tail_d = self.iter.get_at(l1-1);
        for i in self.base_len..(l1-1){
            yielded += self.iter.get_at( i ) as usize;
        }
        yielded += 
            if tail_d > self.num.get_at(l1-1){
                (9*(l2-l1) + tail_d - 1)as usize
            }else{
                ( tail_d )as usize
            };
        let bound = 9 * (l2 - self.base_len)as usize;
        (bound-yielded, Some(bound - yielded))
    }
}
impl fmt::Display for NumStem{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result{
        // let l = self.len();
        // let mut num = String::with_capacity(l as usize);
        // for i in 0..l {
        //     num.push((b'0' + self.get_at(i)) as char)
        // }
        // write!(f, "{}", num)
        let bcd = self.get_bcd();
        write!(f,"D`{:0width$x}", bcd.0, width = bcd.1 as usize)
    }
}
impl fmt::Debug for NumStem{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result{
        // let l = self.len();
        // let mut num = String::with_capacity(l as usize);
        // for i in 0..l {
        //     num.push((b'0' + self.get_at(i)) as char)
        // }
        // write!(f, "D'{}", num)
        let bcd = self.get_bcd();
        write!(f,"D`{:0width$x}", bcd.0, width = bcd.1 as usize)
    }
}

impl  str::FromStr for NumStemSet{
    type Err = &'static str;
    fn from_str(s: &str)->Result<Self, Self::Err>{
        let r1 = Regex::new(r"[\s,;，；]+").unwrap();
        let mut nums:Vec<NumStem> = 
            r1.split(s).filter_map(|ss|
                if ss=="" || ss.contains(|c:char|!c.is_digit(10)){
                    None
                }else{
                    Some(NumStem::new(ss))
                }
            )
            .collect();
        NumStemSet::_neat(&mut nums);
        Ok(NumStemSet(nums))
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
        let np1 = ss1.parse::<NumStemSet>().unwrap();
        println!("parse test1: \"{}\" => {}",ss1, np1);
        assert!(np1==NumStemSet::new("234"));

        let ss2 = r"0,1,2,4,31,39,2,5,6,8,9,7,3";
        let np2 = ss2.parse::<NumStemSet>().unwrap();
        println!("parse test2: \"{}\" => {}",ss2, np2);
        assert!(np2==NumStemSet::new("0,1,3,4,6,7,8,9,2,5,0"));

        let ss2 = r"2001;2002;2003;2004;2005;2006;2007;2008;2009
        ;201;202;203; 204;205;206;207;208;209;
        21;22;23;24  ;25; 26，27;28;290
        ;291;292;293;294;295；296;297;298;   2990;2991;2992;2993;
        2994;2995;2996;2997;2998,2000，2999;";
        let np2 = ss2.parse::<NumStemSet>().unwrap();
        println!("parse test3: \"{}\" => {}",ss2, np2);
        assert!(np2==NumStemSet::new("20，2999；2"));

        let ss2 = r"112;113;115;1100;;1108;1109;1102;1103;1104;1105;1106;
        1107;111;116;117;118;1101";
        let np2 = ss2.parse::<NumStemSet>().unwrap();
        println!("parse test4: \"{}\" => {}",ss2, np2);
        assert!(np2==NumStemSet::new("110;111;112;113;115;116;117;118"));

        let ss2 = r"20;21;22;23;24;25;26;27;28;41;42;
        808;809;81;82;83;84;850;851;852;853;854;8550;8551;8552;
        ;8006;8007;8008;8009;801;802;803;804;805;806;807;
        8553;8554;8556;8557;8558;8559;
        43;44;45;46;47;48;49;8001;8002;8003;8004;8005
        893;894;895;896;897;898;8990;8991;8992;8993;8994;
        856;857;858;859;86;87;88;890;891;892;
        8995;8996;8997;8998,8000,8555,8999";
        let np2 = ss2.parse::<NumStemSet>().unwrap();
        println!("parse test5: \"{}\" => {}",ss2, np2);
        assert!(np2==NumStemSet::new("20;21;22;23;24;25;26;27;28;41;42;43;44;45;46;47;48;49;8"));

        let ss2 = r"21;20;22;23;24;25;26;27;28
        ;2900;311;3225,3220,3221,3223,3222,3224,3225,3226,3227,3228,2902,3229,;
        ;2909,2901,2902,2903,2904,2905,2906,2907,2908,555,2909,292,291,293,294,295,296,297,299,298";
        let np2 = ss2.parse::<NumStemSet>().unwrap();
        println!("parse test6: \"{}\" => {}",ss2, np2);
        assert!(np2==NumStemSet::new("20，2999；2,311,322,555"));
    }
    #[test]
    fn test_range_iter(){
        let rg1 = NumStemRange::new(NumStem::new("100"), 1);
        assert_eq!(rg1.collect::<Vec<_>>(),NumStemSet::new("101,102,103,104,105,106,107,108,109,11,12,13,14,15,16,17,18,19").0);
        let rg1 = NumStemRange::new(NumStem::new("199"), 1);
        assert_eq!(rg1.collect::<Vec<_>>(),NumStemSet::new("10,11,12,13,14,15,16,17,18,190,191,192,193,194,195,196,197,198").0);
        let rg1 = NumStemRange::new(NumStem::new("0"), 0);
        assert_eq!(rg1.collect::<Vec<_>>(),NumStemSet::new("1,2,3,4,5,6,7,8,9").0);
        let rg1 = NumStemRange::new(NumStem::new("10900"), 1);
        assert_eq!(rg1.collect::<Vec<_>>(),NumStemSet::new(r"100,101,102,103,104,105,106,107,108,10901,10902,10903,10904,10905,10906,
        10907,10908,10909,1091,1092,1093,1094,1095,1096,1097,1098,1099,11,12,13,14,15,16,17,18,19").0);
    }
    #[test]
    fn test_add_sub(){
        assert_eq!((NumStemSet::new("234")).sub(&NumStemSet::new("2340")), NumStemSet::new("2341,2342,2343,2344,2345,2346,2347,2348,2349"));
        assert_eq!((NumStemSet::new("0234,99")).add(&NumStemSet::new("9901,56,023")), NumStemSet::new("023,56,99"));
        assert_eq!((NumStemSet::new("1")).sub(&NumStemSet::new("199,10,15").add(&NumStemSet::new("19"))), NumStemSet::new("11,12,13,14,16,17,18"));
        fn ns(s:&str)->NumStemSet{NumStemSet::new(s)}
        assert_eq!(ns("2,3,4008").sub(&ns("200, 4")), ns("2").sub(&ns("200")).add(&ns("3")) );
        assert_eq!(ns("999").sub(&ns("999999")).add(&ns("999999")), ns("999"));
        assert_eq!(ns("1,5,9").sub(&ns("10000,51234,9090,4008")).add(&ns("10000,9090,51234")), ns("1,5,9"));
        assert_eq!(ns("2").sub(&ns("2123456")).add(&ns("212345634")).add(&ns("212")).add(&ns("210,211,213,214,215,216,217,218,219")),ns("2"));
    }

    #[test]
    fn test_intersecut(){
        fn ns(s:&str)->NumStemSet{NumStemSet::new(s)}
        fn ic(s1:&str, s2:&str) ->NumStemSet{ ns(s1).intersect(&ns(s2))}
        assert_eq!(ic("123,5,2","1,200"), ns("123,200"));
        assert_eq!(ic("0123456,9870","98700,01234"), ns("0123456,98700"));
        assert_eq!(ns("23").sub(&ns("23000")).intersect(&ns("23000")), ns(""));
        assert_eq!(ns("23,999000,000999,90009,09990").sub(&ns("234567890")).intersect(&ns("234567890,9,0")), ns("999000,000999,90009,09990"));
    }

}

