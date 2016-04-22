use std::path::PathBuf;

use libimagstore::store::Entry;

pub trait IsInDiary {

    fn is_in_diary(&self, name: &str) -> bool;

}

impl IsInDiary for Entry {

    fn is_in_diary(&self, name: &str) -> bool {
        self.get_location().is_in_diary(name)
    }

}

impl IsInDiary for PathBuf {

    fn is_in_diary(&self, name: &str) -> bool {
        self.parent()
            .map(|parent| parent.is_dir() && parent.to_str().map(|l| l == name).unwrap_or(false))
            .unwrap_or(false)
    }

}

