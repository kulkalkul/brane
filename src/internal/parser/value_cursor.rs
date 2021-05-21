use std::ops::Range;

pub struct ValueCursor {
    index: usize,
    value: Vec<u8>,
}

impl ValueCursor {
    pub fn new(value: Vec<u8>) -> ValueCursor {
        ValueCursor {
            index: 0,
            value,
        }
    }
}

impl ValueCursor {
    pub fn skip_next(&mut self) {
        self.skip_by(1);
    }
    pub fn skip_by(&mut self, n: usize) {
        self.index += n;
    }
    pub fn skip_reverse_by(&mut self, n: usize) {
        self.index -= n;
    }
    pub fn read_next(&mut self) -> u8 {
        let i = self.index;
        self.index += 1;
        self.value[i]
    }
    pub fn read_by(&mut self, n: usize) -> &[u8] {
        let i = self.index;
        self.index += n;
        &self.value[i..self.index]
    }
    pub fn read_range(&mut self, range: Range<usize>) -> &[u8] {
        &self.value[range]
    }
    pub fn get_index(&self) -> usize {
        self.index
    }
    pub fn get_value(&self) -> &[u8] {
        self.value.as_slice()
    }
}