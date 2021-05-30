use neon::prelude::*;
use crate::Cx;
use crate::internal::query::{Query};
use crate::internal::store::{Collection};
use crate::internal::parser::{Parser, JSONParser};
use crate::callers::JsBoxWrapperHelper;

pub struct CollectionWrapper {
    internal: Collection,
}

impl Finalize for CollectionWrapper {}
impl JsBoxWrapperHelper for CollectionWrapper {}

impl CollectionWrapper {
    pub fn new(collection: Collection) -> CollectionWrapper {
        CollectionWrapper { internal: collection }
    }
}

impl CollectionWrapper {
    pub fn js_get_name(mut cx: Cx) -> JsResult<JsString> {
        let collection = Self::this(&mut cx);

        let name = collection.internal.get_name();

        Ok(cx.string(name))
    }
    pub fn js_insert(mut cx: Cx) -> JsResult<JsUndefined> {
        let id = cx.argument::<JsString>(0)?.value(&mut cx);
        let json = cx.argument::<JsString>(1)?.value(&mut cx);

        let collection = Self::this(&mut cx);

        let parser = JSONParser::new_with_id(id.clone(), json);
        let tson = parser.parse();

        collection.internal.insert(id.as_bytes(), tson);

        Ok(cx.undefined())
    }
    pub fn js_query(mut cx: Cx) -> JsResult<JsUndefined> {
        let json = cx.argument::<JsString>(0)?.value(&mut cx);

        let collection = Self::this(&mut cx);
        let iterator = collection.internal.query_request_all();

        let query = Query::new(json);

        Ok(cx.undefined())
    }
}