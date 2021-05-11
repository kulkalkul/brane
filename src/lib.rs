mod internal {
    pub mod database;
    pub mod byte_helper;
    pub mod object_helper;
    pub mod document;
    pub mod document_field;
    pub mod document_array;
}
mod database;

pub type Cx<'a> = FunctionContext<'a>;

use neon::prelude::*;
use database::*;

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("database_new", database_new)?;
    cx.export_function("database_collection", database_collection)?;
    cx.export_function("database_debug", database_debug)?;

    cx.export_function("collection_get_name", collection_get_name)?;
    cx.export_function("collection_insert", collection_insert)?;

    Ok(())
}
