use super::RangeOfPrefix;
use super::U4Number;
use std::fmt::Display;
use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Clone, PartialEq, Eq)]
/// Normalized set of packed numeric prefixes.
pub struct U4NumberVec<const NUM_SIZE: usize>(pub Vec<U4Number<NUM_SIZE>>);

impl<const NUM_SIZE: usize> U4NumberVec<NUM_SIZE> {
    /// Parses and normalizes a prefix set from text.
    pub fn new(s: &str) -> Self {
        s.parse().unwrap()
    }
    fn reduce_range(nums: &mut [U4Number<NUM_SIZE>], mut st: usize, ed: usize) -> bool {
        let size = ed - st + 1;
        //let nums = self.0.as_mut_slice();
        //println!("size = {}", size);
        if size >= 10 {
            let mut it = RangeOfPrefix::from_u4(nums[st], nums[ed]);
            if size > it.size_hint().0 {
                //println!("replace [{}..{}] <-({})items of {:?}", st, st+size, it.size_hint().0, it);
                while let Some(nb) = it.next() {
                    nums[st] = nb;
                    st += 1;
                }
                for p in st..=ed {
                    nums[p].set_len(0);
                }
                return true;
            }
        }
        false
    }
    fn rmv_empty(&mut self) {
        let (l, mut s, mut t) = (self.0.len(), 0, 0);
        while s < l {
            if self.0[s].len() == 0 {
                s += 1;
            } else {
                if s > t {
                    self.0[t] = self.0[s];
                }
                s += 1;
                t += 1;
            }
        }
        self.0.truncate(t);
    }

    /// Sorts and normalizes the internal prefix set.

    pub fn get_ready(&mut self) {
        if self.0.len() < 2 {
            return;
        }
        let nums = self.0.as_mut_slice();
        let mut succeeding = nums.first().map(|_| 0);
        let mut last = 0;
        nums.sort();
        //println!("sorted = \"{:?}\"", nums);
        for i in 1..nums.len() {
            //println!("[{}]=\"{}\"   [{}]=\"{}\"", last, nums[last], i, nums[i]);
            if nums[i].same_prefix_len(&nums[last]) == nums[last].len() {
                nums[i].set_len(0);
                continue;
            }
            if succeeding.is_some() && !nums[last].is_succeed_by(&nums[i]) {
                Self::reduce_range(nums, succeeding.take().unwrap(), last);
            }
            last = i;
            if succeeding.is_none() && nums[i].is_end_with(0) {
                succeeding = Some(i);
            }
        }
        if let Some(st) = succeeding {
            Self::reduce_range(nums, st, last);
        }
        self.rmv_empty();
        //self.0 = self.0.iter().filter(|&n|n.len() > 0).map(|n|n.clone()).collect();
    }

    /// Checks whether the set contains `num2` under prefix semantics.

    pub fn include(&self, num2: &U4Number<NUM_SIZE>) -> bool {
        //dbg!(num2);
        self.0.binary_search_by(|num| num.include_cmp(num2)).is_ok()
    }

    /// Deletes `num2` from the set, splitting prefixes when needed.

    pub fn delete(&mut self, num2: &U4Number<NUM_SIZE>) -> bool {
        use super::NumsMatch::SUBSET;
        use super::U4NumberDivided;
        if let Ok(idx) = self
            .0
            .binary_search_by(|num| num.overlap_check(num2).into())
        {
            let l = self.0[idx].len();

            if l > num2.len() {
                let mut a = idx;
                while a > 0 && self.0[a - 1].overlap_check(num2) == SUBSET {
                    a -= 1;
                }
                let mut b = idx + 1;
                while b < self.0.len() && self.0[b].overlap_check(num2) == SUBSET {
                    b += 1;
                }
                self.0.drain(a..b);
            } else if l == num2.len() {
                self.0.remove(idx);
            } else {
                let iter = U4NumberDivided::new(self.0[idx], *num2);
                self.0.splice(idx..(idx + 1), iter);
            }
            return true;
        }
        false
    }

    /// Deletes every prefix from `many`.

    pub fn delete_many(&mut self, many: &Self) {
        for one in many.0.iter() {
            self.delete(&one);
        }
    }

    /// Computes the intersection with another prefix set.

    pub fn intersect(&self, other: &Self) -> Self {
        use super::NumsMatch::*;
        let mut ab = vec![];
        let (mut it1, mut it2) = (self.0.iter(), other.0.iter());
        let (mut a, mut b) = (it1.next(), it2.next());
        'outer: while let Some(n1) = a {
            'inner: while let Some(n2) = b {
                match n1.overlap_check(&n2) {
                    BELOW => {
                        a = it1.next();
                        continue 'outer;
                    }
                    ABOVE => {
                        b = it2.next();
                        continue 'inner;
                    }
                    EQUAL => {
                        ab.push(*n1);
                        a = it1.next();
                        b = it2.next();
                        continue 'outer;
                    }
                    SUPERSET => {
                        ab.push(*n2);
                        b = it2.next();
                        continue 'inner;
                    }
                    SUBSET => {
                        ab.push(*n1);
                        a = it1.next();
                        continue 'outer;
                    }
                }
            }
            break;
        }
        Self(ab)
    }
}

impl<const NUM_SIZE: usize> std::str::FromStr for U4NumberVec<NUM_SIZE> {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let nums: Vec<U4Number<NUM_SIZE>> = s
            .split(&[';', ' ', ',', '\t', '\n'])
            .filter_map(|t| match t.trim() {
                "" => None,
                _ => Some(t),
            })
            .map(|t| t.parse())
            .collect::<Result<_, String>>()?;
        let mut nums = Self(nums);
        nums.get_ready();
        Ok(nums)
    }
}

impl<const NUM_SIZE: usize> Add for U4NumberVec<NUM_SIZE> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let mut o = self.clone();
        o.0.extend(&rhs.0);
        o.get_ready();
        o
    }
}

impl<const NUM_SIZE: usize> AddAssign for U4NumberVec<NUM_SIZE> {
    fn add_assign(&mut self, rhs: Self) {
        self.0.append(rhs.0.clone().as_mut());
        self.get_ready();
    }
}

impl<const NUM_SIZE: usize> Sub for U4NumberVec<NUM_SIZE> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        let mut o = self.clone();
        o.delete_many(&rhs);
        o
    }
}

impl<const NUM_SIZE: usize> SubAssign for U4NumberVec<NUM_SIZE> {
    fn sub_assign(&mut self, rhs: Self) {
        self.delete_many(&rhs)
    }
}
impl<const NUM_SIZE: usize> Display for U4NumberVec<NUM_SIZE> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut it = self.0.iter();
        if let Some(n) = it.next() {
            //f.write_str(&n.to_string())?;
            write!(f, "{}", n)?;
        }
        while let Some(n) = it.next() {
            write!(f, "; {}", n)?;
        }
        Ok(())
    }
}

impl<const NUM_SIZE: usize> std::fmt::Debug for U4NumberVec<NUM_SIZE> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("U4NumberVec:[ ")?;
        let mut it = self.0.iter();
        if let Some(n) = it.next() {
            //f.write_str(&n.to_string())?;
            write!(f, "{}", n)?;
        }
        while let Some(n) = it.next() {
            write!(f, "; {}", n)?;
        }
        f.write_str(" ]")?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    type NumStemSet = U4NumberVec<12>;
    #[test]
    fn parse_from_string() {
        let ss1 = r"23400;23402;23403;23404;23405;23406;23407;
        23408;23409;2341;2342;2343;2344;
        2345;2346;2347;2348;2349,23401";
        let np1 = ss1.parse::<NumStemSet>().unwrap();
        println!("parse test1: \"{}\" => {}", ss1, np1);
        assert!(np1 == NumStemSet::new("234"));

        let ss2 = r"0,1,2,4,31,39,2,5,6,8,9,7,3";
        let np2 = ss2.parse::<NumStemSet>().unwrap();
        println!("parse test2: \"{}\" => {}", ss2, np2);
        assert!(np2 == NumStemSet::new("0,1,3,4,6,7,8,9,2,5,0"));

        let ss2 = r"2001;2002;2003;2004;2005;2006;2007;2008;2009
        ;201;202;203; 204;205;206;207;208;209;
        21;22;23;24  ;25; 26閿?7;28;290
        ;291;292;293;294;295閿?96;297;298;   2990;2991;2992;2993;
        2994;2995;2996;2997;2998,2000閿?999;";
        let np2 = ss2.parse::<NumStemSet>().unwrap();
        println!("parse test3: \"{}\" => {}", ss2, np2);
        assert!(np2 == NumStemSet::new("20閿?999閿?"));

        let ss2 = r"112;113;115;1100;;1108;1109;1102;1103;1104;1105;1106;
        1107;111;116;117;118;1101";
        let np2 = ss2.parse::<NumStemSet>().unwrap();
        println!("parse test4: \"{}\" => {}", ss2, np2);
        assert!(np2 == NumStemSet::new("110;111;112;113;115;116;117;118"));

        let ss2 = r"20;21;22;23;24;25;26;27;28;41;42;
        808;809;81;82;83;84;850;851;852;853;854;8550;8551;8552;
        ;8006;8007;8008;8009;801;802;803;804;805;806;807;
        8553;8554;8556;8557;8558;8559;
        43;44;45;46;47;48;49;8001;8002;8003;8004;8005
        893;894;895;896;897;898;8990;8991;8992;8993;8994;
        856;857;858;859;86;87;88;890;891;892;
        8995;8996;8997;8998,8000,8555,8999";
        let np2 = ss2.parse::<NumStemSet>().unwrap();
        println!("parse test5: \"{}\" => {}", ss2, np2);
        assert!(np2 == NumStemSet::new("20;21;22;23;24;25;26;27;28;41;42;43;44;45;46;47;48;49;8"));

        let ss2 = r"21;20;22;23;24;25;26;27;28
        ;2900;311;3225,3220,3221,3223,3222,3224,3225,3226,3227,3228,2902,3229,;
        ;2909,2901,2902,2903,2904,2905,2906,2907,2908,555,2909,292,291,293,294,295,296,297,299,298";
        let np2 = ss2.parse::<NumStemSet>().unwrap();
        println!("parse test6: \"{}\" => {}", ss2, np2);
        assert!(np2 == NumStemSet::new("20閿?999閿?,311,322,555"));
    }
    #[test]
    fn test_range_iter() {
        type NumStemRange = crate::U4NumberDivided<12>;
        let rg1 = NumStemRange::new("1".into(), 100.into());
        assert_eq!(
            rg1.collect::<Vec<_>>(),
            NumStemSet::new("101,102,103,104,105,106,107,108,109,11,12,13,14,15,16,17,18,19").0
        );
        let rg1 = NumStemRange::new("1".into(), 199.into());
        assert_eq!(
            rg1.collect::<Vec<_>>(),
            NumStemSet::new("10,11,12,13,14,15,16,17,18,190,191,192,193,194,195,196,197,198").0
        );
        let rg1 = NumStemRange::new("".into(), "0".into());
        assert_eq!(
            rg1.collect::<Vec<_>>(),
            NumStemSet::new("1,2,3,4,5,6,7,8,9").0
        );
        let rg1 = NumStemRange::new("1".into(), 10900.into());
        assert_eq!(
            rg1.collect::<Vec<_>>(),
            NumStemSet::new(
                r"100,101,102,103,104,105,106,107,108,10901,10902,10903,10904,10905,10906,
        10907,10908,10909,1091,1092,1093,1094,1095,1096,1097,1098,1099,11,12,13,14,15,16,17,18,19"
            )
            .0
        );
    }
    #[test]
    fn test_add_sub() {
        let mut ss = NumStemSet::new("234");
        ss.delete_many(&NumStemSet::new("2340"));
        assert_eq!(
            ss,
            NumStemSet::new("2341,2342,2343,2344,2345,2346,2347,2348,2349")
        );
        assert_eq!(
            (NumStemSet::new("0234,99") + NumStemSet::new("9901,56,023")),
            NumStemSet::new("023,56,99")
        );
        assert_eq!(
            (NumStemSet::new("1") - (NumStemSet::new("199,10,15") + NumStemSet::new("19"))),
            NumStemSet::new("11,12,13,14,16,17,18")
        );

        fn ns(s: &str) -> NumStemSet {
            NumStemSet::new(s)
        }
        assert_eq!(ns("2,3,4008") - ns("200, 4"), ns("2") - ns("200") + ns("3"));
        assert_eq!(ns("999") - ns("999999") + ns("999999"), ns("999"));
        assert_eq!(
            ns("1,5,9") - ns("10000,51234,9090,4008") + ns("10000,9090,51234"),
            ns("1,5,9")
        );
        assert_eq!(
            ns("2") - ns("2123456")
                + ns("212345634")
                + ns("212")
                + ns("210,211,213,214,215,216,217,218,219"),
            ns("2")
        );
    }

    #[test]
    fn test_intersect() {
        fn ns(s: &str) -> NumStemSet {
            NumStemSet::new(s)
        }
        fn ic(s1: &str, s2: &str) -> NumStemSet {
            ns(s1).intersect(&ns(s2))
        }
        assert_eq!(ic("123,5,2", "1,200"), ns("123,200"));
        assert_eq!(ic("0123456,9870", "98700,01234"), ns("0123456,98700"));
        assert_eq!((ns("23") - ns("23000")).intersect(&ns("23000")), ns(""));
        assert_eq!(
            (ns("23,999000,000999,90009,09990") - ns("234567890")).intersect(&ns("234567890,9,0")),
            ns("999000,000999,90009,09990")
        );
    }
}
