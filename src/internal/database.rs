use neon::prelude::*;
use rocksdb::{DB, Error, ReadOptions, IteratorMode, Direction, WriteBatch};
use std::sync::Arc;
use crate::internal::parser::{Parser, KeyValuePair, Grouped};
use crate::internal::byte_helper::concat_bytes;

pub mod controls {
    pub const COLLECTION:     &str = "0";
    pub const COLLECTION_END: &str = "1";
    pub const INTERNAL:       &str = "1";
    pub const DEFINED:        &str = "2";
    pub const INDEX:          &str = "3";
    pub const VALUES:         &str = "4";
    pub const NS_BEGIN:       &str = "\u{10F41F}";
    pub const COMPLEX:        &str = "\u{10F420}";
    pub const PRIMITIVE:      &str = "\u{10F421}";
    pub const NS_END:         &str = "\u{10F422}";
}
pub mod types {
    pub const OBJECT:    u8 = 0;
    pub const ARRAY:     u8 = 1;
    pub const STRING:    u8 = 2;
    pub const NUMBER:    u8 = 3;
    pub const BOOLEAN:   u8 = 4;
    pub const NULL:      u8 = 5;
    pub const UNDEFINED: u8 = 6;
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

        start.push_str(controls::COLLECTION);
        end.push_str(controls::COLLECTION_END);

        options.set_iterate_upper_bound(end.into_bytes());
        let iter = self.db.iterator_opt(IteratorMode::From(start.as_bytes(), Direction::Forward), options);

        for (key, value) in iter {
            println!(
                "{:?} = {:?}",
                String::from_utf8(key.into_vec()).unwrap(),
                String::from_utf8(value.into_vec()).unwrap()
            );
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
    pub fn insert(&self, grouped: Grouped) {
        let mut batch = WriteBatch::default();
        self.insert_internal_index(&mut batch, grouped.index);
        for pair in grouped.values {
            self.insert_defined_value(&mut batch, pair);
        }
        self.db.write(batch);
    }
    fn insert_internal_index(&self, batch: &mut WriteBatch, pair: KeyValuePair) {
        let KeyValuePair(key, value) = pair;
        let key = self.key_as_internal_index(key);
        batch.put(key, value);
    }
    fn insert_defined_value(&self, batch: &mut WriteBatch, pair: KeyValuePair) {
        let KeyValuePair(key, value) = pair;
        let key = self.key_as_value(key);
        batch.put(key, value);
    }
    fn key_as_internal_index<K: AsRef<[u8]>>(&self, key: K) -> Vec<u8> {
        concat_bytes(vec![
            self.name.as_bytes(),
            controls::COLLECTION.as_bytes(),
            controls::INTERNAL.as_bytes(),
            controls::INDEX.as_bytes(),
            key.as_ref(),
        ])
    }
    fn key_as_value<K: AsRef<[u8]>>(&self, key: K) -> Vec<u8> {
        concat_bytes(vec![
            self.name.as_bytes(),
            controls::COLLECTION.as_bytes(),
            controls::DEFINED.as_bytes(),
            controls::VALUES.as_bytes(),
            key.as_ref(),
        ])
    }
}

impl Finalize for Collection {}
