use neon::prelude::*;
use crate::Cx;
use crate::internal::store::{ Database, Collection };

pub fn database_new(mut cx: Cx) -> JsResult<JsBox<Database>> {
    let path = cx.argument::<JsString>(0)?.value(&mut cx);

    let db = Database::new(path);

    Ok(cx.boxed(db))
}
pub fn database_collection(mut cx: Cx) -> JsResult<JsBox<Collection>> {
    let db = cx.argument::<JsBox<Database>>(0)?;
    let name = cx.argument::<JsString>(1)?.value(&mut cx);

    let collection = db.collection(name);

    Ok(cx.boxed(collection))
}
pub fn database_debug(mut cx: Cx) -> JsResult<JsUndefined> {
    let db = cx.argument::<JsBox<Database>>(0)?;

    db.debug();

    Ok(cx.undefined())
}