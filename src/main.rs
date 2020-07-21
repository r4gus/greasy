use memmap::MmapOptions;
use std::{
    fs::File,
};
use greasy::formats::fat;


fn main() -> std::io::Result<()> {
    let file = File::open("../fat-16.dd")?;
    let mem = unsafe { MmapOptions::new().map(&file)? };

    let fat = fat::Fat::new(mem);
    
    fat.info();

    Ok(())
}
