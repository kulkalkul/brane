use neon::prelude::*;
use crate::internal::database::{ Database, Collection };
use crate::Cx;
use crate::internal::parser::json_parser::JSONParser;
use crate::internal::parser::parser::Parser;
use crate::internal::parser::tson_parser::TSONParser;

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

pub fn collection_get_name(mut cx: Cx) -> JsResult<JsString> {
    let collection = cx.argument::<JsBox<Collection>>(0)?;

    Ok(JsString::new(&mut cx, collection.get_name()))
}

type Buffer<'a, 'b> = neon::borrow::Ref<'a, BinaryData<'b>>;

pub fn collection_insert(mut cx: Cx) -> JsResult<JsUndefined> {
    let collection = cx.argument::<JsBox<Collection>>(0)?;
    let id = cx.argument::<JsString>(1)?.value(&mut cx);
    let json = cx.argument::<JsString>(2)?.value(&mut cx);

    let parser = JSONParser::new_with_id(id.clone(), json);
    let tson = parser.parse();

    collection.insert(id.as_bytes(), tson);

    Ok(cx.undefined())
}