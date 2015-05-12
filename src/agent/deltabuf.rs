use std::cmp::min;
use std::ops::{Sub, BitAnd, BitOr, Shr};
use std::fmt::Display;
use std::collections::VecDeque;

const SIGN_BIT: i64 = 0b00100000;
const SPECIAL_BIT: i64 = 0b01000000;
const SPECIAL_BITS: i64 = 0b01100000;
const SPECIAL_MASK: i64 = 0b00011111;
//                       vv
const SKIP_BITS: i64 = 0b01100000;
const ZERO_BITS: i64 = 0b01000000;
//                       ^^
const FIRST_BYTE_SHIFT: u32 = 5;
const CONTINUATION_BIT: i64 = 0b10000000;
const CONTINUATION_SHIFT: u32 = 7;
const FIRST_BYTE_MASK: i64 = 0b00011111;
const CONTINUATION_MASK: i64 = 0b01111111;

#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct DeltaBuf(VecDeque<u8>);

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Delta {
    Positive(u64),
    Negative(u64),
    Skip,
}

impl DeltaBuf {
    pub fn new() -> DeltaBuf {
        return DeltaBuf(VecDeque::new());
    }
    pub fn push(&mut self, old_value: i64, new_value: i64, mut age_diff: u64)
    {
        let DeltaBuf(ref mut deque) = *self;
        let byte_mask = 0xFF;
        if age_diff == 0 {
            warn!("Duplicate write at same age"); // Shouldn't we panic?
            return;
        }
        age_diff -= 1;
        while age_diff > 0 {
            let cd = min(age_diff as i64, SPECIAL_MASK);
            deque.push_front((SKIP_BITS | cd) as u8);
            age_diff -= cd as u64;
        }
        let (mut delta, sign) = if old_value > new_value {
            (old_value - new_value, SIGN_BIT)
        } else {
            (new_value - old_value, 0)
        };
        if delta == 0 {
            if deque.len() > 0 && deque[0] as i64 & SPECIAL_BITS == ZERO_BITS {
                let old_val = deque[0] & SPECIAL_MASK as u8;
                if old_val < SPECIAL_MASK as u8 {
                    deque[0] = (old_val+1) | ZERO_BITS as u8;
                    return;
                }
            }
            deque.push_front(ZERO_BITS as u8 | 1);
            return;
        }
        deque.push_front((sign | (delta & FIRST_BYTE_MASK)) as u8);
        delta = delta >> FIRST_BYTE_SHIFT;
        while delta > From::from(0) {
            deque.push_front(((delta & CONTINUATION_MASK) |
                CONTINUATION_BIT) as u8);
            delta = delta >> CONTINUATION_SHIFT;
        }
    }
    pub fn deltas(&self, limit: usize) -> Vec<Delta> {
        let DeltaBuf(ref deque) = *self;
        let mut res = vec!();
        let mut delta: u64 = 0;
        'outer: for &bits in deque.iter() {
            let byte = bits as i64;
            if res.len() >= limit { break; }
            if byte & CONTINUATION_BIT != 0 {
                delta <<= CONTINUATION_SHIFT;
                delta |= (byte & CONTINUATION_MASK) as u64;
            } else {
                if byte & SPECIAL_BIT != 0 {
                    if byte & SPECIAL_BITS == SKIP_BITS {
                        for _ in 0..(byte & SPECIAL_MASK) {
                            res.push(Delta::Skip);
                            if res.len() >= limit { break 'outer; }
                        }
                    } else if byte & SPECIAL_BITS == ZERO_BITS {
                        for _ in 0..(byte & SPECIAL_MASK) {
                            res.push(Delta::Positive(0));
                            if res.len() >= limit { break 'outer; }
                        }
                    } else {
                        unreachable!();
                    }
                } else {
                    delta <<= FIRST_BYTE_SHIFT;
                    delta |= (byte & FIRST_BYTE_MASK) as u64;
                    if byte & SIGN_BIT != 0 {
                        res.push(Delta::Negative(delta));
                    } else {
                        res.push(Delta::Positive(delta));
                    }
                    delta = 0;
                }
            }
        }
        return res;
    }
    pub fn truncate(&mut self, limit: usize) -> usize {
        if limit == 0 {
            *self = DeltaBuf::new();  // Is this efficient?
            return 0;
        }
        match self._truncate_bytes(limit) {
            Ok((limit_bytes, truncate_num)) => {
                let DeltaBuf(ref mut deque) = *self;
                if truncate_num > 0 {
                    let b = deque[limit_bytes-1];
                    debug_assert!(b & CONTINUATION_BIT as u8 == 0);
                    debug_assert!(b & SPECIAL_MASK as u8 > truncate_num as u8);
                    deque[limit_bytes-1] = (b & SPECIAL_BITS as u8) |
                        ((b & SPECIAL_MASK as u8) - truncate_num as u8);
                }
                // TODO(tailhook) use truncate
                while deque.len() > limit_bytes {
                    deque.pop_back();
                }
                // deque.truncate(limit_bytes);
                limit
            }
            Err(num_current) => num_current,
        }
    }
    fn _truncate_bytes(&self, limit: usize) -> Result<(usize, u8), usize> {
        let DeltaBuf(ref deque) = *self;
        let mut counter = 0usize;
        for (idx, &byte) in deque.iter().enumerate() {
            if byte & CONTINUATION_BIT as u8 != 0 {
                continue;
            }
            if byte & SPECIAL_BIT as u8 != 0 {
                let cnt = byte & SPECIAL_MASK as u8;
                let newcnt = counter + cnt as usize;
                if newcnt == limit {
                    return Ok((idx+1, 0));
                } else if newcnt > limit {
                    return Ok((idx+1, cnt - (limit - counter) as u8));
                } else {
                    counter = newcnt;
                }
            } else {
                counter += 1;
                if counter >= limit {
                    return Ok((idx+1, 0));
                }
            }
        }
        return Err(counter);
    }
}


#[cfg(test)]
mod test {

    use std::num::{Int, FromPrimitive};
    use std::fmt::Display;
    use super::{Delta, DeltaBuf};
    use super::Delta::*;

    fn to_buf<T:Int+FromPrimitive+Display>(values: &[T]) -> DeltaBuf {
        let mut buf = DeltaBuf::new();
        for idx in 0..(values.len()-1) {
            buf.push(values[idx], values[idx+1], 1);
        }
        return buf;
    }
    fn to_buf_opt<T:Int+FromPrimitive+Display>(values: &[Option<T>])
        -> DeltaBuf
    {
        let mut buf = DeltaBuf::new();
        let mut off = 0;
        let mut old = values[0].unwrap();
        for idx in 0..(values.len()-1) {
            off += 1;
            values[idx+1].map(|v| {
                buf.push(old, v, off);
                old = v;
                off = 0;
            });
        }
        return buf;
    }

    fn deltify<T:Int+FromPrimitive+Display>(values: &[T]) -> Vec<Delta> {
        return to_buf(values).deltas(100)
    }
    fn deltify_opt<T:Int+FromPrimitive+Display>(values: &[Option<T>])
        -> Vec<Delta>
    {
        return to_buf_opt(values).deltas(100)
    }

    #[test]
    fn u64_no_skips() {
        assert_eq!(deltify(&[1u64, 2, 10, 1000, 100000, 5, 5, 5, 5, 10]),
            vec!(Positive(5), Positive(0), Positive(0), Positive(0),
                 Negative(99995), Positive(99000),
                 Positive(990), Positive(8), Positive(1) ));
    }
    #[test]
    fn u64_skips() {
        assert_eq!(deltify_opt(&[Some(1u64), Some(2), None, Some(10),
                                 Some(1000), None, None, None, None,
                                 Some(100000), Some(5), Some(10)]),
            vec!(Positive(5), Negative(99995), Positive(99000),
                 Skip, Skip, Skip, Skip,
                 Positive(990), Positive(8), Skip, Positive(1) ));
    }

    #[test]
    fn u64_partial_read() {
        let buf = to_buf_opt(&[Some(1u64), Some(2), None, Some(10),
                               Some(1000), None, None, None, None,
                               Some(100000), Some(5), Some(10)]);
        let result = vec!(Positive(5), Negative(99995), Positive(99000),
                          Skip, Skip, Skip, Skip,
                          Positive(990), Positive(8), Skip, Positive(1));
        for i in 0..result.len() {
            assert_eq!(&buf.deltas(i)[..], &result[..i]);
        }
    }

    #[test]
    fn u64_truncate() {
        let buf = to_buf_opt(&[Some(1u64), Some(2), None, Some(10),
                               Some(1000), None, None, None, None,
                               Some(100000), Some(5), Some(10)]);
        let result = vec!(Positive(5), Negative(99995), Positive(99000),
                          Skip, Skip, Skip, Skip,
                          Positive(990), Positive(8), Skip, Positive(1));
        for i in 0..result.len() {
            let mut b = buf.clone();
            assert_eq!(b.truncate(i), i);
            assert_eq!(&b.deltas(100)[..], &result[..i]);
        }
        let mut b = buf.clone();
        assert_eq!(b.deltas(100).len(), 11);
        assert_eq!(b.truncate(100), 11);
        assert_eq!(b.deltas(100), result);
    }
}
