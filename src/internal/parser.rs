use neon::prelude::*;
use uuid::Uuid;
use crate::internal::database::{controls, types};
use crate::internal::byte_helper::concat_bytes;

enum JsValues<'a> {
    Object(Handle<'a, JsObject>),
    Array(Handle<'a, JsArray>),
    String(Handle<'a, JsString>),
    Number(Handle<'a, JsNumber>),
    Boolean(Handle<'a, JsBoolean>),
    Null,
    Undefined,
}

pub struct Grouped {
    pub index: KeyValuePair,
    pub values: Vec<KeyValuePair>,
}
pub struct KeyValuePair(pub String, pub Vec<u8>);

mod parsers {
    use super::*;
    use neon::macro_internal::runtime::napi::date::value;

    pub enum JsPairRecursive<'a> {
        No(KeyValuePair),
        Object(Handle<'a, JsObject>, KeyValuePair, String),
        Array(Handle<'a, JsArray>, KeyValuePair, String),
    }

    pub struct JsObjectParser<'a> {
        object: Handle<'a, JsObject>,
        db_key: String,
    }
    impl<'a> JsObjectParser<'a> {
        pub fn new(object: Handle<JsObject>, db_key: String) -> JsObjectParser {
            JsObjectParser { object, db_key }
        }
        pub fn get_value_parser<'b>(
            &self,
            cx: &mut FunctionContext<'b>,
            key: Handle<'b, JsValue>
        ) -> NeonResult<JsValueParser<'b>> {
            let value = self.object.get(cx, key)?;
            Ok(JsValueParser::new(self.db_key.clone(), Self::parse_key(cx, key), value))
        }
        fn parse_key(cx: &mut FunctionContext, key: Handle<JsValue>) -> String {
            match key.to_string(cx) {
                Ok(key) => key.value(cx),
                Err(err) => panic!("Unexpected error while parsing key: {}", err),
            }
        }
    }

    pub struct JsArrayParser {
        db_key: String,
    }

    impl JsArrayParser {
        pub fn new(db_key: String) -> JsArrayParser {
            JsArrayParser { db_key }
        }
        pub fn get_value_parser<'a>(
            &self,
            index: usize,
            value: Handle<'a, JsValue>
        ) -> NeonResult<JsValueParser<'a>> {
            Ok(JsValueParser::new(self.db_key.clone(), index.to_string(), value))
        }
    }

    pub struct JsValueParser<'a> {
        db_key: Option<String>,
        key: String,
        value: Handle<'a, JsValue>,
    }
    impl<'a> JsValueParser<'a> {
        fn new(
            db_key: String,
            key: String,
            value: Handle<'a, JsValue>
        ) -> JsValueParser<'a> {
            JsValueParser {
                db_key: Some(db_key),
                key,
                value
            }
        }
        pub fn check_unsupported(&self, cx: &mut FunctionContext) {
            self.panic_if_array_buffer(cx);
            self.panic_if_buffer(cx);
            self.panic_if_error(cx);
            self.panic_if_function(cx);
        }
        pub fn parse(&mut self, cx: &mut FunctionContext) -> JsPairRecursive {
            let value = self.parse_js_value(cx);

            match value {
                JsValues::Object(object) => self.parse_object(object),
                JsValues::Array(array) => self.parse_array(array),
                JsValues::String(string) => self.parse_string(cx, string),
                JsValues::Boolean(boolean) => self.parse_boolean(cx, boolean),
                JsValues::Number(number) => self.parse_number(cx, number),
                JsValues::Null => self.parse_null(),
                JsValues::Undefined => self.parse_undefined(),
            }

        }
        fn panic_if_array_buffer(&self, cx: &mut FunctionContext) {
            if !self.value.is_a::<JsArrayBuffer, _>(cx) { return }
            panic!("Storing ArrayBuffer is not supported.");
        }
        fn panic_if_buffer(&self, cx: &mut FunctionContext) {
            if !self.value.is_a::<JsBuffer, _>(cx) { return }
            panic!("Storing Buffer is not supported.");
        }
        fn panic_if_error(&self, cx: &mut FunctionContext) {
            if !self.value.is_a::<JsError, _>(cx) { return }
            panic!("Storing Error is not supported.");
        }
        fn panic_if_function(&self, cx: &mut FunctionContext) {
            if !self.value.is_a::<JsFunction, _>(cx) { return }
            panic!("Storing Function is not supported.");
        }
        fn parse_js_value(&self, cx: &mut FunctionContext) -> JsValues<'a> {
            if let Ok(array) = self.value.downcast::<JsArray, _>(cx) {
                JsValues::Array(array)
            } else if let Ok(boolean) = self.value.downcast::<JsBoolean, _>(cx) {
                JsValues::Boolean(boolean)
            } else if let Ok(number) = self.value.downcast::<JsNumber, _>(cx) {
                JsValues::Number(number)
            } else if let Ok(object) = self.value.downcast::<JsObject, _>(cx) {
                JsValues::Object(object)
            } else if let Ok(string) = self.value.downcast::<JsString, _>(cx) {
                JsValues::String(string)
            } else if self.value.is_a::<JsNull, _>(cx) {
                JsValues::Null
            } else {
                JsValues::Undefined
            }
        }
        fn parse_object<'b>(&mut self, object: Handle<'b, JsObject>) -> JsPairRecursive<'b> {
            let (primitive, complex) = self.primitive_and_complex_db_key();
            let pair = KeyValuePair(complex, vec![types::OBJECT]);
            JsPairRecursive::Object(object, pair, primitive)
        }
        fn parse_array<'b>(&mut self, array: Handle<'b, JsArray>) -> JsPairRecursive<'b> {
            let (primitive, complex) = self.primitive_and_complex_db_key();
            let pair = KeyValuePair(complex, vec![types::ARRAY]);
            JsPairRecursive::Array(array, pair, primitive)
        }
        fn parse_string(
            &mut self,
            cx: &mut FunctionContext,
            string: Handle<JsString>
        ) -> JsPairRecursive {
            let db_key = self.primitive_db_key();
            let pair = KeyValuePair(db_key, concat_bytes(vec![
                vec![types::STRING].as_ref(),
                string.value(cx).as_bytes()
            ]));
            JsPairRecursive::No(pair)
        }
        fn parse_boolean(
            &mut self,
            cx: &mut FunctionContext,
            boolean: Handle<JsBoolean>
        ) -> JsPairRecursive {
            let db_key = self.primitive_db_key();
            let value = match boolean.value(cx) {
                false => (0 as u8).to_le_bytes(),
                true => (1 as u8).to_le_bytes(),
            };
            let pair = KeyValuePair(db_key, concat_bytes(vec![
                [types::BOOLEAN].as_ref(),
                value.as_ref()
            ]));
            JsPairRecursive::No(pair)
        }
        fn parse_number(
            &mut self,
            cx: &mut FunctionContext,
            number: Handle<JsNumber>
        ) -> JsPairRecursive {
            let db_key = self.primitive_db_key();
            let pair = KeyValuePair(db_key, concat_bytes(vec![
                [types::NUMBER].as_ref(),
                number.value(cx).to_le_bytes().as_ref()
            ]));
            JsPairRecursive::No(pair)
        }
        fn parse_null(&mut self) -> JsPairRecursive {
            let db_key = self.primitive_db_key();
            let pair = KeyValuePair(db_key, vec![types::NULL]);
            JsPairRecursive::No(pair)
        }
        fn parse_undefined(&mut self) -> JsPairRecursive {
            let db_key = self.primitive_db_key();
            let pair = KeyValuePair(db_key, vec![types::UNDEFINED]);
            JsPairRecursive::No(pair)
        }
        fn primitive_db_key(&mut self) -> String {
            self.extend_and_get_db_key(controls::PRIMITIVE)
        }
        fn complex_db_key(&mut self) -> String {
            self.extend_and_get_db_key(controls::COMPLEX)
        }
        fn primitive_and_complex_db_key(&mut self) -> (String, String) {
            let mut primitive = self.get_db_key();
            let mut complex = primitive.clone();

            let primitive = self.extend_key(primitive, controls::PRIMITIVE);
            let complex = self.extend_key(complex, controls::COMPLEX);

            (primitive, complex)
        }
        fn extend_and_get_db_key(&mut self, extension: &str) -> String {
            let mut db_key = self.get_db_key();

            self.extend_key(db_key, extension)
        }
        fn extend_key(
            &mut self,
            mut key: String,
            extension: &str
        ) -> String {
            key.push_str(extension);
            key.push_str(self.key.as_str());

            key
        }
        fn get_db_key(&mut self) -> String {
            match self.db_key.take() {
                Some(db_key) => db_key,
                None => panic!("Unexpected error.")
            }
         }
    }

}

use parsers::JsPairRecursive;
use parsers::JsObjectParser;
use parsers::JsArrayParser;

pub struct Parser<T> {
    value: T,
}
impl<T> Parser<T> {
    pub fn new(value: T) -> Parser<T> {
        Parser { value }
    }
}
impl<'a> Parser<Handle<'a, JsObject>> {
    pub fn parse(&self, cx: &mut FunctionContext) -> NeonResult<Grouped> {
        let mut result = vec![];

        let id = self.get_or_create_id(cx);
        let internal_id = Self::create_internal_id();
        Self::parse_object(cx, self.value, internal_id.clone(), &mut result);

        Ok(Grouped {
            index: KeyValuePair(id, internal_id.into_bytes()),
            values: result,
        })
    }
    fn create_internal_id() -> String {
        Uuid::new_v4().to_simple().to_string()
    }
    fn get_or_create_id(&self, cx: &mut FunctionContext) -> String {
        let id = self.value.get(cx, "_id").unwrap();

        if let Ok(id) = id.downcast::<JsNumber, _>(cx) {
            id.to_string(cx).unwrap().value(cx)
        } else if let Ok(id) = id.downcast::<JsString, _>(cx) {
            id.value(cx)
        } else if let Ok(_) = id.downcast::<JsUndefined, _>(cx) {
            let id = Uuid::new_v4().to_string();
            let value = cx.string(id.clone());
            self.value.set(cx, "_id", value);

            id
        } else {
            panic!("Only strings and numbers can be used as _id.")
        }
    }

    fn parse_object(
        cx: &mut FunctionContext,
        object: Handle<JsObject>,
        db_key: String,
        result: &mut Vec<KeyValuePair>
    ) -> NeonResult<()> {
        let names = object.get_own_property_names(cx)?.to_vec(cx)?;

        let parsed = JsObjectParser::new(object, db_key);

        for name in names {
            let mut value_parser = parsed.get_value_parser(cx, name)?;
            value_parser.check_unsupported(cx);
            let pair = value_parser.parse(cx);

            let pair = Self::handle_recursive_pair(
                cx,
                pair,
                result
            );

            result.push(pair);
        }

        Ok(())
    }
    fn parse_array(
        cx: &mut FunctionContext,
        array: Handle<JsArray>,
        db_key: String,
        result: &mut Vec<KeyValuePair>
    ) -> NeonResult<()> {
        let entries = array.to_vec(cx)?;

        let parsed = JsArrayParser::new(db_key);

        for (index, entry) in entries.into_iter().enumerate() {
            let mut value_parser = parsed.get_value_parser(index, entry)?;
            value_parser.check_unsupported(cx);
            let pair = value_parser.parse(cx);

            let pair = Self::handle_recursive_pair(
                cx,
                pair,
                result
            );

            result.push(pair);
        }

        Ok(())
    }

    fn handle_recursive_pair(
        cx: &mut FunctionContext,
        pair: JsPairRecursive,
        result: &mut Vec<KeyValuePair>,
    ) -> KeyValuePair {
        match pair {
            JsPairRecursive::No(pair) => pair,
            JsPairRecursive::Array(array, pair, db_key) => {
                pair
            },
            JsPairRecursive::Object(object, pair, db_key) => {
                Self::parse_object(cx, object, db_key, result);
                pair
            },
        }
    }
}
