use neon::prelude::*;
use rocksdb::{DB, ReadOptions, IteratorMode, Direction, WriteBatch};
use std::sync::Arc;
use crate::internal::byte_helper::concat_bytes;
use crate::internal::parser::tson_parser::TSONParser;
use crate::internal::parser::parser::Parser;

pub mod controls {
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

        start.push_str(controls::NS_BEGIN);
        end.push_str(controls::NS_END);

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

pub struct Collection {
    db: Arc<DB>,
    name: String,
}

impl Collection {
    pub fn new(db: Arc<DB>, name: String) -> Collection {
        Collection { db, name }
    }
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }
    pub fn insert<K, T>(&self, id: K, value: T)
    where
        K: AsRef<[u8]>,
        T: AsRef<[u8]>,
    {
        let mut batch = WriteBatch::default();
        batch.put(self.values_key(id), value);
        self.db.write(batch);
    }
    fn values_key<T: AsRef<[u8]>>(&self, id: T) -> Vec<u8> {
        concat_bytes(vec![
            self.name.as_bytes(),
            controls::NS_BEGIN.as_bytes(),
            controls::VALUES.as_bytes(),
            id.as_ref(),
        ])
    }
    fn index_key(&self, index: String, id: String) -> Vec<u8> {
        concat_bytes(vec![
            self.name.as_bytes(),
            controls::NS_BEGIN.as_bytes(),
            controls::INDEX.as_bytes(),
            index.as_ref(),
            id.as_ref(),
        ])
    }
}

impl Finalize for Collection {}
