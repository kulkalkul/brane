
pub mod json_delimiters {
    pub const OBJECT_BEGIN: u8 = b'{';
    pub const OBJECT_END:   u8 = b'}';
    pub const ARRAY_BEGIN:  u8 = b'[';
    pub const ARRAY_END:    u8 = b']';
    pub const STRING:       u8 = b'"';
    pub const TRUE:         u8 = b't';
    pub const FALSE:        u8 = b'f';
    pub const NULL:         u8 = b'n';
    pub const PAIR:         u8 = b':';
    pub const SEPARATOR:    u8 = b',';
}

pub mod tson_delimiters {
    pub const OBJECT_BEGIN: u8 = 0x00;
    pub const OBJECT_END:   u8 = 0x01;
    pub const ARRAY_BEGIN:  u8 = 0x02;
    pub const ARRAY_END:    u8 = 0x03;
    pub const STRING:       u8 = 0x04;
    pub const NUMBER:       u8 = 0x05;
    pub const TRUE:         u8 = 0x06;
    pub const FALSE:        u8 = 0x07;
    pub const NULL:         u8 = 0x08;
    pub const PAIR:         u8 = 0x09;
    pub const SEPARATOR:    u8 = 0x0A;
}