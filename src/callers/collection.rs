use neon::prelude::*;
use crate::Cx;
use crate::internal::store::{Collection};
use crate::internal::parser::{Parser, JSONParser, TSONParser};

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
