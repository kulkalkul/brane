use crate::internal::parser::parser::{Parser, Loop};
use crate::internal::parser::delimiters::{json_delimiters, tson_delimiters};

pub struct JSONParser {
    index: usize,
    original: Vec<u8>,
    parsed: Vec<u8>,
    stack: Vec<usize>,
}

impl JSONParser {
    pub fn new(json: String) -> JSONParser {
        let capacity = json.len();
        JSONParser {
            index: 0,
            original: json.into_bytes(),
            parsed: Vec::with_capacity(capacity),
            stack: Vec::new(),
        }
    }
    pub fn new_with_id(id: String, json: String) -> JSONParser {
        let capacity = json.len() + id.len();

        let mut parser = JSONParser {
            index: 0, // Skip first object literal
            original: json.into_bytes(),
            parsed: Vec::with_capacity(capacity),
            stack: Vec::new(),
        };

        parser.write_object_begin();
        parser.skip_by(1);

        let key = "_id".as_bytes();
        parser.write(tson_delimiters::STRING);
        parser.write_length(key.len() as u32);
        parser.write_slice(key);
        parser.write_pair();

        parser.write(tson_delimiters::STRING);
        parser.write_length(id.len() as u32);
        parser.write_slice(id.as_bytes());
        parser.write_separator();

        parser
    }
}

impl Parser for JSONParser {
    fn get_index(&self) -> usize {
        self.index
    }
    fn get_original_len(&self) -> usize {
        self.original.len()
    }
    fn get_parsed(self) -> Vec<u8> {
        self.parsed
    }
    fn parse_next(&mut self) {
        match self.read_next() {
            json_delimiters::OBJECT_BEGIN => self.write_object_begin(),
            json_delimiters::OBJECT_END => self.write_object_end(),
            json_delimiters::ARRAY_BEGIN => self.write_array_begin(),
            json_delimiters::ARRAY_END => self.write_array_end(),
            json_delimiters::STRING => self.write_string(),
            json_delimiters::TRUE => self.write_true(),
            json_delimiters::FALSE => self.write_false(),
            json_delimiters::NULL => self.write_null(),
            json_delimiters::PAIR => self.write_pair(),
            json_delimiters::SEPARATOR => self.write_separator(),
            _ => self.write_number(),
        }
    }
}

impl JSONParser {
    fn skip_next(&mut self) {
        self.index += 1;
    }
    fn skip_by(&mut self, n: usize) {
        self.index += n;
    }
    fn read_next(&mut self) -> u8 {
        let i = self.index;
        self.index += 1;
        self.original[i]
    }
    fn write(&mut self, val: u8) {
        self.parsed.push(val);
    }
    fn write_slice(&mut self, slice: &[u8]) {
        self.parsed.extend_from_slice(slice);
    }
    fn write_length(&mut self, length: u32) {
        self.write_slice(&length.to_le_bytes());
    }
    fn rewrite_slice(&mut self, start: usize, slice: &[u8]) {
        for (i, val) in slice.iter().enumerate() {
            self.parsed[start + i] = *val;
        }
    }
    fn begin_collection(&mut self) {
        self.write_length(0);
        self.stack.push(self.parsed.len() - 1);
    }
    fn end_collection(&mut self) {
        let start = self.stack.pop().unwrap();
        let len = (self.parsed.len() - start) as u32;
        self.rewrite_slice(start - 3, &len.to_le_bytes());
    }
}

impl JSONParser {
    fn write_object_begin(&mut self) {
        self.write(tson_delimiters::OBJECT_BEGIN);
        self.begin_collection();
    }
    fn write_object_end(&mut self) {
        self.write(tson_delimiters::OBJECT_END);
        self.end_collection();
    }
    fn write_array_begin(&mut self) {
        self.write(tson_delimiters::ARRAY_BEGIN);
        self.begin_collection();
    }
    fn write_array_end(&mut self) {
        self.write(tson_delimiters::ARRAY_END);
        self.end_collection();
    }
    fn write_string(&mut self) {
        self.write(tson_delimiters::STRING);
        let string = self.read_string();
        let length = string.len() as u32;
        self.write_length(length);
        self.write_slice(string.as_slice());
    }
    fn write_true(&mut self) {
        self.write(tson_delimiters::TRUE);
        self.skip_by(3);
    }
    fn write_false(&mut self) {
        self.write(tson_delimiters::FALSE);
        self.skip_by(4);
    }
    fn write_null(&mut self) {
        self.write(tson_delimiters::NULL);
        self.skip_by(3);
    }
    fn write_pair(&mut self) {
        self.write(tson_delimiters::PAIR);
    }
    fn write_separator(&mut self) {
        self.write(tson_delimiters::SEPARATOR);
    }
    fn write_number(&mut self) {
        self.index -= 1;
        let number = self.read_number();
        let number: f64 = String::from_utf8(number.to_vec()).unwrap().parse().unwrap();
        self.write(tson_delimiters::NUMBER);
        self.write_slice(&number.to_le_bytes());
    }
}

impl JSONParser {
    fn read_number(&mut self) -> &[u8] {
        let i = self.index;

        loop {
            match self.read_next() {
                json_delimiters::SEPARATOR => {
                    self.index -= 1;
                    break;
                },
                json_delimiters::OBJECT_END => {
                    self.index -= 1;
                    break;
                },
                json_delimiters::ARRAY_END => {
                    self.index -= 1;
                    break;
                },
                _ => continue,
            }
        }

        &self.original[i..self.index]
    }
    fn read_string(& mut self) -> Vec<u8> {
        let i = self.index;

        loop {
            let val = self.read_next();
            if val == b'\\' {
                self.skip_next();
                continue;
            }
            if val == json_delimiters::STRING {
                break;
            }
        }

        self.original[i..self.index - 1].to_vec()
    }
}