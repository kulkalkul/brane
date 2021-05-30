use neon::prelude::*;
use crate::Cx;

pub trait JsBoxWrapperHelper {
    fn this<'a>(cx: &mut Cx<'a>) -> Handle<'a, JsBox<Self>> where Self: Sized + Send {
        let casted = cx.this()
            .downcast_or_throw::<JsBox<Self>, _>(cx)
            .or_else(|err| cx.throw_error(err.to_string()))
            .unwrap();

        casted
    }
}