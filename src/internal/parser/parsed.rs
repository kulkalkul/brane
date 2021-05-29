
pub struct Parsed {
    parsed: Vec<u8>
}

impl Parsed {
    pub fn new() -> Parsed {
        Parsed {
            parsed: Vec::new(),
        }
    }
    pub fn with_capacity(capacity: usize) -> Parsed {
        Parsed {
            parsed: Vec::with_capacity(capacity),
        }
    }
}

impl Parsed {
    pub fn get_parsed(self) -> Vec<u8> {
        self.parsed
    }
    pub fn get_parsed_clone(&self) -> Vec<u8> {
        self.parsed.clone()
    }
    pub fn get_parsed_len(&self) -> usize {
        self.parsed.len()
    }
    pub fn write(&mut self, val: u8) {
        self.parsed.push(val);
    }
    pub fn write_slice(&mut self, slice: &[u8]) {
        self.parsed.extend_from_slice(slice);
    }
    pub fn rewrite_slice(&mut self, start: usize, slice: &[u8]) {
        let end = start + slice.len();
        self.parsed.splice(start..end, slice.iter().cloned());
    }
}