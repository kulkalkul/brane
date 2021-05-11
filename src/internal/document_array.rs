use neon::prelude::*;
use bson::Bson;
use crate::internal::document_field::DocumentField;
use crate::Cx;

pub trait DocumentArray {
    fn parse_array(&self, cx: &mut Cx) -> Vec<Bson>;
}

impl DocumentArray for Handle<'_, JsArray> {
    fn parse_array(&self, cx: &mut Cx) -> Vec<Bson> {
        let values = self.to_vec(cx)
            .expect("Unexpected error while parsing Array.");

        let values = values.into_iter()
            .map(|val| val.parse_value(cx))
            .filter(|val| val.is_some())
            .map(|val| val.unwrap())
            .collect();

        values
    }
}