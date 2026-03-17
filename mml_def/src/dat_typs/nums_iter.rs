use super::U4Number;
use std::marker::PhantomData;

#[derive(Debug, Clone)]
/// 前缀区间迭代器，将 `[start, end]` 转换为可压缩前缀序列。
pub struct RangeOfPrefix<const NUM_SIZE: usize> {
    nxt: Vec<u8>,
    end: Vec<u8>,
    b_len: usize,
    phantom: PhantomData<U4Number<NUM_SIZE>>,
}
impl<const NUM_SIZE: usize> RangeOfPrefix<NUM_SIZE> {
    #[inline]
    fn len_base(st: &[u8], ed: &[u8]) -> usize {
        let mut b_len = 0;
        let min_len = std::cmp::min(st.len(), ed.len());
        while b_len < min_len {
            if st[b_len] != ed[b_len] {
                break;
            }
            b_len += 1;
        }
        b_len
    }

    /// 构造前缀区间迭代器（要求 `start <= end`）。

    pub fn new(start: U4Number<NUM_SIZE>, end: U4Number<NUM_SIZE>) -> Self {
        Self::from_u4(start, end)
    }

    /// 以 `U4Number` 形式构造前缀区间迭代器。

    pub fn from_u4(start: U4Number<NUM_SIZE>, end: U4Number<NUM_SIZE>) -> Self {
        assert!(
            start <= end,
            "NumRange: ending number must not less than start number."
        );

        let mut st = start.to_bytes();
        let mut ed = end.to_bytes();
        let b_len = Self::len_base(&st, &ed);
        //let min_len = std::cmp::max(b_len, 1);
        while st.len() > b_len + 1 && *st.last().unwrap() == 0 {
            st.pop();
        }
        while (ed.len() > b_len + 1) && *ed.last().unwrap() == 9 {
            ed.pop();
        }
        if st.len() == b_len + 1
            && ed.len() == b_len + 1
            && *st.last().unwrap() == 0
            && *ed.last().unwrap() == 9
        {
            st.pop();
            ed.pop();
        }
        Self {
            nxt: st,
            end: ed,
            b_len,
            phantom: PhantomData,
        }
    }
    //    fn from_str(start: &str, end: &str) -> Self {
    //        Self {
    //            st: start.as_bytes().to_vec(),
    //            ed: end.as_bytes().to_vec() ,
    //            b_len: Self::b_len(start.as_bytes(), end.as_bytes()),
    //            yd: PhantomData,
    //        }
    //    }
    fn calc_size(&self) -> usize {
        if self.is_done() {
            return 0;
        }
        if self.nxt.len() == self.b_len {
            return 1;
        }

        let p = self.b_len;
        let mut calc = self.end[p] - self.nxt[p] + 1;

        for b in self.nxt[p + 1..self.nxt.len()].iter() {
            calc += 10 - *b - 1;
        }
        for b in self.end[p + 1..self.end.len()].iter() {
            calc += *b;
        }
        calc as usize
    }

    fn go_next(&mut self) {
        let bl = self.b_len;
        if self.nxt.len() <= bl {
            self.nxt.truncate(0);
        } else if self.nxt[bl] < self.end[bl] {
            while let Some(9) = self.nxt.last() {
                self.nxt.pop();
            }
            *self.nxt.last_mut().unwrap() += 1;
            if self.nxt.len() == bl + 1 {
                self.do_extend_if_need();
            }
        } else {
            //println!("{:?}, {:?}", self.st, self.ed);
            let extended = self.do_extend_if_need();
            if !extended {
                *self.nxt.last_mut().unwrap() += 1;
                self.do_extend_if_need();
                if self.nxt.len() == self.end.len()
                    && *self.nxt.last().unwrap() > *self.end.last().unwrap()
                {
                    self.nxt.truncate(0);
                }
            }
        }
    }

    #[inline]
    fn do_extend_if_need(&mut self) -> bool {
        let mut l = self.nxt.len();
        let mut extended = false;
        while l < self.end.len() && self.nxt[l - 1] == self.end[l - 1] {
            self.nxt.push(0);
            l = self.nxt.len();
            extended = true;
        }
        extended
    }
    #[inline]
    fn is_done(&self) -> bool {
        self.nxt.len() == 0
    }
}

impl<const NUM_SIZE: usize> Iterator for RangeOfPrefix<NUM_SIZE> {
    type Item = U4Number<NUM_SIZE>;
    fn next(&mut self) -> Option<Self::Item> {
        //let Self{ref mut st, ref mut ed,  b_len, ..} = *self;
        if self.is_done() {
            return None;
        }
        //println!("{:?}", st);
        let u4num = U4Number::from_u8(self.nxt.as_slice());

        self.go_next();

        Some(u4num)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.calc_size();
        (size, Some(size))
    }
}
