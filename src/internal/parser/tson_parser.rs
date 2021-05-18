use crate::internal::parser::parser::{Parser, Loop};
use crate::internal::parser::delimiters::{tson_delimiters, json_delimiters};
use std::convert::TryInto;

pub struct TSONParser {
    index: usize,
    original: Vec<u8>,
    parsed: Vec<u8>,
}

impl TSONParser {
    pub fn new(tson: Vec<u8>) -> TSONParser {
        let capacity = tson.len();
        TSONParser {
            index: 0,
            original: tson,
            parsed: Vec::with_capacity(capacity),
        }
    }
}

impl Parser for TSONParser {
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
            tson_delimiters::OBJECT_BEGIN => self.write_object_begin(),
            tson_delimiters::OBJECT_END => self.write_object_end(),
            tson_delimiters::ARRAY_BEGIN => self.write_array_begin(),
            tson_delimiters::ARRAY_END => self.write_array_end(),
            tson_delimiters::STRING => self.write_string(),
            tson_delimiters::NUMBER => self.write_number(),
            tson_delimiters::TRUE => self.write_true(),
            tson_delimiters::FALSE => self.write_false(),
            tson_delimiters::NULL => self.write_null(),
            tson_delimiters::PAIR => self.write_pair(),
            tson_delimiters::SEPARATOR => self.write_separator(),
            (val) => panic!("Unexpected delimiter while parsing TSON: {}", val as char),
        }
    }
}

impl TSONParser {
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
}

impl TSONParser {
    fn write_object_begin(&mut self) {
        self.write(json_delimiters::OBJECT_BEGIN);
        self.skip_by(4);
    }
    fn write_object_end(&mut self) {
        self.write(json_delimiters::OBJECT_END);
    }
    fn write_array_begin(&mut self) {
        self.write(json_delimiters::ARRAY_BEGIN);
        self.skip_by(4);
    }
    fn write_array_end(&mut self) {
        self.write(json_delimiters::ARRAY_END);
    }
    fn write_string(&mut self) {
        let length = self.read_length();
        let string = self.read_string(length);
        self.write(json_delimiters::STRING);
        self.write_slice(string.as_slice());
        self.write(json_delimiters::STRING);
    }
    fn write_number(&mut self) {
        let number = self.read_number();
        let slice = number.to_string();
        let slice = slice.as_bytes();
        self.write_slice(slice);
    }
    fn write_true(&mut self) {
        self.write_slice("true".as_bytes());
    }
    fn write_false(&mut self) {
        self.write_slice("false".as_bytes());
    }
    fn write_null(&mut self) {
        self.write_slice("null".as_bytes());
    }
    fn write_pair(&mut self) {
        self.write(json_delimiters::PAIR);
    }
    fn write_separator(&mut self) {
        self.write(json_delimiters::SEPARATOR);
    }
}

impl TSONParser {
    fn read_length(&mut self) -> u32 {
        let i = self.index;
        self.skip_by(4);

        u32::from_le_bytes(self.original[i..self.index].try_into().unwrap())
    }
    fn read_string(&mut self, length: u32) -> Vec<u8> {
        let i = self.index;
        self.skip_by(length as usize);

        self.original[i..self.index].to_vec()
    }
    fn read_number(&mut self) -> f64 {
        let i = self.index;
        self.skip_by(8);

        f64::from_le_bytes(self.original[i..self.index].try_into().unwrap())
    }
}