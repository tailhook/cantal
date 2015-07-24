use cantal::Value as TipValue;


#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
pub enum HistoryChunk {
    State((u64, String)),
    Counter(Vec<Option<u64>>),
    Integer(Vec<Option<i64>>),
    Float(Vec<Option<f64>>),
}

pub struct HistoryChunkIter<'a> {
    chunk: &'a HistoryChunk,
    start_index: usize,
    end_index: usize,
}

impl HistoryChunk {
    pub fn iter<'x>(&'x self) -> HistoryChunkIter {
        use self::HistoryChunk::*;
        let size = match self {
            &State(_) => 1,
            &Counter(ref slc) => slc.len(),
            &Integer(ref slc) => slc.len(),
            &Float(ref slc) => slc.len(),
        };
        assert!(size >= 1);
        HistoryChunkIter {
            chunk: self,
            start_index: 0,
            end_index: size,
        }
    }
}

impl<'a> Iterator for HistoryChunkIter<'a> {
    type Item = Option<TipValue>;
    fn next(&mut self) -> Option<Option<TipValue>> {
        use self::HistoryChunk as S;
        use cantal::Value as D;
        let idx = self.start_index;
        if idx >= self.end_index {
            return None;
        }
        self.start_index += 1;
        Some(match self.chunk {
            &S::State((ts, ref val)) => Some(D::State(ts, val.clone())),
            &S::Counter(ref slc) => slc[idx].map(D::Counter),
            &S::Integer(ref slc) => slc[idx].map(D::Integer),
            &S::Float(ref slc) => slc[idx].map(D::Float),
        })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<'a> DoubleEndedIterator for HistoryChunkIter<'a> {
    fn next_back(&mut self) -> Option<Option<TipValue>> {
        use self::HistoryChunk as S;
        use cantal::Value as D;
        if self.end_index <= self.start_index {
            return None;
        }
        self.end_index -= 1;
        Some(match self.chunk {
            &S::State((ts, ref val)) => Some(D::State(ts, val.clone())),
            &S::Counter(ref slc) => slc[self.end_index].map(D::Counter),
            &S::Integer(ref slc) => slc[self.end_index].map(D::Integer),
            &S::Float(ref slc) => slc[self.end_index].map(D::Float),
        })
    }
}

impl<'a> ExactSizeIterator for HistoryChunkIter<'a> {
    fn len(&self) -> usize {
        return self.end_index - self.start_index;
    }
}
