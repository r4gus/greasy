use memmap::MmapOptions;
use std::{
    fs::File,
};
use greasy::formats::fat;
use clap::{Arg, App, SubCommand};

fn main() -> std::io::Result<()> {
    let matches = App::new("Greasy")
        .version("0.1.0")
        .author("David Sugar (r4gus)")
        .about("Fat file system information and data recovery tool")
        .arg(Arg::with_name("info")
             .short("i")
             .long("info")
             .help("Display general file system layout information"))
        .arg(Arg::with_name("tree")
             .short("t")
             .long("tree")
             .help("Display all directories in a tree like manner"))
        .arg(Arg::with_name("INPUT")
             .help("Fat volume to parse (e.g. fat-16.dd)")
             .required(true)
             .index(1))
        .get_matches();

    let file = File::open(matches.value_of("INPUT").unwrap())?;
    let mem = unsafe { MmapOptions::new().map(&file)? };

    let fat = fat::Fat::new(mem);


    if matches.is_present("info") {
        fat.info();
    }
    
    if matches.is_present("tree") {
        fat.tree();
    }
    

    Ok(())
}
