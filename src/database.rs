use neon::prelude::*;
use crate::internal::database::{ Database, Collection };
use crate::Cx;
use crate::internal::parser::Extender;

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
    let buf = cx.argument::<JsArrayBuffer>(1)?;

    let vec = cx.borrow(&buf, |buf: Buffer| buf.as_slice::<u8>().to_vec());

    let extender = Extender::new(vec);
    extender.extend();

    Ok(cx.undefined())
}