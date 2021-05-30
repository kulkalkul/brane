use neon::prelude::*;
use crate::Cx;
use crate::internal::store::{ Database, Collection };
use crate::callers::{CollectionWrapper, JsBoxWrapperHelper};

impl Finalize for DatabaseWrapper {}
impl JsBoxWrapperHelper for DatabaseWrapper {}

pub struct DatabaseWrapper {
    internal: Database,
}

impl DatabaseWrapper {
    pub fn js_new(mut cx: Cx) -> JsResult<JsBox<DatabaseWrapper>> {
        let path = cx.argument::<JsString>(0)?.value(&mut cx);

        Ok(cx.boxed(DatabaseWrapper { internal: Database::new(path) }))
    }
    pub fn js_collection(mut cx: Cx) -> JsResult<JsBox<CollectionWrapper>> {
        let name = cx.argument::<JsString>(0)?.value(&mut cx);
        let database = Self::this(&mut cx);

        let collection = database.internal.collection(name);

        Ok(cx.boxed(CollectionWrapper::new(collection)))
    }
}