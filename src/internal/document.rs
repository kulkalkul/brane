use neon::prelude::*;
use bson::Bson;
use bson::oid::ObjectId;
use std::convert::TryInto;
use crate::Cx;
use crate::internal::object_helper::ObjectHelper;
use crate::internal::document_field::DocumentField;

const ID_KEY: &str = "_id";
const BSON_TYPE_KEY: &str = "_bsontype";
const BSON_OBJECTID_TYPE: &str = "ObjectID";
const BSON_OBJECTID_ID: &str = "id";

pub struct DocumentWithID(pub Vec<u8>, pub Vec<u8>);

pub trait Document {
    fn document(&self, cx: &mut Cx) -> NeonResult<DocumentWithID>;
    fn parse_object(&self, cx: &mut Cx) -> NeonResult<bson::Document>;
}
impl Document for Handle<'_, JsObject> {
    fn document(&self, cx: &mut Cx) -> NeonResult<DocumentWithID> {
        let id = self.get_or_create_id(cx);

        let mut document = bson::Document::new();
        document.insert(ID_KEY, id.clone());

        self.parse_object_with_document(cx, &mut document)?;

        let mut byte_array: Vec<u8> = Vec::new();
        document.to_writer(&mut byte_array);

        let id = match id {
            Bson::String(string) => string.into_bytes(),
            Bson::Double(double) => double.to_le_bytes().to_vec(),
            Bson::ObjectId(id) => id.bytes().to_vec(),
            _ => panic!("Unexpected error while parsing id as string."),
        };

        Ok(DocumentWithID(byte_array, id))
    }
    fn parse_object(&self, cx: &mut Cx) -> NeonResult<bson::Document> {
        let mut document = bson::Document::new();
        self.parse_object_with_document(cx, &mut document)?;
        Ok(document)
    }
}

trait DocumentParser {
    fn parse_object_with_document(
        &self,
        cx: &mut Cx,
        document: &mut bson::Document,
    ) -> NeonResult<()>;
    fn parse_key(cx: &mut Cx, key: Handle<JsValue>) -> String;
}
impl DocumentParser for Handle<'_, JsObject> {
    fn parse_object_with_document(
        &self,
        cx: &mut Cx,
        document: &mut bson::Document,
    ) -> NeonResult<()> {
        let keys = self.get_own_property_names(cx)?.to_vec(cx)?;

        for key in keys {
            if let Some(value) = self.get(cx, key)?.parse_value(cx) {
                let key = Self::parse_key(cx, key);
                document.insert(key, value);
            }
        }

        Ok(())
    }
    fn parse_key(cx: &mut Cx, key: Handle<JsValue>) -> String {
        key
            .to_string(cx)
            .unwrap_or_else(|err| panic!(
                "Unexpected error while parsing key into js value: {}",
                err
            ))
            .value(cx)
    }
}

trait IDHelper {
    fn get_or_create_id(&self, cx: &mut Cx) -> Bson;
    fn get_id_field<'a>(&self, cx: &mut Cx<'a>) -> Handle<'a, JsValue>;
    fn parse_id_as_object_id(cx: &mut Cx, id: Handle<JsObject>) -> Bson;
}
impl IDHelper for Handle<'_, JsObject> {
    fn get_or_create_id(&self, cx: &mut Cx) -> Bson {
        let id = self.get_id_field(cx);

        let bson = if let Ok(id) = id.downcast::<JsNumber, _>(cx) {
            Bson::Double(id.value(cx))
        } else if let Ok(id) = id.downcast::<JsString, _>(cx) {
            Bson::String(id.value(cx))
        } else if let Ok(id) = id.downcast::<JsObject, _>(cx) {
            Self::parse_id_as_object_id(cx, id)
        } else if id.is_a::<JsUndefined, _>(cx) {
            Bson::ObjectId(ObjectId::new())
        } else {
            panic!("Only number, string and ObjectID types are supported as _id.")
        };

        let undefined = cx.undefined();
        self.set(cx, "_id", undefined);

        bson
    }
    fn get_id_field<'a>(&self, cx: &mut Cx<'a>) -> Handle<'a, JsValue> {
        match self.get_value(cx, ID_KEY.to_string()) {
            Ok(value) => value,
            Err(err) => panic!("Unexpected error while getting _id field {}", err),
        }
    }
    fn parse_id_as_object_id(cx: &mut Cx, id: Handle<JsObject>) -> Bson {
        let bson_type = id.get_value_string(cx, BSON_TYPE_KEY.to_string())
            .expect("Object you provided as _id isn't a valid ObjectID.");

        if bson_type != BSON_OBJECTID_TYPE {
            panic!("Object you provided as _id isn't a valid ObjectID.");
        }

        let array = id.get_value_buffer(cx, BSON_OBJECTID_ID.to_string())
            .expect("id field of ObjectID must be a buffer type.")
            .try_into()
            .expect("id field of ObjectID is in wrong shape!");

        Bson::ObjectId(ObjectId::with_bytes(array))
    }
}