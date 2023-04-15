use std::{
    collections::HashMap,
    sync::{RwLockReadGuard, RwLockWriteGuard},
};

use crate::{
    implementation::rtps::{
        history_cache::RtpsWriterCacheChange, reader_proxy::WriterAssociatedReaderProxy,
        stateful_writer::RtpsStatefulWriter,
    },
    infrastructure::instance::InstanceHandle,
};

pub struct ReaderProxyListIntoIter<'a> {
    writer_lock: RwLockWriteGuard<'a, RtpsStatefulWriter>,
}

impl<'a> IntoIterator for &'a mut ReaderProxyListIntoIter<'_> {
    type Item = WriterAssociatedReaderProxy<'a>;
    type IntoIter = ReaderProxyListIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ReaderProxyListIter {
            list: self.writer_lock.matched_reader_list().into_iter(),
        }
    }
}

impl<'a> ReaderProxyListIntoIter<'a> {
    pub fn new(writer_lock: RwLockWriteGuard<'a, RtpsStatefulWriter>) -> Self {
        Self { writer_lock }
    }
}

pub struct ReaderProxyListIter<'a> {
    list: std::vec::IntoIter<WriterAssociatedReaderProxy<'a>>,
}

impl<'a> Iterator for ReaderProxyListIter<'a> {
    type Item = WriterAssociatedReaderProxy<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.list.next()
    }
}

pub struct WriterChangeListIntoIter<'a> {
    lock: RwLockReadGuard<'a, RtpsStatefulWriter>,
}

impl<'a> WriterChangeListIntoIter<'a> {
    pub fn new(lock: RwLockReadGuard<'a, RtpsStatefulWriter>) -> Self {
        Self { lock }
    }
}

impl<'a> IntoIterator for &'a WriterChangeListIntoIter<'_> {
    type Item = &'a RtpsWriterCacheChange;

    type IntoIter = std::slice::Iter<'a, RtpsWriterCacheChange>;

    fn into_iter(self) -> Self::IntoIter {
        self.lock.change_list().iter()
    }
}

pub struct PairListIntoIter<'a, T> {
    lock: RwLockReadGuard<'a, HashMap<InstanceHandle, T>>,
}

impl<'a, T> PairListIntoIter<'a, T> {
    pub fn new(lock: RwLockReadGuard<'a, HashMap<InstanceHandle, T>>) -> Self {
        Self { lock }
    }
}

impl<'a, T> IntoIterator for &'a PairListIntoIter<'_, T> {
    type Item = (&'a InstanceHandle, &'a T);
    type IntoIter = std::collections::hash_map::Iter<'a, InstanceHandle, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.lock.iter()
    }
}
