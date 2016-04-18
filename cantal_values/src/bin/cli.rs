extern crate argparse;
extern crate cantal_values;

use std::path::{Path, PathBuf};
use std::error::Error;

use argparse::{ArgumentParser, ParseList};

use cantal_values::Metadata;


fn main() {
    let mut files = Vec::<PathBuf>::new();
    {
        let mut ap = ArgumentParser::new();
        ap.refer(&mut files)
            .add_argument("file", ParseList,
                "List of `.values` files");
        ap.parse_args_or_exit();
    }
    for f in files.iter() {
        let meta = match Metadata::read(&Path::new(&f.with_extension("meta"))) {
            Ok(meta) => meta,
            Err(e) => panic!("Error parsing metadata: {}", e),
        };
        let data = match meta.read_data(&Path::new(f)) {
            Ok(data) => data,
            Err(e) => panic!("Error parsing data: {}", e.description()),
        };
        for &(ref descr, ref item) in data.iter() {
            println!("{:?} {} {:?}", f, descr.textname, item);
        }
    }
}
