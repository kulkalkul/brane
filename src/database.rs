use neon::prelude::*;
use crate::internal::database::{ Database, Collection };
use crate::internal::parser::{Parser, KeyValuePair};

pub fn database_new(mut cx: FunctionContext) -> JsResult<JsBox<Database>> {
    let path = cx.argument::<JsString>(0)?.value(&mut cx);

    let db = Database::new(path);

    Ok(cx.boxed(db))
}
pub fn database_collection(mut cx: FunctionContext) -> JsResult<JsBox<Collection>> {
    let db = cx.argument::<JsBox<Database>>(0)?;
    let name = cx.argument::<JsString>(1)?.value(&mut cx);

    let collection = db.collection(name);

    Ok(cx.boxed(collection))
}
pub fn database_debug(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let db = cx.argument::<JsBox<Database>>(0)?;

    db.debug();

    Ok(cx.undefined())
}

pub fn collection_get_name(mut cx: FunctionContext) -> JsResult<JsString> {
    let collection = cx.argument::<JsBox<Collection>>(0)?;

    Ok(JsString::new(&mut cx, collection.get_name()))
}
pub fn collection_insert(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let collection = cx.argument::<JsBox<Collection>>(0)?;
    let object = cx.argument::<JsObject>(1)?;

    let parser = Parser::new(object);
    collection.insert(Parser::parse(&parser, &mut cx)?);

    Ok(cx.undefined())
}
