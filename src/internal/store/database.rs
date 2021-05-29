use neon::prelude::*;
use std::sync::Arc;
use rocksdb::{DB, ReadOptions, IteratorMode, Direction};
use crate::internal::parser::{ Parser, TSONParser };
use crate::internal::store::Collection;

pub mod key_controls {
    pub const NS_BEGIN:        &str = "\u{10F41F}";
    pub const NS_END:          &str = "\u{10F420}";
    pub const INDEX:           &str = "0";
    pub const VALUES:          &str = "1";
}

pub struct Database {
    db: Arc<DB>,
}

impl Database {
    pub fn new(path: String) -> Database {
        let db = match DB::open_default(path) {
            Ok(db) => Arc::new(db),
            Err(err) => panic!("Unexpected error: {}", err)
        };

        Database { db }
    }
    pub fn collection(&self, name: String) -> Collection {
        Collection::new(Arc::clone(&self.db), name)
    }
}

impl Finalize for Database {}
