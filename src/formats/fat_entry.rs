use super::fat::*;
use std::{
    ffi::CString,
    collections::HashMap,
};
use byteorder::{ByteOrder, LittleEndian};

#[derive(Debug)]
/// Entry represents an FAT directory entry
///
/// A directory entry can belong to a file or to a subdirectory.
pub struct Entry {
    /// Name of the directory entry
    name: String,                   
    /// Long version of the file name
    ///
    /// This entry is initially None. One can add a LFN by using the
    /// add_lfn() method.
    long_name: Option<String>,
    /// Attributes of the entry
    attributes: u8,                 
    /// Time created (epoche)
    creat_tos: u8,                  
    /// Time created (hours, minutes, seconds)
    creat_hms: u16,                 
    /// Day created
    creat_day: u16,                 
    /// Day accessed
    access_day: u16,                
    /// Time written to (hours, minutes, seconds)
    written_hms: u16,               
    /// Day written to
    written_day: u16,               
    /// First Cluster that belongs to the file
    start: Cluster,                 
    /// All clusters belonging to the file. One can add the cluster chain by
    /// invoking add_cluster_chain()
    clusters: Option<Vec<Cluster>>,
    /// File size (in bytes)
    size: u32,                      
    /// Checksum of file (required for LFN entries)
    checksum: u8,                   
    /// Deletion marker (0xe5) set? [yes/no]
    deleted: bool                   
}

#[derive(Debug)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
/// Represents a Long File Name entry (LFN)
///
/// FAT uses LFN entries to store long file names (> 11 Bytes).
/// LFN entries store the file name in utf-8 unicode.
pub struct LFNEntry {
    /// An entry can have multiple associated LFN entries.
    /// The sequence number is used to order all LFN entries
    /// belonging to a file.
    sequence_number: u8,
    /// The long file name (or part of it) in utf-8
    filename: String,
    /// A checksum is calculated from the short file name of
    /// the actual directory entry and stored within an LFN entry.
    checksum: u8,
}

impl Entry {
    /// Calculate the checksum of a filename
    ///
    /// The checksum is used to connect LFN entries to a normal directory entry.
    ///
    /// # Arguments
    ///
    /// * `s` - A string slice that holds the name of a file. It's expected to be exactly 11 Bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use greasy::formats::fat::Entry;
    ///
    /// let alice = "Alice      ";
    /// let work = "WORK       ";
    ///
    /// assert_eq!(8, Entry::checksum(alice));
    /// assert_eq!(163, Entry::checksum(work));
    /// ```
    pub fn checksum(s: &str) -> u8 {
        let mut checksum: u16 = 0;

        for c in s.as_bytes() {
            checksum = (((checksum & 1) << 7 | (checksum >> 1)) + *c as u16) % 256;
        }

        checksum as u8
    }

    /// Returns a entry
    ///
    /// # Arguments
    ///
    /// * `mem` - A byte slice representing the entry in memory (Expected to be 32-Bytes)
    pub fn new(mem: &[u8]) -> Entry {
        /*
        let name = CString::new(&mem[..11]).expect("Parsing name field failed")
                                          .into_string()
                                          .expect("Translation from CString to String failed");
                                          */
        let name = String::from_utf8_lossy(&mem[..11]).to_string();
        Entry {
            attributes: mem[11] as u8,
            creat_tos: mem[13] as u8,
            creat_hms: LittleEndian::read_u16(&mem[14..16]),
            creat_day: LittleEndian::read_u16(&mem[16..18]),
            access_day: LittleEndian::read_u16(&mem[18..20]),
            written_hms: LittleEndian::read_u16(&mem[22..24]),
            written_day: LittleEndian::read_u16(&mem[24..26]),
            start: Cluster(((LittleEndian::read_u16(&mem[20..22]) as u32) << 16) +
                             LittleEndian::read_u16(&mem[26..28]) as u32),
            clusters: None,
            size: LittleEndian::read_u32(&mem[28..32]),
            checksum: Entry::checksum(&name),
            deleted: match mem[0] {
                0xe5 => true,
                _ => false,
            },
            long_name: None,
            name: name,
        }
    }

    /// Checks if entry is a disk volume entry 
    pub fn is_disk_volume_entry(&self) -> bool {
        (self.attributes & 0x08) != 0
    }
    
    /// Checks if entry is a sub directory
    pub fn is_subdir_entry(&self) -> bool {
        (self.attributes & 0x10) != 0
    }
    
    /// Checks if deltion marker is set
    pub fn is_deleted(&self) -> bool {
        self.deleted
    }

    pub fn is_this_entry(&self) -> bool {
        self.name.trim() == "."
    }

    pub fn is_prev_entry(&self) -> bool {
        self.name.trim() == ".."
    }
    
    /// Returns the string representation of the given entry
    pub fn to_string(&self) -> String {
        let entry_type;
        let del;
        let name = match &self.long_name {
            Some(n) => n,
            None => self.name.trim(),
        };

        if self.is_disk_volume_entry() {
            entry_type = "V"; // Disk Volume
        } else if self.is_subdir_entry() {
            entry_type = "D"; // Directory
        } else {
            entry_type = "F"; // File
        }

        if self.deleted {
            del = "X | ";
        } else {
            del = "";
        }

        

        format!("[{}: {}{}]", name, del, entry_type)
    }
    
    /// Add the LFN name to the entry
    ///
    /// # Arguments
    ///
    /// * `lfns` - A hash map that maps from a cheksum to a vector of LFN entries
    ///
    /// The LFN entries are sorted based on their sequencing number and then
    /// concatendated to build a single string. That string is then assigned to
    /// the long_name filed of the given entry.
    pub fn add_lfn(&mut self, lfns: &mut HashMap<u8, Vec<LFNEntry>>) {
        if let Some(lfn_vec) = lfns.get_mut(&self.checksum) {
            let mut s = String::new();
            lfn_vec.sort();

            for e in lfn_vec {
                s.push_str(&e.filename);
            }

            self.long_name = Some(s);
        }
    }

    pub fn add_clusters(&mut self, clusters: Vec<Cluster>) {
        self.clusters = Some(clusters);
    }

    pub fn clusters(&self) -> &Option<Vec<Cluster>> {
        &self.clusters
    }

    pub fn start(&self) -> &Cluster {
        &self.start
    }
}

impl LFNEntry {
    /// Returns a LFN entry
    ///
    /// # Arguments
    ///
    /// * `mem` - A byte slice representing the entry in memory (Expected to be 32-Bytes)
    pub fn new(mem: &[u8]) -> LFNEntry {
        let mut s1 = String::from_utf8_lossy(&mem[1..11]).to_string();
        let s2 = String::from_utf8_lossy(&mem[14..26]).to_string();
        let s3 = String::from_utf8_lossy(&mem[28..32]).to_string();

        s1.push_str(&s2);
        s1.push_str(&s3);

        LFNEntry {
            sequence_number: mem[0],
            filename: s1.trim_matches(|c| c == std::char::REPLACEMENT_CHARACTER).to_string(),
            checksum: mem[13],
        }
    }

    pub fn checksum(&self) -> u8 {
        self.checksum
    }
    
    /// Checks if the a attributes indicate an LFN entry
    ///
    /// # Arguments
    ///
    /// * `attributes` - Attributes byte (offset 11) of an directory entry
    pub fn is_lfn_entry(attributes: u8) -> bool {
        attributes == 0x0f
    }
}
