
mod controls {
    pub const BEGIN_OBJECT:         u8 = '{'  as u8;
    pub const END_OBJECT:           u8 = '}'  as u8;
    pub const BEGIN_ARRAY:          u8 = '['  as u8;
    pub const END_ARRAY:            u8 = ']'  as u8;
    pub const STRING:               u8 = '"'  as u8;
    pub const TRUE:                 u8 = 't'  as u8;
    pub const FALSE:                u8 = 'f'  as u8;
    pub const NULL:                 u8 = 'n'  as u8;
}

pub struct Extender {
    original: Vec<u8>,
    extended: Vec<u8>,
    stack: Vec<usize>,
    index: usize,
}

impl Extender {
    pub fn new(original: Vec<u8>) -> Extender {
        let stack = Vec::new();
        let extended = Vec::new();
        Extender { original, stack, extended, index: 0 }
    }
}

enum Loop {
    Continue,
    Stop,
}

impl Extender {
    pub fn extend(mut self) -> Vec<u8> {
        let mut val = Loop::Continue;
        while let Loop::Continue = val {
            val = self.next();
        }

        self.extended
    }
    fn next(&mut self) -> Loop {
        if self.index == self.original.len() {
            return Loop::Stop;
        }

        let control = self.read_control();

        self.read_next();

        // match control {
        //     controls::STRING =>       (),
        //     controls::BEGIN_OBJECT => (),
        //     controls::END_OBJECT =>   (),
        //     controls::BEGIN_ARRAY =>  (),
        //     controls::END_ARRAY =>    (),
        //     controls::TRUE =>         (),
        //     controls::FALSE =>        (),
        //     controls::NULL =>         (),
        //     _ => panic!("Unknown symbol in JSON data!"),
        // };

        Loop::Continue
    }

    fn read_control(&mut self) -> u8 {
        self.read_next()
    }
    fn skip_by(&mut self, n: usize) {
        let prev = self.index;
        self.index += n;
        self.extend_range(prev, self.index);
    }
    fn extend_range(&mut self, begin: usize, end: usize) {
        self.extended.extend_from_slice(&self.original[begin..end]);
    }
    fn read_next(&mut self) -> u8 {
        let prev = self.index;
        self.index += 1;

        let val = self.original[prev];

        self.extended.push(val);

        val
    }
}

impl Extender {
}