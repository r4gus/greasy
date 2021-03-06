use memmap::{Mmap};
use std::{
    ffi::CString,
    collections::HashMap,
};
use byteorder::{ByteOrder, LittleEndian};
use super::fat_entry::*;

// ###################### TRAITS #############################

pub trait FAT {
    fn tree(&self);
    fn info(&self);
}

// ###################### STRUCTURES #########################

#[derive(Debug)]
/// Represents a specific Cluster (not a range)
pub struct Cluster(pub u32);

#[derive(Debug)]
/// Represents a specific Sector (not a range)
pub struct Sector(pub u32);

#[derive(Debug)]
/// Fat represents the base of a FAT File System
///
/// All attributes are shared between the different
/// types of FAT file systems.
pub struct Fat {
    /// Memory mapping of the File System ([u8])
    mem: Mmap,                      
    /// original equipment manufacturer label
    oem: String,                    
    /// The FAT type (FAT16, FAT32) as String
    fat_type: String,               
    /// Number of sectors per FAT table
    fat_table_sectors: u32,         
    /// Number of bytes per FAT table entry
    fat_table_entry_size: u8,       
    /// Number of FAT tables (usually: table + copy, e.g. 2)
    fat_table_count: u8,            
    /// Number of Bytes per sector (512, 1024, ...)
    bytes_per_sector: u16,          
    /// Number of sectors per cluster (usually 2^n, for a small n)
    sectors_per_cluster: u8,        
    /// Total number of clusters in cluster area
    total_clusters: u32,            
    /// Total number of sectors of the volume
    total_sectors: u32,             
    /// Number of sectors belonging to the reserved area
    sectors_reserved_area: u16,     
    /// Offset to the reserved area
    start_reserved_area: Sector,    
    /// Total number of sectors of the fat area
    sectors_fat_area: u32,          
    /// Offset to the fat area
    start_fat_area: Sector,         
    /// Offset to the data area
    start_data_area: Sector,        
    /// Offset to the root directory
    start_root_dir: Sector,
    /// Offset to the cluster area
    start_cluster_area: Sector      
}

#[derive(Debug)]
/// Fat represents a FAT16 File System
pub struct Fat16 {
    /// Parent
    fat: Fat,
    /// Total number of root entries
    total_root_entries: u16,
}

#[derive(Debug)]
/// Fat represents a FAT32 File System
pub struct Fat32 {
    fat: Fat,
    /// All clusters that belong to the root dir
    root_clusters: Vec<Cluster>,
}

// ###################### IMPLEMENTATIONS #########################

impl Fat {
    /// Size of a directory entry in bytes
    const DIR_ENTRY_SIZE: u16 = 32;
    const EOF16: i16 = -1;
    const EOF32: i32 = 0x0fffffff;
    const BAD16: i16 = -9;
    const BAD32: i32 = -9;
    
    /// Convert a cluster number into a sector number
    ///
    /// # Arguments
    ///
    /// * `cluster` - Cluster number (must be >= 2)
    fn cluster_to_sector(&self, cluster: &Cluster) -> Sector {
        assert!(cluster.0 >= 2);
        Sector(((cluster.0 - 2) * self.sectors_per_cluster as u32) + self.start_cluster_area.0)
    }
    
    /// Convert a sector number into a cluster number
    ///
    /// # Arguments
    ///
    /// * `sector` - Sector number
    fn sector_to_cluster(&self, sector: &Sector) -> Cluster {
        Cluster((sector.0 - self.start_cluster_area.0) / self.sectors_per_cluster as u32)
    }
    
    /// Calculate the offset from the beginning of the file (in bytes)
    ///
    /// # Arguments
    ///
    /// * `sector` - Secotr number that should be converted into an offset
    fn offset(&self, sector: &Sector) -> usize {
        sector.0 as usize * self.bytes_per_sector as usize
    }
    
    /// Returns a byte index into the FAT table that corresponds to the given cluster
    ///
    /// # Arguments
    ///
    /// * `cluster` - The n'th cluster to get the index for
    pub fn fat_table_offset(&self, cluster: &Cluster) -> usize {
        assert!(cluster.0 >= 2);
        ((self.start_fat_area.0 * self.bytes_per_sector as u32) + (cluster.0 * (self.fat_table_entry_size / 8) as u32)) as usize
    }
    
    /// Converts a vector of clusters into a vector of byte offsets
    ///
    /// # Arguments
    ///
    /// * `clusters' - Vector of clusters
    pub fn clusters_to_offsets(&self, clusters: &Vec<Cluster>) -> Vec<usize> {
        let mut offsets = Vec::new();

        for cluster in clusters {
            let sector = self.cluster_to_sector(cluster);
            offsets.push(self.offset(&sector));
        }

        offsets
    }
    
    /// Returns a Vector of Clusters that belong to a single file or directory
    ///
    /// # Arguments
    ///
    /// * `cluster` - First cluster of the cluster chain
    ///
    /// # FAT table Entry types
    /// ## Fat16
    /// 1. unused/ free cluster: 0x0000
    /// 2. bad cluster: -9
    /// 3. address of next cluster: n
    /// 4. last cluster in a file (EOF): -1
    ///
    /// ## Fat32
    /// 1. unused/ free cluster: 0x0000
    /// 2. bad cluster: 0xfffffff7
    /// 3. address of next cluster: n
    /// 4. last cluster in a file (EOF): 0x0fffffff
    fn get_cluster_chain(&self, cluster: &Cluster) -> Vec<Cluster> {
        let mut clusters = Vec::new();
        let mut offset;

        if self.fat_table_entry_size == 16 {
            let mut n = cluster.0 as i16;

            while n != Fat::EOF16 && n != 0 && n != Fat::BAD16 {
                let clu = Cluster(n as u32);
                offset = self.fat_table_offset(&clu);
                clusters.push(clu);
                n = LittleEndian::read_i16(&self.mem[offset..offset+self.fat_table_entry_size as usize]);
            }
        } else {
            let mut n = cluster.0 as i32;

            while n != Fat::EOF32 && n != 0 && n != Fat::BAD32 {
                let clu = Cluster(n as u32);
                offset = self.fat_table_offset(&clu);
                clusters.push(clu);
                n = LittleEndian::read_i32(&self.mem[offset..offset+self.fat_table_entry_size as usize]);
            }
        }

        clusters
    }
    
    /// Returns a new Box pointer to a Fat16 or Fat32
    ///
    /// # Arguments
    ///
    /// * `mem` - Mmap struct that holds the byte stream to parse
    ///
    /// # Examplse
    ///
    /// ```
    /// use greasy::formats::fat;
    /// use MmapOptions;
    /// use std::fs::File;
    ///
    /// let file = File::open("fat-16.dd")?;                    // open a fat volume
    /// let mem = unsafe { MmapOptions::new().map(&file)? };    // map the volume into memory
    ///
    /// let fat = fat::Fat::new(mem);                           // create a new Fat object
    /// ```
    pub fn new(mem: Mmap) -> Box<dyn FAT> {
        let oem = CString::new(&mem[3..11]).expect("Parsing oem field for failed")
                          .into_string().expect("Translation from CString to String failed");

        let fat_table_sectors = match LittleEndian::read_u16(&mem[22..24]) {
                0 => LittleEndian::read_u32(&mem[36..40]),                  // 0 indicates FAT32
                _ => LittleEndian::read_u16(&mem[22..24]) as u32,
        };

        let fat_type = match LittleEndian::read_u16(&mem[22..24]) {
                0 => CString::new(&mem[82..90]).expect("Parsing type field for FAT32 failed") // 0 indicates FAT32
                        .into_string().expect("Translation from CString to String failed"),
                _ => CString::new(&mem[54..62]).expect("Parsing type field for FAT12/16 failed")
                        .into_string().expect("Translation from CString to String failed"),
        };

        let fat_table_entry_size = match fat_type.trim() {
                "FAT16" => 16,
                "FAT32" => 32,
                _ => 0,
        };

        let total_sectors = match LittleEndian::read_u16(&mem[19..21]) {
                0 => LittleEndian::read_u32(&mem[32..36]),
                _ => LittleEndian::read_u16(&mem[19..21]) as u32,
        };

        let sectors_reserved_area = LittleEndian::read_u16(&mem[14..16]);
        let fat_table_count = mem[16] as u8;
        let bytes_per_sector = LittleEndian::read_u16(&mem[11..13]);
        let sectors_per_cluster = mem[13] as u8;

        let sectors_fat_area = (fat_table_count as u32) * fat_table_sectors;
        let start_fat_area = sectors_reserved_area;
        let start_data_area = (start_fat_area as u32) + sectors_fat_area;
        let total_root_entries = LittleEndian::read_u16(&mem[17..19]);
        let start_cluster_area = match fat_type.trim() {
                "FAT32" => start_data_area,
                _ => start_data_area + ((total_root_entries * Fat::DIR_ENTRY_SIZE) / bytes_per_sector) as u32,
        };
        let root_cluster = LittleEndian::read_u32(&mem[44..48]);
        let start_root_dir = match fat_type.trim() {
                "FAT32" => ((root_cluster - 2) * sectors_per_cluster as u32) + start_cluster_area,
                _ => start_data_area,
        };
        let total_clusters = ((total_sectors - start_cluster_area) / sectors_per_cluster as u32) + 1;


        let f = Fat {
            oem: oem,
            fat_table_sectors: fat_table_sectors,
            fat_type: fat_type,
            fat_table_entry_size: fat_table_entry_size,
            fat_table_count: fat_table_count,
            bytes_per_sector: bytes_per_sector,
            sectors_per_cluster: sectors_per_cluster,
            total_sectors: total_sectors,
            sectors_reserved_area: sectors_reserved_area,
            start_reserved_area: Sector(0),
            sectors_fat_area: sectors_fat_area,
            start_fat_area: Sector(start_fat_area as u32),
            start_data_area: Sector(start_data_area),
            start_root_dir: Sector(start_root_dir),
            start_cluster_area: Sector(start_cluster_area),
            total_clusters: total_clusters,
            mem: mem,
        };

        if f.fat_type.trim() == "FAT16" {
            return Box::new(Fat16{fat: f, total_root_entries: total_root_entries});
        } else {
            return Box::new(Fat32{fat: f, root_clusters: vec![Cluster(root_cluster)]});
        }
    }
    
    /// Parse and display entries of a directory and it's sub directories
    /// recursively.
    ///
    /// # Arguments
    ///
    /// * `offset` - Vector of byte offsets to the different clusters of a directory
    /// * `max` - Maximum number of bytes per cluster
    /// * 'indentation' - Indentation level
    ///
    /// There is only one offset if the fat is of type fat16 and it has a max size of
    /// <total_root_entries * Fat::DIR_ENTRY_SIZE>.
    fn _tree(&self, offset: Vec<usize>, max: usize, indentation: u8) {
        let mut files: Vec<Entry> = Vec::new();
        let mut lfns: HashMap<u8, Vec<LFNEntry>> = HashMap::new();
        let mut i: usize;
        let mut next; 
        let mut indent_str = String::new();
        
        // build indentation string
        for _x in 0..indentation {
            indent_str.push_str("*");
        }

        // iterate over each cluster offset of the current dir
        for coff in offset {   
            i = 0;

            while self.mem[coff + i] != 0 && i < max {
                next = i + Fat::DIR_ENTRY_SIZE as usize;

                if LFNEntry::is_lfn_entry(self.mem[coff + i + 11]) {
                    let lfn_entry = LFNEntry::new(&self.mem[coff+i..coff+next]);
                    let lfn_vec = lfns.entry(lfn_entry.checksum()).or_insert(Vec::new());
                    lfn_vec.push(lfn_entry);
                } else {
                    let entry = Entry::new(&self.mem[coff+i..coff+next]);
                    files.push(entry);
                }

                i = next;
            }

            if self.mem[coff + i] != 0 {
                break;
            }
        }

        for e in &mut files {
            e.add_lfn(&mut lfns);
            e.add_clusters(self.get_cluster_chain(&e.start()));
            

            if e.is_this_entry() == false && e.is_prev_entry() == false {
                print!("{}", indent_str);
                println!("{}", e.to_string());
            }

            if e.is_subdir_entry() && e.is_this_entry() == false && e.is_prev_entry() == false {
                match e.clusters() {
                    Some(clu) => self._tree(self.clusters_to_offsets(clu), (self.bytes_per_sector * (self.sectors_per_cluster as u16)) as usize, indentation + 1),
                    None => (),
                };
            }
        }
    }
    
    /// Display general information about the file system
    pub fn info(&self) {
        println!("FILE SYSTEM INFORMATION
--------------------------------
File System Type: {}
OEM Name: {}
Vloume ID:
Volume Label (Boot Sector):
File System Type Label: {}

Size
--------------------------------
Sector Size (in bytes): {}
Cluster Size (in bytes): {}
Cluster Range: 2 - {}

File System Layout (in sectors)
--------------------------------
Total Sector Range: 0 - {}
|- Reserved: 0 - {}
|  └─ Boot Sector: 0",
        self.fat_type,
        self.oem,
        self.fat_type,
        self.bytes_per_sector,
        self.bytes_per_sector * (self.sectors_per_cluster as u16),
        self.total_clusters,
        self.total_sectors - 1,
        self.sectors_reserved_area - 1,
        );

        for i in 0..self.fat_table_count as u32 {
            println!("|- FAT {}: {} - {}", i, self.start_fat_area.0 + (i * self.fat_table_sectors), 
                     self.start_fat_area.0 + ((i+1) * self.fat_table_sectors) - 1);
        }

        println!("└─ Data Area: {} - {}", self.start_data_area.0, self.total_sectors - 1);

        if self.fat_type.trim() == "FAT32" {
            println!("    └─ Cluster Area: {} - {}", self.start_cluster_area.0, self.total_sectors - 1);
            println!("        └─ Root: {}", self.start_root_dir.0);
        } else {
            println!("    |- Root: {} - {}", self.start_root_dir.0, self.start_cluster_area.0 - 1);
            println!("    └─ Cluster Area: {} - {}", self.start_cluster_area.0, self.total_sectors - 1);
        }

        println!("\n");
    }
    
}

impl FAT for Fat16 {
    fn tree(&self) {
        println!("File layout:\nDeleted = X, Disk Volume = V\nDirectory = D, File = F\n---------------------------------------");
        let offset = self.fat.offset(&self.fat.start_root_dir);
        self.fat._tree(vec![offset], (self.total_root_entries * Fat::DIR_ENTRY_SIZE) as usize, 1);
    }


    fn info(&self) {
        self.fat.info();
    }
}

impl FAT for Fat32 {
    fn tree(&self) {
        println!("Not implemented for Fat32 yet!");
    }

    fn info(&self) {
        self.fat.info();
    }
}

