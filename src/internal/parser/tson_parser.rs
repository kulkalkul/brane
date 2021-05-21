use crate::internal::parser::parser::{Parser, Loop};
use crate::internal::parser::delimiters::{tson_delimiters, json_delimiters};
use crate::internal::parser::value_cursor::ValueCursor;
use crate::internal::parser::parsed::Parsed;
use std::convert::TryInto;

pub struct TSONParser {
    cursor: ValueCursor,
    parsed: Parsed,
}

impl TSONParser {
    pub fn new(tson: Vec<u8>) -> TSONParser {
        let capacity = tson.len();
        TSONParser {
            cursor: ValueCursor::new(tson),
            parsed: Parsed::with_capacity(capacity),
        }
    }
}

impl Parser for TSONParser {
    type Parsed = Vec<u8>;
    fn get_index(&self) -> usize {
        self.cursor.get_index()
    }

    fn get_original_len(&self) -> usize {
        self.cursor.get_value().len()
    }

    fn get_parsed(self) -> Vec<u8> {
        self.parsed.get_parsed()
    }

    fn parse_next(&mut self) {
        match self.cursor.read_next() {
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
            val => panic!("Unexpected delimiter while parsing TSON: {} = {}", val, val as char),
        }
    }
}

impl TSONParser {
    fn write_object_begin(&mut self) {
        self.parsed.write(json_delimiters::OBJECT_BEGIN);
        self.cursor.skip_by(4);
    }
    fn write_object_end(&mut self) {
        self.parsed.write(json_delimiters::OBJECT_END);
    }
    fn write_array_begin(&mut self) {
        self.parsed.write(json_delimiters::ARRAY_BEGIN);
        self.cursor.skip_by(4);
    }
    fn write_array_end(&mut self) {
        self.parsed.write(json_delimiters::ARRAY_END);
    }
    fn write_string(&mut self) {
        let string = self.read_string();
        self.parsed.write(json_delimiters::STRING);
        self.parsed.write_slice(string.as_slice());
        self.parsed.write(json_delimiters::STRING);
    }
    fn write_number(&mut self) {
        let number = self.read_number();
        let slice = number.to_string();
        let slice = slice.as_bytes();
        self.parsed.write_slice(slice);
    }
    fn write_true(&mut self) {
        self.parsed.write_slice("true".as_bytes());
    }
    fn write_false(&mut self) {
        self.parsed.write_slice("false".as_bytes());
    }
    fn write_null(&mut self) {
        self.parsed.write_slice("null".as_bytes());
    }
    fn write_pair(&mut self) {
        self.parsed.write(json_delimiters::PAIR);
    }
    fn write_separator(&mut self) {
        self.parsed.write(json_delimiters::SEPARATOR);
    }
}

impl TSONParser {
    fn read_length(&mut self) -> u32 {
        let slice = self.cursor.read_by(4);
        u32::from_le_bytes(slice.try_into().unwrap())
    }
    fn read_string(&mut self) -> Vec<u8> {
        let length = self.read_length();
        self.cursor.read_by(length as usize).to_vec()
    }
    fn read_number(&mut self) -> f64 {
        let slice = self.cursor.read_by(8);
        f64::from_le_bytes(slice.try_into().unwrap())
    }
}