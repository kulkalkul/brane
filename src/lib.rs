pub mod internal;
pub mod callers;

pub type Cx<'a> = FunctionContext<'a>;

use neon::prelude::*;
use callers::*;

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("database_new", database_new)?;
    cx.export_function("database_collection", database_collection)?;

    cx.export_function("collection_get_name", collection_get_name)?;
    cx.export_function("collection_insert", collection_insert)?;

    Ok(())
}
