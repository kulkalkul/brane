use neon::prelude::*;
use bson::Bson;
use crate::Cx;
use crate::internal::document::Document;
use crate::internal::document_array::DocumentArray;

pub trait DocumentField {
    fn parse_value(&self, cx: &mut Cx) -> Option<Bson>;
}

impl DocumentField for Handle<'_, JsValue> {
    fn parse_value(&self, cx: &mut Cx) -> Option<Bson> {
        self.check_unsupported(cx);
        self.parse_js_value(cx)
            .unwrap_or_else(|err| panic!(
                "Unexpected error while parsing js value: {}",
                err
        ))
    }
}

trait JsValueParser {
    fn check_unsupported(&self, cx: &mut Cx);
    fn parse_js_value(&self, cx: &mut Cx) -> NeonResult<Option<Bson>>;
}

impl JsValueParser for Handle<'_, JsValue> {
    fn check_unsupported(&self, cx: &mut Cx) {
        if self.is_a::<JsArrayBuffer, _>(cx) {
            panic!("Storing ArrayBuffer is not supported.");
        } else if self.is_a::<JsBuffer, _>(cx) {
            panic!("Storing Buffer is not supported.");
        } else if self.is_a::<JsError, _>(cx) {
            panic!("Storing Error is not supported.");
        } else if self.is_a::<JsFunction, _>(cx) {
            panic!("Storing Function is not supported.");
        }
    }
    fn parse_js_value(&self, cx: &mut Cx) -> NeonResult<Option<Bson>> {
        let result =
        if let Ok(object) = self.downcast::<JsObject, _>(cx) {
            Some(Bson::Document(object.parse_object(cx)?))
        } else if let Ok(array) = self.downcast::<JsArray, _>(cx) {
            Some(Bson::Array(array.parse_array(cx)))
        } else if let Ok(string) = self.downcast::<JsString, _>(cx) {
            Some(Bson::String(string.value(cx)))
        } else if let Ok(number) = self.downcast::<JsNumber, _>(cx) {
            Some(Bson::Double(number.value(cx)))
        } else if let Ok(boolean) = self.downcast::<JsBoolean, _>(cx) {
            Some(Bson::Boolean(boolean.value(cx)))
        } else if self.is_a::<JsNull, _>(cx) {
            Some(Bson::Null)
        } else {
            None
        };

        Ok(result)
    }
}