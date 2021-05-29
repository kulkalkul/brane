use crate::internal::parser::Parser;
use crate::internal::parser::delimiters::tson_delimiters;
use crate::internal::parser::value_cursor::ValueCursor;
use std::convert::TryInto;
use std::{str, mem};

pub enum EqualityValue {
    String(Vec<u8>),
    Number(f64),
    True,
    False,
    Null
}
pub enum ComparisonValue {
    String(Vec<u8>),
    Number(f64),
}
pub type ArrayValue = Vec<EqualityValue>;
pub enum Operation {
    Eq(EqualityValue),
    Ne(EqualityValue),
    Lt(ComparisonValue),
    Lte(ComparisonValue),
    Gt(ComparisonValue),
    Gte(ComparisonValue),
    In(ArrayValue),
    Nin(ArrayValue),
}
pub struct NamespacedOperation {
    pub namespace: Vec<Vec<u8>>,
    pub operation: Operation,
}
pub enum LogicalOperation {
    No(Vec<NamespacedOperation>),
    And(Vec<LogicalOperation>),
    Or(Vec<LogicalOperation>),
}

enum InObject {
    No,
    Yes,
}

enum QueryType {
    Operation,
    Logic,
}

pub struct QueryParser {
    tson: Vec<u8>,
}

impl QueryParser {
    pub fn new(tson: Vec<u8>) -> QueryParser {
        let begin = 5; // without object_begin
        let end = tson.len() - 1; // without object_end

        Self::from_objectless(tson[begin..end].to_vec())
    }
    fn from_objectless(tson: Vec<u8>) -> QueryParser {
        QueryParser { tson }
    }
}
impl QueryParser {
    pub fn parse(self) -> LogicalOperation {
        let decider = Decider::new(self.tson);
        let (tson, query_type) = decider.parse();

        match query_type {
            QueryType::Operation => LogicalOperation::No(OperationParser::new(tson).parse()),
            QueryType::Logic => LogicParser::new(tson).parse(),
        }
    }
}

struct Decider {
    cursor: ValueCursor,
    result: QueryType,
}

impl Decider {
    fn new(tson: Vec<u8>) -> Decider {
        Decider {
            cursor: ValueCursor::new(tson),
            result: QueryType::Operation,
        }
    }
}

impl Parser for Decider {
    type Parsed = (Vec<u8>, QueryType);

    fn get_index(&self) -> usize {
        self.cursor.get_index()
    }
    fn get_original_len(&self) -> usize {
        self.cursor.get_value_ref().len()
    }
    fn get_parsed(self) -> Self::Parsed {
        (self.cursor.get_value(), self.result)
    }
    fn parse_next(&mut self) {
        match self.cursor.read_next() {
            tson_delimiters::STRING => self.decide(),
            tson_delimiters::OBJECT_BEGIN => self.skip_collection(),
            tson_delimiters::ARRAY_BEGIN => self.skip_collection(),
            tson_delimiters::NUMBER => self.skip_number(),
            tson_delimiters::SEPARATOR => (),
            _ => self.skip_single(),
        }
    }
}

impl Decider {
    fn decide(&mut self) {
        let string = self.read_string();

        let slice = string.as_slice();

        let slice = unsafe {
            str::from_utf8_unchecked(slice)
        };

        match slice {
            "$or" => self.logic(),
            "$and" => self.logic(),
            "$not" => self.logic(),
            _ => {
                self.cursor.skip_next();
                match self.cursor.read_next() {
                    tson_delimiters::STRING => self.skip_string(),
                    tson_delimiters::OBJECT_BEGIN => self.skip_collection(),
                    tson_delimiters::ARRAY_BEGIN => self.skip_collection(),
                    tson_delimiters::NUMBER => self.skip_number(),
                    val => panic!("Unexpected error at {}!", val),
                }
            }
        }
    }
    fn logic(&mut self) {
        self.cursor.skip_rest();
        self.result = QueryType::Logic;
    }
    fn skip_string(&mut self) {
        let len = self.read_length();
        self.cursor.skip_by(len as usize);
    }
    fn skip_collection(&mut self) {
        let len = self.read_length() + 1; // inclusive collection end
        self.cursor.skip_by(len as usize);
    }
    fn skip_number(&mut self) {
        self.cursor.skip_by(8);
    }
    fn skip_single(&mut self) {
        self.cursor.skip_next();
    }
}

impl Decider {
    fn read_string(&mut self) -> Vec<u8> {
        let length = self.read_length();
        self.cursor.read_by(length as usize).to_vec()
    }
    fn read_length(&mut self) -> u32 {
        let slice = self.cursor.read_by(4);
        u32::from_le_bytes(slice.try_into().unwrap())
    }
}

struct LogicParser {
    cursor: ValueCursor,
    operations: Vec<LogicalOperation>,
}

impl LogicParser {
    fn new(tson: Vec<u8>) -> LogicParser {
        LogicParser {
            cursor: ValueCursor::new(tson),
            operations: Vec::new(),
        }
    }
}

impl Parser for LogicParser {
    type Parsed = LogicalOperation;
    fn get_index(&self) -> usize {
        self.cursor.get_index()
    }
    fn get_original_len(&self) -> usize {
        self.cursor.get_value_ref().len()
    }
    fn get_parsed(self) -> LogicalOperation {
        let mut operations = self.operations;
        match operations.len() {
            0 => panic!("Unexpected query syntax!"),
            1 => operations.pop().unwrap(),
            _ => LogicalOperation::And(operations),
        }
    }
    fn parse_next(&mut self) {
        match self.cursor.read_next() {
            tson_delimiters::STRING => self.decide(),
            tson_delimiters::SEPARATOR => (),
            val => panic!("Unexpected query syntax! Error at {}", val),
        }
    }
}

impl LogicParser {
    fn decide(&mut self) {
        let begin = self.get_index() - 1;
        let string = self.read_string();

        let slice = string.as_slice();

        let slice = unsafe {
            str::from_utf8_unchecked(slice)
        };

        match slice {
            "$or" => self.op_or(),
            "$and" => self.op_and(),
            "$not" => todo!("$not isn't supported yet"),
            _ => {
                self.cursor.skip_next(); // skips PAIR

                let length = match self.cursor.read_next() {
                    tson_delimiters::OBJECT_BEGIN => self.read_length() + 1, // object_end inclusive
                    val => panic!("Unexpected query syntax! Error at {}", val),
                } as usize;

                self.cursor.skip_by(length);
                let end = self.get_index();

                let pair = self.cursor.read_range(begin..end).to_vec();
                let parser = QueryParser::from_objectless(pair);

                self.operations.push(parser.parse());
            },
        }
    }
    fn get_array(&mut self) -> Vec<LogicalOperation> {
        self.cursor.skip_next();

        match self.cursor.read_next() {
            tson_delimiters::ARRAY_BEGIN => (),
            val => panic!("Unexpected query syntax! Error at {}", val),
        }

        self.cursor.skip_by(4);

        let mut operations = Vec::new();

        loop {
            match self.cursor.read_next() {
                tson_delimiters::OBJECT_BEGIN => {
                    let len = self.read_length() as usize;
                    let begin = self.get_index();

                    self.cursor.skip_by(len);

                    let end = self.get_index();
                    let pair = self.cursor.read_range(begin..end).to_vec();

                    let parser = QueryParser::from_objectless(pair);
                    let logic = parser.parse();
                    operations.push(logic);
                },
                tson_delimiters::SEPARATOR => (),
                tson_delimiters::OBJECT_END => (),
                tson_delimiters::ARRAY_END => break,
                val => panic!("Unexpected query syntax! Error at {}", val),
            }
        }

        operations
    }
    fn op_and(&mut self) {
        let operations = self.get_array();
        self.operations.push(LogicalOperation::And(operations));
    }
    fn op_or(&mut self) {
        let operations = self.get_array();
        self.operations.push(LogicalOperation::Or(operations));
    }
}

impl LogicParser {
    fn read_string(&mut self) -> Vec<u8> {
        let length = self.read_length();
        self.cursor.read_by(length as usize).to_vec()
    }
    fn read_length(&mut self) -> u32 {
        let slice = self.cursor.read_by(4);
        u32::from_le_bytes(slice.try_into().unwrap())
    }
}

struct OperationParser {
    cursor: ValueCursor,
    in_object: InObject,
    key: Option<Vec<u8>>,
    operations: Vec<NamespacedOperation>,
}

impl OperationParser {
    fn new(tson: Vec<u8>) -> OperationParser {
        OperationParser {
            cursor: ValueCursor::new(tson),
            in_object: InObject::No,
            key: None,
            operations: Vec::new(),
        }
    }
}

impl Parser for OperationParser {
    type Parsed = Vec<NamespacedOperation>;
    fn get_index(&self) -> usize {
        self.cursor.get_index()
    }
    fn get_original_len(&self) -> usize {
        self.cursor.get_value_ref().len()
    }
    fn get_parsed(self) -> Vec<NamespacedOperation> {
        self.operations
    }
    fn parse_next(&mut self) {
        match self.cursor.read_next() {
            tson_delimiters::OBJECT_END => self.end_object(),
            tson_delimiters::STRING => self.write_key_or_operation(),
            tson_delimiters::SEPARATOR => (),
            tson_delimiters::ARRAY_END => (),
            v => {
                println!("{:?}", self.cursor.get_value_ref());
                println!("{}", self.get_index());
                println!("val = {}, {}", v, v as char);
                panic!("Unexpected error while parsing!");
            },
        }
    }
}

impl OperationParser {
    fn begin_object(&mut self) {
        if let InObject::Yes = self.in_object {
            panic!("Invalid query syntax, use dot notation for nested queries.");
        }
        self.cursor.skip_by(4);
        self.in_object = InObject::Yes;
    }
    fn end_object(&mut self) {
        self.key = None;
        if let InObject::Yes = self.in_object {
            self.in_object = InObject::No;
        }
    }
    fn write_key_or_operation(&mut self) {
        let string = self.read_string();
        let slice = string.as_slice();

        // Value passed from JSON.stringify is UTF-8, so no need to check.
        let slice = unsafe {
            str::from_utf8_unchecked(slice)
        };

        self.cursor.skip_next();

        match slice {
            "$eq" => self.op_eq(),
            "$ne" => self.op_ne(),
            "$lt" => self.op_lt(),
            "$lte" => self.op_lte(),
            "$gt" => self.op_gt(),
            "$gte" => self.op_gte(),
            "$in" => self.op_in(),
            "$nin" => self.op_nin(),
            "$elemMatch" => todo!("$elemMatch isn't supported yet"),
            _ => self.no_op(string),
        }
    }
}

impl OperationParser {
    fn add_operation(&mut self, operation: Operation) {
        let namespace = self.parse_keys();
        self.operations.push(NamespacedOperation {
            namespace,
            operation,
        })
    }
    fn check_comparison_validity(&mut self) {
        if let InObject::No = self.in_object {
            panic!("Comparison operators cannot be used without a preceding key.");
        }
    }
    fn equality_value(&mut self) -> EqualityValue {
        match self.cursor.read_next() {
            tson_delimiters::STRING => EqualityValue::String(self.read_string().to_vec()),
            tson_delimiters::NUMBER => EqualityValue::Number(self.read_number()),
            tson_delimiters::TRUE => EqualityValue::True,
            tson_delimiters::FALSE => EqualityValue::False,
            tson_delimiters::NULL => EqualityValue::Null,
            val => panic!("Invalid query syntax! Error at: {}", val),
        }
    }
    fn comparison_value(&mut self) -> ComparisonValue {
        match self.cursor.read_next() {
            tson_delimiters::STRING => ComparisonValue::String(self.read_string().to_vec()),
            tson_delimiters::NUMBER => ComparisonValue::Number(self.read_number()),
            val => panic!("Invalid query syntax! Error at: {}", val),
        }
    }
    fn array_value(&mut self) -> ArrayValue {
        match self.cursor.read_next() {
            tson_delimiters::ARRAY_BEGIN => {
                let mut values = ArrayValue::new();
                self.cursor.skip_by(4);

                loop {
                    let value = match self.cursor.read_next() {
                        tson_delimiters::STRING => EqualityValue::String(self.read_string().to_vec()),
                        tson_delimiters::NUMBER => EqualityValue::Number(self.read_number()),
                        tson_delimiters::TRUE => EqualityValue::True,
                        tson_delimiters::FALSE => EqualityValue::False,
                        tson_delimiters::NULL => EqualityValue::Null,
                        tson_delimiters::SEPARATOR => continue,
                        tson_delimiters::ARRAY_END => break,
                        val => panic!("Invalid query syntax! Error at: {}", val),
                    };
                    values.push(value);
                }

                values
            },
            val => panic!("Invalid query syntax! Error at: {}", val),
        }
    }
    fn no_op(&mut self, string: Vec<u8>) {
        match self.in_object {
            InObject::No => {
                self.key = Some(string);
                match self.cursor.read_next() {
                    tson_delimiters::OBJECT_BEGIN => self.begin_object(),
                    _ => {
                        self.cursor.skip_reverse_by(1);
                        self.in_object = InObject::Yes;
                        self.op_eq();
                        self.end_object();
                    }
                }
            },
            InObject::Yes => panic!("Invalid query syntax, use dot notation for nested queries."),
        }
    }
    fn op_eq(&mut self) {
        self.check_comparison_validity();
        let value = self.equality_value();
        self.add_operation(Operation::Eq(value));
    }
    fn op_ne(&mut self) {
        self.check_comparison_validity();
        let value = self.equality_value();
        self.add_operation(Operation::Ne(value));
    }
    fn op_lt(&mut self) {
        self.check_comparison_validity();
        let value = self.comparison_value();
        self.add_operation(Operation::Lt(value));
    }
    fn op_lte(&mut self) {
        self.check_comparison_validity();
        let value = self.comparison_value();
        self.add_operation(Operation::Lte(value));
    }
    fn op_gt(&mut self) {
        self.check_comparison_validity();
        let value = self.comparison_value();
        self.add_operation(Operation::Gt(value));
    }
    fn op_gte(&mut self) {
        self.check_comparison_validity();
        let value = self.comparison_value();
        self.add_operation(Operation::Gte(value));
    }
    fn op_in(&mut self) {
        self.check_comparison_validity();
        let value = self.array_value();
        self.add_operation(Operation::In(value));
    }
    fn op_nin(&mut self) {
        self.check_comparison_validity();
        let value = self.array_value();
        self.add_operation(Operation::Nin(value));
    }
}

impl OperationParser {
    fn parse_keys(&mut self) -> Vec<Vec<u8>> {
        let string = self.key.as_ref().unwrap().clone();

        let mut keys = Vec::new();
        let mut accumulated = Vec::new();

        for v in string.iter() {
            let v = *v;
            if v == b'.' {
                keys.push(mem::take(&mut accumulated));
            } else {
                accumulated.push(v);
            }
        }

        keys.push(accumulated);

        keys
    }
    fn read_string(&mut self) -> Vec<u8> {
        let length = self.read_length();
        self.cursor.read_by(length as usize).to_vec()
    }
    fn read_length(&mut self) -> u32 {
        let slice = self.cursor.read_by(4);
        u32::from_le_bytes(slice.try_into().unwrap())
    }
    fn read_number(&mut self) -> f64 {
        let slice = self.cursor.read_by(8);
        f64::from_le_bytes(slice.try_into().unwrap())
    }
}