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
    pub fn skip_rest(&mut self) {
        self.index = self.value.len();
    }
    pub fn read_next(&mut self) -> u8 {
        let i = self.index;
        self.index += 1;
        self.value[i]
    }
    pub fn peek(&self) -> u8 {
        self.value[self.index]
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
    pub fn get_value_ref(&self) -> &[u8] {
        self.value.as_slice()
    }
    pub fn get_value(self) -> Vec<u8> {
        self.value
    }
}