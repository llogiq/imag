use std::iter::Iterator;

use runtime::Runtime;
use storage::file::File;

pub trait FilePrinter {

    fn new(verbose: bool, debug: bool) -> Self;

    /*
     * Print a single file
     */
    fn print_file(&self, &File);

    /*
     * Print a list of files
     */
    fn print_files<'a, I: Iterator<Item = File<'a>>>(&self, files: I) {
        for file in files {
            self.print_file(&file);
        }
    }

}

struct DebugPrinter {
    debug: bool,
}

impl FilePrinter for DebugPrinter {

    fn new(verbose: bool, debug: bool) -> DebugPrinter {
        DebugPrinter {
            debug: debug,
        }
    }

    fn print_file(&self, f: &File) {
        if self.debug {
            debug!("[DebugPrinter] ->\n{:?}", f);
        }
    }

}

struct SimplePrinter {
    verbose:    bool,
    debug:      bool,
}

impl FilePrinter for SimplePrinter {

    fn new(verbose: bool, debug: bool) -> SimplePrinter {
        SimplePrinter {
            debug:      debug,
            verbose:    verbose,
        }
    }

    fn print_file(&self, f: &File) {
        if self.debug {
            debug!("{:?}", f);
        } else if self.verbose {
            info!("{}", f);
        } else {
            info!("[File]: {}", f.id());
        }
    }

}

pub struct TablePrinter {
    verbose:    bool,
    debug:      bool,
    sp:         SimplePrinter,
}

impl FilePrinter for TablePrinter {

    fn new(verbose: bool, debug: bool) -> TablePrinter {
        TablePrinter {
            debug:      debug,
            verbose:    verbose,
            sp:         SimplePrinter::new(verbose, debug),
        }
    }

    fn print_file(&self, f: &File) {
        self.sp.print_file(f);
    }

    fn print_files<'a, I: Iterator<Item = File<'a>>>(&self, files: I) {
        use prettytable::Table;
        use prettytable::row::Row;
        use prettytable::cell::Cell;

        let titles = row!["File#", "Owner", "ID"];

        let mut tab = Table::new();
        tab.set_titles(titles);

        let mut i = 0;
        for file in files {
            i += 1;
            let cell_i  = Cell::new(&format!("{}", i)[..]);
            let cell_o  = Cell::new(&format!("{}", file.owner().name())[..]);
            let cell_id = Cell::new(&file.id()[..]);
            let row = Row::new(vec![cell_i, cell_o, cell_id]);
            tab.add_row(row);
        }

        tab.printstd();
    }

}
