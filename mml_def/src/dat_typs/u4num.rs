use std::cmp::Ordering;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
/// Relationship between two numeric prefix values.
pub enum NumsMatch {
    ABOVE,
    SUPERSET,
    SUBSET,
    EQUAL,
    BELOW,
}
impl From<NumsMatch> for Ordering {
    fn from(value: NumsMatch) -> Self {
        use NumsMatch::*;
        match value {
            ABOVE => Ordering::Greater,
            SUBSET | SUPERSET | EQUAL => Ordering::Equal,
            BELOW => Ordering::Less,
        }
    }
}
//const NUM_SIZE:usize = 12;
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// Packed digit sequence stored in 4-bit nibbles.
pub struct U4Number<const NUM_SIZE: usize>([u8; NUM_SIZE]);

impl<const NUM_SIZE: usize> U4Number<NUM_SIZE> {
    //type BUF = [u8; (NUM_SIZE -1) * 2];
    #[inline]
    /// Returns the number of digits currently stored.
    pub fn len(&self) -> usize {
        self.0[NUM_SIZE - 1] as usize
    }
    #[inline]
    /// Sets the logical digit length.
    pub fn set_len(&mut self, len: u8) {
        self.0[NUM_SIZE - 1] = len;
    }

    /// Formats digits using prefix encoding (`E`/`F` for special digits).

    pub fn to_pfx(&self) -> String {
        //let mut buf: [u8; (NUM_SIZE - 1) * 2] = [0; (NUM_SIZE - 1) * 2];
        //let mut buf:Self::BT = [0u8; NUM_SIZE*2];
        let mut buf = String::with_capacity(self.len());
        for i in 0..(self.len()) {
            let (p, even) = (i / 2, i % 2 == 0);
            buf.push({
                let d = if even {
                    self.0[p] >> 4
                } else {
                    self.0[p] & 0xfu8
                };
                match d {
                    0..=9 => (b'0' + d) as char,
                    14 => 'E',
                    15 => 'F',
                    _ => '?',
                }
            });
        }
        buf
    }

    /// Returns digits as a byte vector.

    pub fn to_bytes(&self) -> Vec<u8> {
        //let mut buf: [u8; (NUM_SIZE - 1) * 2] = [0; (NUM_SIZE - 1) * 2];
        //let mut buf:Self::BT = [0u8; NUM_SIZE*2];
        let mut buf = Vec::with_capacity(self.len());
        for i in 0..(self.len()) {
            let (p, even) = (i / 2, i % 2 == 0);
            buf.push({
                if even {
                    self.0[p] >> 4
                } else {
                    self.0[p] & 0xfu8
                }
            });
        }
        buf
    }

    /// Compares two numbers with prefix-inclusion semantics.

    pub fn include_cmp(&self, num2: &Self) -> Ordering {
        let l = std::cmp::min(self.len(), num2.len());
        for p in 0..l {
            let cmp = self.get_at(p).cmp(&num2.get_at(p));
            if cmp == Ordering::Equal {
                continue;
            } else {
                return cmp;
            }
        }
        if self.len() <= num2.len() {
            Ordering::Equal
        } else {
            Ordering::Greater
        }
    }

    /// Classifies overlap relationship between two prefixes.

    pub fn overlap_check(&self, num2: &Self) -> NumsMatch {
        let l = std::cmp::min(self.len(), num2.len());
        for p in 0..l {
            let (a, b) = (self.get_at(p), num2.get_at(p));
            match a.cmp(&b) {
                Ordering::Greater => return NumsMatch::ABOVE,
                Ordering::Less => return NumsMatch::BELOW,
                Ordering::Equal => continue,
            }
        }
        match self.len().cmp(&num2.len()) {
            Ordering::Equal => NumsMatch::EQUAL,
            Ordering::Less => NumsMatch::SUPERSET,
            Ordering::Greater => NumsMatch::SUBSET,
        }
    }

    /// Returns the length of the common prefix.

    pub fn same_prefix_len(&self, num2: &Self) -> usize {
        let l = std::cmp::min(self.len(), num2.len());
        let mut p = 0;
        while p < l / 2 {
            if self.0[p] != num2.0[p] {
                break;
            }
            p += 1;
        }
        if 2 * p < l && (0xf0u8 & self.0[p]) == (0xf0u8 & num2.0[p]) {
            p * 2 + 1
        } else {
            p * 2
        }
    }

    /// Returns the immediate previous number if it exists.

    pub fn get_precede_num(&self) -> Option<Self> {
        let mut pn = *self;
        for i in (0..pn.len()).rev() {
            let d = self.get_at(i);
            if d == 0 {
                pn.set_at(i, 9);
            } else {
                pn.set_at(i, d - 1);
                return Some(pn);
            }
        }
        None
    }

    /// Returns the immediate next number if it exists.

    pub fn get_succeed_num(&self) -> Option<Self> {
        let mut pn = *self;
        for i in (0..pn.len()).rev() {
            let d = self.get_at(i);
            if d == 9 {
                pn.set_at(i, 0);
            } else {
                pn.set_at(i, d + 1);
                return Some(pn);
            }
        }
        None
    }

    /// Checks whether `num2` is the mergeable successor of `self`.

    pub fn is_succeed_by(&self, num2: &Self) -> bool {
        let mut l1 = self.len();
        while l1 > 0 && 9 == self.get_at(l1 - 1) {
            l1 -= 1
        }
        let mut l2 = num2.len();
        while l2 > 0 && 0 == num2.get_at(l2 - 1) {
            l2 -= 1
        }
        // l1/l2 长度至少为 2 且相等时，才判断是否可连续合并。
        if l1 == l2 && l1 > 1 && self.get_at(l1 - 1) + 1 == num2.get_at(l2 - 1) {
            //println!("is_succ? {}", l1);
            //let l = l1 - 1;
            self.same_prefix_of(num2, l1 - 1)
        } else {
            false
        }
    }

    /// Checks whether two values share the same prefix of length `l`.

    pub fn same_prefix_of(&self, num2: &Self, l: usize) -> bool {
        if l > self.len() {
            return false;
        }
        let mut p = 0;
        while p < l / 2 {
            if self.0[p] != num2.0[p] {
                return false;
            }
            p += 1
        }
        if 2 * p < l && (0xf0u8 & self.0[p]) != (0xf0u8 & num2.0[p]) {
            return false;
        }
        true
    }

    /// Removes `pfx` from the front when it matches.

    pub fn strip_prefix(&self, pfx: &Self) -> Self {
        if self.same_prefix_of(pfx, pfx.len()) {
            self.copy_suffix(pfx.len())
        } else {
            *self
        }
    }

    /// Prepends `pfx` to the current number.

    pub fn with_prefix(&self, pfx: &Self) -> Self {
        let mut n = *pfx;
        for p in 0..self.len() {
            n.set_at(p + pfx.len(), self.get_at(p));
        }
        n.set_len((pfx.len() + self.len()) as u8);
        n
    }

    /// Copies the suffix after dropping the first `l` digits.

    pub fn copy_suffix(&self, l: usize) -> Self {
        let mut n = Self::new();
        if l <= self.len() {
            for p in l..self.len() {
                n.set_at(p - l, self.get_at(p));
            }
            n.set_len((self.len() - l) as u8);
        }
        n
    }
    /// Copies at most the first `l` digits.
    pub fn copy_prefix(&self, l: usize) -> Self {
        let l = std::cmp::min(l, self.len());
        let mut n = Self::new();
        let size = (l + 1) / 2;
        for p in 0..=size {
            n.0[p] = self.0[p];
        }
        if 2 * size > l {
            n.0[size] &= 0xfu8;
        }
        n.set_len(l as u8);
        n
    }

    /// Expands to `snb_len` and returns the base value plus count factor.

    pub fn to_snb(&self, snb_len: usize) -> Option<(Self, usize)> {
        if snb_len < self.len() || snb_len > 2 * (NUM_SIZE - 1) {
            return None;
        }
        let mut cnt = 1;
        let mut n = *self;
        for p in self.len()..snb_len {
            n.set_at(p, 0);
            cnt *= 10;
        }
        n.set_len(snb_len as u8);
        Some((n, cnt))
    }

    #[inline]
    /// Checks whether the last digit equals `d`.
    pub fn is_end_with(&self, d: u8) -> bool {
        let l = self.len();
        if l == 0 {
            false
        } else {
            self.get_at(l - 1) == d
        }
    }

    #[inline]
    /// Reads one digit at position `i`.
    pub fn get_at(&self, i: usize) -> u8 {
        let (p, odd) = (i / 2, i % 2 != 0);
        if odd {
            self.0[p] & 0xfu8
        } else {
            self.0[p] >> 4
        }
    }

    #[inline]
    /// Writes one digit at position `i`.
    pub fn set_at(&mut self, i: usize, d: u8) {
        let (p, odd) = (i / 2, i % 2 != 0);
        if odd {
            self.0[p] &= 0xf0u8;
            self.0[p] |= d;
        } else {
            self.0[p] &= 0xfu8;
            self.0[p] |= d << 4
        }
    }
}

impl<const NUM_SIZE: usize> U4Number<NUM_SIZE> {
    /// Builds a packed number from raw digit bytes.
    pub fn from_u8(buf: &[u8]) -> Self {
        let mut u4num: [u8; NUM_SIZE] = [0; NUM_SIZE];
        buf.chunks(2).enumerate().fold(&mut u4num, |vec, (i, chk)| {
            vec[i] = if chk.len() == 2 {
                (chk[0] << 4) | chk[1]
            } else {
                chk[0] << 4
            };
            vec
        });
        u4num[NUM_SIZE - 1] = buf.len() as u8;
        Self(u4num)
    }

    /// Creates an empty number with zero logical length.

    pub fn new() -> Self {
        Self([0; NUM_SIZE])
    }
}

impl<const NUM_SIZE: usize> FromStr for U4Number<NUM_SIZE> {
    //type Err = super::FormatError;
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() > 2 * (NUM_SIZE - 1) {
            return Err(format!(
                "fail parsing U4Number, the input:{} is out of size",
                s
            ));
        }
        let mut num = [0; NUM_SIZE];
        let it = s.as_bytes();
        for (i, d) in it.into_iter().enumerate() {
            let (p, even) = (i / 2, i % 2 == 0);
            let d = {
                let d = match d {
                    b'0'..=b'9' => d - b'0',
                    b'#' | b'F' | b'f' => 15,
                    b'*' | b'E' | b'e' => 14,
                    _ => return Err(format!("'{}' is not valid dail number", *d as char)),
                    //_ => super::FormatError,
                };
                if even {
                    d << 4
                } else {
                    d
                }
            };
            num[p] |= d;
        }
        num[NUM_SIZE - 1] = s.len() as u8;
        Ok(Self(num))
    }
}

impl<const NUM_SIZE: usize> From<&[u8]> for U4Number<NUM_SIZE> {
    fn from(value: &[u8]) -> Self {
        //println!("dbg from<&[u8]> \"{:?}\"", value);
        if value.len() > 2 * (NUM_SIZE - 1) {
            panic!("U4Number::from() input out of size")
        }
        let mut num = [0; NUM_SIZE];
        for (i, d) in value.into_iter().enumerate() {
            let (p, even) = (i / 2, i % 2 == 0);
            let d = {
                let d = match d {
                    b'0'..=b'9' => d - b'0',
                    b'#' | b'F' | b'f' => 15,
                    b'*' | b'E' | b'e' => 14,
                    //_ => panic!(format!("{} is not a valid dail number", (char::from(d)).to_string())),
                    _ => panic!("Invaild char while U4Number::from"),
                };
                if even {
                    d << 4
                } else {
                    d
                }
            };
            num[p] |= d;
        }
        num[NUM_SIZE - 1] = value.len() as u8;
        Self(num)
    }
}
impl<const NUM_SIZE: usize> From<&str> for U4Number<NUM_SIZE> {
    fn from(value: &str) -> Self {
        //println!("From<&str>  \"{}\"", value);
        From::from(value.as_bytes())
    }
}

impl<const NUM_SIZE: usize> From<u64> for U4Number<NUM_SIZE> {
    fn from(value: u64) -> Self {
        From::from(value.to_string().as_bytes())
    }
}

impl<const N: usize, const NUM_SIZE: usize> From<&[u8; N]> for U4Number<NUM_SIZE> {
    fn from(value: &[u8; N]) -> Self {
        From::from(&value[..])
    }
}

use std::fmt::Write;
impl<const NUM_SIZE: usize> Display for U4Number<NUM_SIZE> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //let mut buf: [u8; (NUM_SIZE - 1) * 2] = [0; (NUM_SIZE - 1) * 2];
        for i in 0..(self.len()) {
            let (p, even) = (i / 2, i % 2 == 0);
            f.write_char({
                let d = if even {
                    self.0[p] >> 4
                } else {
                    self.0[p] & 0xfu8
                };
                match d {
                    0..=9 => (b'0' + d) as char,
                    14 => '*',
                    15 => '#',
                    _ => '?',
                }
            })?
        }
        Ok(())
    }
}

impl<const NUM_SIZE: usize> std::fmt::Debug for U4Number<NUM_SIZE> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "U4Number<{NUM_SIZE}>({self})")
    }
}
