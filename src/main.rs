use memmap::MmapOptions;
use std::{
    fs::File,
    ffi::CString,
};
use recovery::formats::fat;


fn main() -> std::io::Result<()> {
    let file = File::open("../fat-16.dd")?;
    let mem = unsafe { MmapOptions::new().map(&file)? };
    let s = CString::new(&mem[..11]).expect("CString::new failed");

    let fat = fat::Fat::new(mem);
    
    println!("{:?}", s);
    fat.info();

    Ok(())
}
