
const SUPPORTED_VERSION: u32 = 13;

// v8/src/objects/value-serializer.cc
#[repr(u8)]
enum V13 {
    Version             = 0xFF as u8,
    Padding             = '0'  as u8,
    VerifyObjectCount   = '?'  as u8,

    TheHole             = '-'  as u8,
    Undefined           = '_'  as u8,
    Null                = '0'  as u8,
    True                = 'T'  as u8,
    False               = 'F'  as u8,

    Int32               = 'I'  as u8,
    Uint32              = 'U'  as u8,
    Double              = 'D'  as u8,
    BigInt              = 'Z'  as u8,

    Utf8String          = 'S'  as u8,
    OneByteString       = '"'  as u8,
    TwoByteString       = 'c'  as u8,

    ObjectReference     = '^'  as u8,

    BeginJSObject       = 'o'  as u8,
    EndJSObject         = '{'  as u8,
    BeginSparseJSArray  = 'a'  as u8,
    EndSparseJSArray    = '@'  as u8,
    BeginDenseJSArray   = 'A'  as u8,
    EndDenseJSArray     = '$'  as u8,

    Date                = 'D'  as u8,

    TrueObject          = 'y'  as u8,
    FalseObject         = 'x'  as u8,
    NumberObject        = 'n'  as u8,
    BigIntObject        = 'z'  as u8,
    StringObject        = 's'  as u8,
    RegExp              = 'R'  as u8,

    BeginJSMap          = ';'  as u8,
    EndJSMap            = ':'  as u8,
    BeginJSSet          = '\'' as u8,
    EndJSSet            = ','  as u8,

    ArrayBuffer         = 'B'  as u8,
    ArrayBufferTransfer = 't'  as u8,
    ArrayBufferView     = 'V'  as u8,
    SharedArrayBuffer   = 'u'  as u8,

    WasmModuleTransfer  = 'w'  as u8,
    HostObject          = '\\' as u8,
    WasmMemoryTransfer  = 'm'  as u8,

    Error               = 'r'  as u8,
}

#[repr(u8)]
enum TSONDelimiters {
    ObjectBegin = 0x00,
    ObjectEnd   = 0x01,
    ArrayBegin  = 0x02,
    ArrayEnd    = 0x03,
    String      = 0x04,
    Number      = 0x06,
    Boolean     = 0x07,
    Null        = 0x08,
}

#[repr(u8)]
enum JSONDelimiters {
    String = "\"" as u8
}

enum Loop {
    Continue,
    Stop,
}

#[repr(u8)]
enum VarInt {
    RestMSB =  0b0111_1111,
    MSB =      0b1000_0000,
}

pub trait Parser {
    type Parsed;

    fn parse(mut self) -> Self::Parsed {
        let mut keep = Loop::Continue;

        while let Loop::Continue = keep {
            val = self.next();
        }

        self.get_parsed()
    }

    fn next(&mut self) -> Loop;
    fn get_parsed(self) -> Self::Parsed;
}

struct StructuredDataParser {
    original: Vec<u8>,
    parsed: Vec<u8>,
    stack: Vec<usize>,
    index: usize,
}

impl Parser for StructuredDataParser {
    type Parsed = Vec<u8>;

    fn next(&mut self) -> Loop {
        if self.index == self.original.len() {
            Loop::Stop
        }

        self.parse_next();

        Loop::Continue
    }

    fn get_parsed(self) -> Self::Parsed {
        self.parsed
    }
}

impl StructuredDataParser {
    fn parse_next(&mut self) {
        match self.read_next_control() {
            V13::Version => self.verify_version(),

            V13::Utf8String => self.read_utf8_string(),
            V13::OneByteString => self.read_one_byte_string(),
            V13::TwoByteString => self.read_two_byte_string(),

            V13::BeginJSObject => self.read_begin_js_object(),
            V13::EndJSObject => self.read_end_js_object(),
            V13::BeginSparseJSArray => self.read_begin_sparse_js_array(),
            V13::EndSparseJSArray => self.read_end_sparse_js_array(),
            V13::BeginDenseJSArray => self.read_begin_dense_js_array(),
            V13::EndDenseJSArray => self.read_end_dense_js_array(),

            V13::Int32 => self.read_int_32(),
            V13::Uint32 => self.read_uint_32(),
            V13::Double => self.read_double(),
            V13::BigInt => self.read_bigint(),

            V13::Undefined => self.read_undefined(),
            V13::Null => self.read_null(),
            V13::True => self.read_true(),
            V13::False => self.read_false(),

            V13::Padding => self.ignore_padding(),
            V13::VerifyObjectCount => self.ignore_verify_object_count(),

            V13::Date => self.panic_date(),
            V13::RegExp => self.panic_regexp(),

            V13::BeginJSMap => self.panic_begin_js_map(),
            V13::EndJSMap => self.panic_end_js_map(),
            V13::BeginJSSet => self.panic_begin_js_set(),
            V13::EndJSSet => self.panic_end_js_set(),

            V13::TheHole => self.panic_the_hole(),
            V13::ObjectReference => self.panic_object_reference(),

            V13::TrueObject => self.panic_true_object(),
            V13::FalseObject => self.panic_false_object(),
            V13::NumberObject => self.panic_number_object(),
            V13::BigIntObject => self.panic_bigint_object(),
            V13::StringObject => self.panic_string_object(),

            V13::ArrayBuffer => self.panic_array_buffer(),
            V13::ArrayBufferTransfer => self.panic_array_buffer_transfer(),
            V13::ArrayBufferView => self.panic_array_buffer_view(),
            V13::SharedArrayBuffer => self.panic_shared_array_buffer(),

            V13::WasmModuleTransfer => self.panic_wasm_module_transfer(),
            V13::HostObject => self.panic_host_object(),
            V13::WasmMemoryTransfer => self.panic_wasm_memory_transfer(),

            V13::Error => self.panic_error(),
        }
    }
    fn read_next_control(&mut self) -> V13 {
        self.read_next() as V13
    }
    fn read_next(&mut self) -> u8 {
        let prev = self.index;
        self.move_read_index(1);

        self.original[prev]
    }
    fn move_read_index(&mut self, n: u32) {
        self.index += n;
    }
    fn read_next_slice(&mut self, length: u32) -> &[u8] {
        let prev = self.index;
        self.move_read_index(length);
        &self.original[prev..length]
    }
    fn rewrite_slice(&mut self, start: usize, slice: &[u8]) {
        for (i, val) in slice.iter().enumerate() {
            self.parsed[start + i] = *val;
        }
    }
}

impl StructuredDataParser {
    fn write_delimiter(&mut self, del: TSONDelimiters) {
        self.parsed.push(del as u8);
    }
    fn write_length(&mut self, length: u32) {
        self.parsed.push(*length.to_le_bytes())
    }
    fn update_length(&mut self, index: usize, slice: &[u8]) {
        self.rewrite_slice(index, slice);
    }
    fn write_slice(&mut self, slice: &[u8]) {
        self.parsed.extend_from_slice(slice);
    }
    fn write_string(&mut self, slice: &[u8]) {
        self.write_delimiter(TSONDelimiters::String);
        self.write_length(slice.len() as u32);
        self.write_slice(slice);
    }
    fn write_begin(&mut self, delimiter: TSONDelimiters) {
        self.write_delimiter(delimiter);
        self.write_length(0);
        self.stack.push(self.parsed.len());
    }
    fn write_end(&mut self, delimiter: TSONDelimiters) {
        self.write_delimiter(delimiter);
        let prev = self.stack.pop().unwrap();
        let length = self.parsed.len() - prev;
        self.update_length(prev, *length.to_le_bytes());
    }
}

impl StructuredDataParser {
    fn verify_version(&mut self) {
        let version = self.read_uint_32();
        if version != SUPPORTED_VERSION {
            panic!(
                "Unsupported V8 serialization version! Expected: {}, found: {}",
                SUPPORTED_VERSION,
                version
            );
        }
    }

    fn read_utf8_string(&mut self) {
        let length = self.read_uint_32();
        let slice = self.read_next_slice(length);

        let string = String::from_utf8(slice.to_vec())
            .expect("Unexpected error while parsing UTF-8 string.");

        self.write_string(string.as_bytes());
    }
    fn read_one_byte_string(&mut self) {
        let length = self.read_uint_32();
        let slice = self.read_next_slice(length);

        let string = String::from(slice).as_bytes();
        self.write_string(string);

    }
    fn read_two_byte_string(&mut self) {
        let length = self.read_uint_32();

        if length % 2 {
            panic!("Unexpected error while parsing UTF-16 string, length is odd.");
        }

        let slice = self.read_next_slice(length);
        let length = length / 2;

        let mut combined = Vec::with_capacity(length as usize);

        for i in 0..length {
            let prev: u8 = slice[i * 2 - 1];
            let next: u8 = slice[i * 2];

            let result = (prev as u16) << 8 | next as u16;
            combined.push(result);
        }

        let string = String::from_utf16(combined.as_slice())
            .expect("Unexpected error while parsing UTF-16 string.")
            .as_bytes();

        self.write_string(string)
    }

    fn read_begin_js_object(&mut self) {
        self.write_begin(TSONDelimiters::ObjectBegin);
    }
    fn read_end_js_object(&mut self) {
        self.write_end(TSONDelimiters::ObjectEnd);
        self.skip_uint_32();
    }
    fn read_begin_sparse_js_array(&mut self) {
        self.write_begin(TSONDelimiters::ArrayBegin);
        self.skip_uint_32()
    }
    fn read_end_sparse_js_array(&mut self) {
        todo!();
    }
    fn read_begin_dense_js_array(&mut self) {
        todo!();
    }
    fn read_end_dense_js_array(&mut self) {
        todo!();
    }

    fn read_int_32(&self) {
        todo!();
    }
    fn read_uint_32(&mut self) -> u32 {
        let mut result: u32 = 0;
        let mut shift = 0;
        let mut n: usize = 0;

        loop {
            let val = self.read_next();
            let rest = val & VarInt::RestMSB;
            result |= (rest as u32) << shift;

            shift += 7;
            n += 1;

            if val & VarInt::MSB == 0 || n > 4 {
                break
            }
        }

        result
    }

    fn skip_uint_32(&mut self) {
        let mut n: usize = 0;

        loop {
            let val = self.read_next();

            n += 1;

            if val & VarInt::MSB == 0 || n > 4 {
                break
            }
        }
    }
    fn read_double(&self) {
        todo!();
    }
    fn read_bigint(&self) {
        todo!();
    }

    fn read_undefined(&self) {
        todo!();
    }
    fn read_null(&self) {
        todo!();
    }
    fn read_true(&self) {
        todo!();
    }
    fn read_false(&self) {
        todo!();
    }

    fn ignore_padding(&self) {
        todo!();
    }
    fn ignore_verify_object_count(&self) {
        todo!();
    }

    fn panic_date(&self) {
        panic!("Date is a unsupported type.");
    }
    fn panic_regexp(&self) {
        panic!("RegExp is a unsupported type.");
    }

    fn panic_begin_js_map(&self) {
        panic!("Map is a unsupported type.");
    }
    fn panic_end_js_map(&self) {
        panic!("Map is a unsupported type.");
    }
    fn panic_begin_js_set(&self) {
        panic!("Set is a unsupported type.");
    }
    fn panic_end_js_set(&self) {
        panic!("Set is a unsupported type.");
    }

    fn panic_the_hole(&self) {
        panic!("Unexpected oddball: The Hole.");
    }
    fn panic_object_reference(&self) {
        panic!("Circle references are unsupported.");
    }
    fn panic_true_object(&self) {
        panic!("True Object is a unsupported type.");
    }
    fn panic_false_object(&self) {
        panic!("False Object is a unsupported type.");
    }
    fn panic_number_object(&self) {
        panic!("Number Object is a unsupported type.");
    }
    fn panic_bigint_object(&self) {
        panic!("BigInt Object is a unsupported type.");
    }
    fn panic_string_object(&self) {
        panic!("String Object is a unsupported type.");
    }

    fn panic_array_buffer(&self) {
        panic!("ArrayBuffer is a unsupported type.");
    }
    fn panic_array_buffer_transfer(&self) {
        panic!("ArrayBufferTransfer is a unsupported type.");
    }
    fn panic_array_buffer_view(&self) {
        panic!("ArrayBufferView is a unsupported type.");
    }
    fn panic_shared_array_buffer(&self) {
        panic!("SharedArrayBuffer is a unsupported type.");
    }

    fn panic_wasm_module_transfer(&self) {
        panic!("Wasm values are unsupported.");
    }
    fn panic_host_object(&self) {
        panic!("Native values are unsupported.");
    }
    fn panic_wasm_memory_transfer(&self) {
        panic!("Wasm values are unsupported.");
    }

    fn panic_error(&self) {
        panic!("Error is a unsupported type.");
    }
}