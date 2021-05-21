use crate::internal::parser::parser::{Parser, Loop};
use crate::internal::parser::delimiters::{json_delimiters, tson_delimiters};
use crate::internal::parser::value_cursor::ValueCursor;
use crate::internal::parser::parsed::Parsed;

pub struct JSONParser {
    cursor: ValueCursor,
    parsed: Parsed,
    stack: Vec<usize>,
}

impl JSONParser {
    pub fn new(json: String) -> JSONParser {
        let len = json.len();
        Self::with_capacity(json, len)
    }
    pub fn new_with_id(id: String, json: String) -> JSONParser {
        let capacity = json.len() + id.len();

        let mut parser = Self::with_capacity(json, capacity);

        parser.write_object_begin();
        parser.cursor.skip_by(1);

        let key = "_id".as_bytes();
        parser.parsed.write(tson_delimiters::STRING);
        parser.write_length(key.len() as u32);
        parser.parsed.write_slice(key);
        parser.write_pair();

        parser.parsed.write(tson_delimiters::STRING);
        parser.write_length(id.len() as u32);
        parser.parsed.write_slice(id.as_bytes());
        parser.write_separator();

        parser
    }
    fn with_capacity(json: String, capacity: usize) -> JSONParser {
        JSONParser {
            cursor: ValueCursor::new(json.into_bytes()),
            parsed: Parsed::with_capacity(capacity),
            stack: Vec::new(),
        }
    }
}

impl Parser for JSONParser {
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
    fn write_length(&mut self, length: u32) {
        self.parsed.write_slice(&length.to_le_bytes());
    }
    fn begin_collection(&mut self) {
        self.write_length(0);
        self.stack.push(self.parsed.get_parsed_len() - 1);
    }
    fn end_collection(&mut self) {
        let start = self.stack.pop().unwrap();
        let len = (self.parsed.get_parsed_len() - start) as u32;
        self.parsed.rewrite_slice(start - 3, &len.to_le_bytes());
    }
}

impl JSONParser {
    fn write_object_begin(&mut self) {
        self.parsed.write(tson_delimiters::OBJECT_BEGIN);
        self.begin_collection();
    }
    fn write_object_end(&mut self) {
        self.parsed.write(tson_delimiters::OBJECT_END);
        self.end_collection();
    }
    fn write_array_begin(&mut self) {
        self.parsed.write(tson_delimiters::ARRAY_BEGIN);
        self.begin_collection();
    }
    fn write_array_end(&mut self) {
        self.parsed.write(tson_delimiters::ARRAY_END);
        self.end_collection();
    }
    fn write_string(&mut self) {
        self.parsed.write(tson_delimiters::STRING);
        let string = self.read_string();
        let length = string.len() as u32;
        self.write_length(length);
        self.parsed.write_slice(string.as_slice());
    }
    fn write_true(&mut self) {
        self.parsed.write(tson_delimiters::TRUE);
        self.cursor.skip_by(3);
    }
    fn write_false(&mut self) {
        self.parsed.write(tson_delimiters::FALSE);
        self.cursor.skip_by(4);
    }
    fn write_null(&mut self) {
        self.parsed.write(tson_delimiters::NULL);
        self.cursor.skip_by(3);
    }
    fn write_pair(&mut self) {
        self.parsed.write(tson_delimiters::PAIR);
    }
    fn write_separator(&mut self) {
        self.parsed.write(tson_delimiters::SEPARATOR);
    }
    fn write_number(&mut self) {
        self.cursor.skip_reverse_by(1);
        let number = self.read_number();
        let number: f64 = String::from_utf8(number.to_vec()).unwrap().parse().unwrap();
        self.parsed.write(tson_delimiters::NUMBER);
        self.parsed.write_slice(&number.to_le_bytes());
    }
}

impl JSONParser {
    fn read_number(&mut self) -> &[u8] {
        let prev = self.cursor.get_index();

        loop {
            match self.cursor.read_next() {
                json_delimiters::SEPARATOR => break,
                json_delimiters::OBJECT_END => break,
                json_delimiters::ARRAY_END => break,
                _ => continue,
            }
        }

        self.cursor.skip_reverse_by(1);
        let current = self.cursor.get_index();
        self.cursor.read_range(prev..current)
    }
    fn read_string(& mut self) -> Vec<u8> {
        let prev = self.cursor.get_index();

        loop {
            let val = self.cursor.read_next();
            if val == b'\\' {
                self.cursor.skip_next();
                continue;
            }
            if val == json_delimiters::STRING {
                break;
            }
        }

        let current = self.cursor.get_index();
        self.cursor.read_range(prev..current - 1).to_vec()
    }
}