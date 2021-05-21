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
    pub fn debug(&self) {
        let mut options = ReadOptions::default();

        let mut start = String::from("documents");
        let mut end = String::from("documents");

        start.push_str(key_controls::NS_BEGIN);
        end.push_str(key_controls::NS_END);

        options.set_iterate_upper_bound(end.into_bytes());
        let iter = self.db.iterator_opt(IteratorMode::From(start.as_bytes(), Direction::Forward), options);

        for (key, value) in iter {
            let key = String::from_utf8(key.into_vec()).unwrap();
            let value = TSONParser::new(value.into_vec()).parse();
            let value = String::from_utf8(value).unwrap();
            println!("{:?} = {:?}", key, value);
        }
    }
}

impl Finalize for Database {}
