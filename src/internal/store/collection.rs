use neon::prelude::*;
use rocksdb::{DB, WriteBatch};
use std::sync::Arc;
use crate::internal::byte_helper::concat_bytes;
use crate::internal::store::key_controls;

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
            key_controls::NS_BEGIN.as_bytes(),
            key_controls::VALUES.as_bytes(),
            id.as_ref(),
        ])
    }
    fn index_key(&self, index: String, id: String) -> Vec<u8> {
        concat_bytes(vec![
            self.name.as_bytes(),
            key_controls::NS_BEGIN.as_bytes(),
            key_controls::INDEX.as_bytes(),
            index.as_ref(),
            id.as_ref(),
        ])
    }
}

impl Finalize for Collection {}