use std::ops::Deref;
use std::ops::DerefMut;

use libimagstore::store::FileLockEntry;

#[derive(Debug)]
pub struct Entry<'a>(FileLockEntry<'a>);

impl<'a> Deref for Entry<'a> {
    type Target = FileLockEntry<'a>;

    fn deref(&self) -> &FileLockEntry<'a> {
        &self.0
    }

}

impl<'a> DerefMut for Entry<'a> {

    fn deref_mut(&mut self) -> &mut FileLockEntry<'a> {
        &mut self.0
    }

}

impl<'a> Entry<'a> {

    pub fn new(fle: FileLockEntry<'a>) -> Entry<'a> {
        Entry(fle)
    }

}

impl<'a> Into<FileLockEntry<'a>> for Entry<'a> {

    fn into(self) -> FileLockEntry<'a> {
        self.0
    }

}

impl<'a> From<FileLockEntry<'a>> for Entry<'a> {

    fn from(fle: FileLockEntry<'a>) -> Entry<'a> {
        Entry::new(fle)
    }

}


