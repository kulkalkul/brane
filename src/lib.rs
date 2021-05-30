pub mod internal;
pub mod callers;

pub type Cx<'a> = FunctionContext<'a>;

use neon::prelude::*;
use callers::*;

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("databaseNew", DatabaseWrapper::js_new)?;
    cx.export_function("databaseCollection", DatabaseWrapper::js_collection)?;

    cx.export_function("collectionGetName", CollectionWrapper::js_get_name)?;
    cx.export_function("collectionInsert", CollectionWrapper::js_insert)?;
    cx.export_function("collectionQuery", CollectionWrapper::js_query)?;

    Ok(())
}
