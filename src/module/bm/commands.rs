use std::vec::IntoIter;

use clap::ArgMatches;
use regex::Regex;

use module::{CommandEnv, CommandResult, Module, ModuleError};
use module::bm::header::{build_header, get_tags_from_header};
use runtime::Runtime;
use storage::StorageError;
use storage::file::File;
use storage::json::parser::JsonHeaderParser;
use storage::parser::Parser;
use ui::file::{FilePrinter, TablePrinter};

pub fn add_command(module: &Module, env: CommandEnv) -> CommandResult {
    use url::Url;
    use module::helpers::utils::cli::get_tags;

    let url = env.matches.value_of("url").unwrap();

    if let Err(e) = Url::parse(url) {
        info!("Not an URL: '{}'", url);
        info!("  this will turn into an hard error before 0.1.0");
        debug!("URL parsing error: {:?}", e);
    }

    let tags = get_tags(env.rt, env.matches);
    info!("Adding url '{}' with tags '{:?}'", url, tags);

    let header  = build_header(&String::from(url), &tags);
    let file    = File::new_with_header(module, header);
    let parser  = Parser::new(JsonHeaderParser::new(None));
    let putres  = env.bk.put_file(file, &parser);

    putres.map_err(|sberr| {
        let mut err = ModuleError::new("Storage Backend Error");
        err.caused_by = Some(Box::new(sberr));
        err
    })
}

pub fn list_command(module: &Module, env: CommandEnv) -> CommandResult {
    let printer = TablePrinter::new(env.rt.is_verbose(), env.rt.is_debugging());
    let files   = get_filtered_files_from_backend(module, &env);

    debug!("Printing files now");
    files.map(|f| printer.print_files(f));

    Ok(())
}

pub fn remove_command(module: &Module, env: CommandEnv) -> CommandResult {
    fn remove_by_id(module: &Module, env: CommandEnv, id: String, checked: bool) -> CommandResult {
        let parser = Parser::new(JsonHeaderParser::new(None));
        let file = env.bk
            .get_file_by_id(module, &id.into(), &parser)
            .unwrap_or({
                info!("No files found");
                return Ok(())
            });

        debug!("Remove file: {:?}", file);

        if let Err(e) = env.bk.remove_file(module, file, checked) {
            debug!("Remove failed");
            let mut err = ModuleError::new("Removing file failed");
            err.caused_by = Some(Box::new(e));
            Err(err)
        } else {
            info!("Remove worked");
            Ok(())
        }
    }

    fn remove_by_filtering(module: &Module, env: CommandEnv, checked: bool) -> CommandResult {
        get_filtered_files_from_backend(module, &env).and_then(|files| {
            let nfiles = files.len();
            info!("Removing {} Files", nfiles);

            let errs = files.map(|file| {
                    debug!("Remove file: {:?}", file);
                    env.bk.remove_file(module, file, checked)
                })
                .filter(|e| e.is_err())
                .map(|e| {
                    let err = e.err().unwrap();
                    warn!("Error occured in Filesystem operation: {}", err);
                    err
                })
                .collect::<Vec<StorageError>>();

            if errs.len() != 0 {
                warn!("{} Errors occured while removing {} files", errs.len(), nfiles);
                let moderr = ModuleError::new("File removal failed");

                // TODO : Collect StorageErrors

                Err(moderr)
            } else {
                Ok(())
            }
        })
    }

    let checked : bool = run_removal_checking(&env);
    debug!("Checked mode: {}", checked);

    if let Some(id) = get_id(env.rt, env.matches) {
        debug!("Remove by id: {}", id);
        remove_by_id(module, env, id, checked)
    } else {
        debug!("Remove more than one file");
        remove_by_filtering(module, env, checked)
    }
}

/*
 *
 * Private helpers
 *
 */

fn get_filtered_files_from_backend<'a>(module: &'a Module,
                                       env: &CommandEnv)
    -> Result<IntoIter<File<'a>>, ModuleError>
{
    use module::helpers::utils::cli::get_tags;

    fn check_tags(tags: &Vec<String>, file: &File) -> bool {
        if tags.len() != 0 {
            debug!("Checking tags of: {:?}", file.id());
            get_tags_from_header(&file.header())
                .iter()
                .any(|t| tags.contains(t))
        } else {
            true
        }
    }

    let parser = Parser::new(JsonHeaderParser::new(None));
    let tags = get_tags(env.rt, env.matches);
    debug!("Tags: {:?}", tags);
    env.bk
        .iter_files(module, &parser)
        .map(|files| {
            files.filter(|file| {
                debug!("Backend returns file: {:?}", file);
                check_tags(&tags, file)
            }).filter(|file| {
                debug!("Checking matches of: {:?}", file.id());
                get_matcher(env.rt, env.matches)
                    .map(|r| file.matches_with(&r))
                    .unwrap_or(true)
            })
            .collect::<Vec<File>>()
            .into_iter()
        }).map_err(|e| {
            debug!("Error from Backend: {:?}", e);
            let mut merr = ModuleError::new("Could not filter files");
            merr.caused_by = Some(Box::new(e));
            merr
        })
}

fn get_matcher<'a>(rt: &Runtime, sub: &ArgMatches<'a, 'a>) -> Option<Regex> {
    debug!("Fetching matcher from commandline");
    if let Some(s) = sub.value_of("match") {
        if let Ok(r) = Regex::new(s) {
            return Some(r)
        } else {
            error!("Regex error, continuing without regex");
        }
    }
    None

}

fn get_id<'a>(rt: &Runtime, sub: &ArgMatches<'a, 'a>) -> Option<String> {
    debug!("Fetching id from commandline");
    sub.value_of("id").and_then(|s| Some(String::from(s)))
}

/*
 * Checks whether the commandline call was set to run the removal "checked",
 * so if another entry from the store refers to this ID, do not remove the file.
 */
fn run_removal_checking(env: &CommandEnv) -> bool {
    env.matches.is_present("check")
}