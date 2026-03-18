#![allow(unused)]
use super::{NumsMatch, U4Number};
use std::marker::PhantomData;

#[derive(Debug, Clone)]
/// Iterator over prefixes in `pfx - to_skip`.
pub struct U4NumberDivided<const NUM_SIZE: usize> {
    nxt: Vec<u8>,
    skip: Vec<u8>,
    b_len: usize,
    phant: PhantomData<U4Number<NUM_SIZE>>,
}

impl<const NUM_SIZE: usize> U4NumberDivided<NUM_SIZE> {
    /// Creates an iterator that yields prefixes in `pfx - to_skip`.
    pub fn new(pfx: U4Number<NUM_SIZE>, to_skip: U4Number<NUM_SIZE>) -> Self {
        //println!("{} / {} is {:?}", pfx, to_skip, pfx.overlap_check(&to_skip));
        match pfx.overlap_check(&to_skip) {
            NumsMatch::SUPERSET => {
                let mut nd = Self {
                    nxt: pfx.to_bytes(),
                    skip: to_skip.to_bytes(),
                    b_len: pfx.len(),
                    phant: PhantomData,
                };
                nd.nxt.push(0);
                nd.chk_skip();
                nd
            }
            NumsMatch::EQUAL | NumsMatch::SUBSET => Self {
                nxt: vec![],
                skip: vec![],
                b_len: 0,
                phant: PhantomData,
            },
            _ => Self {
                nxt: pfx.to_bytes(),
                skip: vec![],
                b_len: pfx.len(),
                phant: PhantomData,
            },
        }
    }
}

impl<const NUM_SIZE: usize> U4NumberDivided<NUM_SIZE> {
    fn chk_skip(&mut self) {
        let mut l = self.nxt.len();
        if self.nxt[l - 1] == self.skip[l - 1] {
            if l < self.skip.len() {
                while l < self.skip.len() {
                    self.nxt.push(0);
                    l += 1;
                    if self.skip[l - 1] != 0 {
                        return;
                    }
                }
            }
            self.go_next()
        }
    }
    fn go_next(&mut self) {
        let mut l = self.nxt.len();
        assert_ne!(l, 0);
        while l > self.b_len && self.nxt[l - 1] == 9 {
            self.nxt.pop();
            l -= 1;
        }
        if l == self.b_len {
            self.nxt.clear();
        } else {
            self.nxt[l - 1] += 1;
            self.chk_skip();
        }
    }
}

impl<const NUM_SIZE: usize> Iterator for U4NumberDivided<NUM_SIZE> {
    type Item = U4Number<NUM_SIZE>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.nxt.len() == 0 {
            return None;
        }
        let num = U4Number::from_u8(&self.nxt);
        if self.skip.len() == 0 {
            self.nxt.clear();
        } else {
            self.go_next();
        }
        Some(num)
    }
}
