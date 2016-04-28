#[macro_use] extern crate log;
#[macro_use] extern crate version;
extern crate clap;

extern crate libimagdiary;
extern crate libimagentrylist;
extern crate libimagrt;
extern crate libimagstore;
extern crate libimagutil;

use std::ops::Deref;
use std::process::exit;

use libimagdiary::diary::Diary;
use libimagdiary::error::DiaryError as DE;
use libimagdiary::error::DiaryErrorKind as DEK;
use libimagentrylist::listers::core::CoreLister;
use libimagentrylist::lister::Lister;
use libimagrt::edit::Edit;
use libimagrt::runtime::Runtime;
use libimagstore::store::FileLockEntry;
use libimagstore::storeid::StoreId;
use libimagutil::trace::trace_error;

mod ui;

use ui::build_ui;

fn main() {
    let name = "imag-diary";
    let version = &version!()[..];
    let about = "Personal Diary/Diaries";
    let ui = build_ui(Runtime::get_default_cli_builder(name, version, about));
    let rt = {
        let rt = Runtime::new(ui);
        if rt.is_ok() {
            rt.unwrap()
        } else {
            println!("Could not set up Runtime");
            println!("{:?}", rt.err().unwrap());
            exit(1);
        }
    };

    rt.cli()
        .subcommand_name()
        .map(|name| {
            debug!("Call {}", name);
            match name {
                "create" => create(&rt),
                // "delete" => delete(&rt),
                // "edit" => edit(&rt),
                "list" => list(&rt),
                "diary" => diary(&rt),
                _        => {
                    debug!("Unknown command"); // More error handling
                },
            }
        });
}

fn create(rt: &Runtime) {
    let diaryname = get_diary_name(rt);
    if diaryname.is_none() {
        warn!("No diary selected. Use either the configuration file or the commandline option");
        exit(1);
    }
    let diaryname = diaryname.unwrap();

    let prevent_edit = rt.cli().subcommand_matches("create").unwrap().is_present("no-edit");

    let diary = Diary::open(rt.store(), &diaryname[..]);
    let res = diary.new_entry_today()
        .and_then(|mut entry| {
            if prevent_edit {
                debug!("Not editing new diary entry");
                Ok(())
            } else {
                debug!("Editing new diary entry");
                entry.edit_content(rt)
                    .map_err(|e| DE::new(DEK::DiaryEditError, Some(Box::new(e))))
            }
        });

    if let Err(e) = res {
        trace_error(&e);
    } else {
        info!("Ok!");
    }
}

fn list(rt: &Runtime) {
    let diaryname = get_diary_name(rt);
    if diaryname.is_none() {
        warn!("No diary selected. Use either the configuration file or the commandline option");
        exit(1);
    }
    let diaryname = diaryname.unwrap();

    fn location_to_listing_string(id: &StoreId) -> String {
        unimplemented!()
    }

    let diary = Diary::open(rt.store(), &diaryname[..]);
    diary.entries()
        .and_then(|es| {
            CoreLister::new(&|e| location_to_listing_string(e.get_location()))
                .list(es.filter_map(Result::ok).map(|e| e.into())) // TODO: Do not ignore non-ok()s
                .map_err(|e| DE::new(DEK::IOError, Some(Box::new(e))))
        })
        .map(|_| debug!("Ok"))
        .map_err(|e| trace_error(&e));
}

fn delete(rt: &Runtime) {
    unimplemented!()
}

fn edit(rt: &Runtime) {
    unimplemented!()
}

fn diary(rt: &Runtime) {
    unimplemented!()
}


fn get_diary_name(rt: &Runtime) -> Option<String> {
    use libimagdiary::config::get_default_diary_name;

    get_default_diary_name(rt)
        .or(rt.cli().value_of("diaryname").map(String::from))
}
