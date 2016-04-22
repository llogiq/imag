use libimagstore::store::Store;
use libimagstore::storeid::IntoStoreId;
use chrono::DateTime;
use chrono::NaiveDateTime;
use chrono::offset::local::Local;
use chrono::Timelike;
use chrono::Datelike;

use entry::Entry;
use diaryid::DiaryId;
use error::DiaryError as DE;
use error::DiaryErrorKind as DEK;
use result::Result;
use iter::DiaryEntryIterator;
use is_in_diary::IsInDiary;

pub struct Diary<'a> {
    store: &'a Store,
    name: &'a str,
}

impl<'a> Diary<'a> {

    pub fn open(store: &'a Store, name: &'a str) -> Diary<'a> {
        Diary {
            store: store,
            name: name,
        }
    }

    // create or get a new entry for today
    pub fn new_entry_today(&self) -> Result<Entry> {
        let dt = Local::now();
        let ndt = dt.naive_local();

        // Currenty we only have support for per-day entries
        let id = DiaryId::new(String::from(self.name), ndt.year(), ndt.month(), ndt.day(), 0, 0);

        self.store
            .retrieve(id.into_storeid())
            .map(|fle| Entry::new(fle))
            .map_err(|e| DE::new(DEK::StoreWriteError, Some(Box::new(e))))
    }

    // Get an iterator for iterating over all entries
    pub fn entries(&self) -> Result<DiaryEntryIterator<'a>> {
        self.store
            .retrieve_for_module("diary")
            .map(|iter| DiaryEntryIterator::new(self.name, self.store, iter))
            .map_err(|e| DE::new(DEK::StoreReadError, Some(Box::new(e))))
    }

    pub fn delete_entry(&self, entry: Entry) -> Result<()> {
        if !entry.is_in_diary(self.name) {
            return Err(DE::new(DEK::EntryNotInDiary, None));
        }
        let id = entry.get_location().clone();
        drop(entry);

        self.store.delete(id)
            .map_err(|e| DE::new(DEK::StoreWriteError, Some(Box::new(e))))
    }

}

