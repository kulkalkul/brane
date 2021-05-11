use neon::prelude::*;
use neon::handle::Managed;
use neon::result::Throw;
use crate::Cx;

// IntelliJ-Rust can't guess the type correct, so necessary.
type Buffer<'a, 'b> = neon::borrow::Ref<'a, BinaryData<'b>>;

pub trait ObjectHelper {
    fn get_value<'a, T: Managed + Value>(
        &self,
        cx: &mut Cx<'a>,
        key: String
    ) -> NeonResult<Handle<'a, T>>;
    fn get_value_string(&self, cx: &mut Cx, key: String) -> NeonResult<String>;
    fn get_value_number(&self, cx: &mut Cx, key: String) -> NeonResult<f64>;
    fn get_value_buffer<'a>(&self, cx: &mut Cx<'a>, key: String) -> NeonResult<Vec<u8>>;
}
impl ObjectHelper for Handle<'_, JsObject> {
    fn get_value<'a, T: Managed + Value>(
        &self,
        cx: &mut Cx<'a>,
        key: String
    ) -> NeonResult<Handle<'a, T>> {
        let key = cx.string(key);
        match self.get(cx, key)?.downcast::<T, _>(cx) {
            Ok(value) => Ok(value),
            Err(_) => Err(Throw),
        }
    }
    fn get_value_string(&self, cx: &mut Cx, key: String) -> NeonResult<String> {
        Ok(self.get_value::<JsString>(cx, key)?.value(cx))
    }
    fn get_value_number(&self, cx: &mut Cx, key: String) -> NeonResult<f64> {
        Ok(self.get_value::<JsNumber>(cx, key)?.value(cx))
    }
    fn get_value_buffer(&self, cx: &mut Cx, key: String) -> NeonResult<Vec<u8>> {
        let buffer = self.get_value::<JsBuffer>(cx, key)?;
        let buffer = cx.borrow(&buffer, |buf: Buffer| buf.as_slice::<u8>().to_owned());
        Ok(buffer)
    }
}