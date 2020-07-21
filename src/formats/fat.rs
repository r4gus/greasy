use memmap::{Mmap};
use std::{
    ffi::CString,
};
use byteorder::{ByteOrder, LittleEndian};

#[derive(Debug)]
pub struct Cluster(u32);

#[derive(Debug)]
pub struct Sector(u32);

#[derive(Debug)]
pub struct Fat {
    mem: Mmap,
    oem: String,                    // original equipment manufacturer label
    fat_type: String,               // FAT12, FAT16, FAT32
    fat_table_sectors: u32,         // Number of sectors per FAT table
    fat_table_entry_size: u8,       // Number of bytes per FAT table entry
    fat_table_count: u8,            // Number of FAT tables (usually: table + copy, e.g. 2)
    bytes_per_sector: u16,          // Number of Bytes per sector (512, 1024, ...)
    sectors_per_cluster: u8,        // Number of sectors per cluster (usually 2^n, for a small n)
    total_clusters: u32,            // Total number of clusters in cluster area
    total_sectors: u32,             // Total number of sectors of the volume
    sectors_reserved_area: u16,     // Number of sectors belonging to the reserved area
    start_reserved_area: Sector,    // Offset to the reserved area
    sectors_fat_area: u32,          // Total number of sectors of the fat area
    start_fat_area: Sector,         // Offset to the fat area
    start_data_area: Sector,        // Offset to the data area
    start_root_dir: Sector,        // Offset to the root directory
    start_cluster_area: Sector      // Offset to the cluster area
}

impl Fat {
    const DIR_ENTRY_SIZE: u16 = 32;

    pub fn new(mem: Mmap) -> Fat {
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
                "FAT12" => 12,
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
        let start_cluster_area = match fat_type.trim() {
                "FAT32" => start_data_area,
                _ => start_data_area + ((LittleEndian::read_u16(&mem[17..19]) * Fat::DIR_ENTRY_SIZE) / bytes_per_sector) as u32,
        };
        let start_root_dir = match fat_type.trim() {
                "FAT32" => ((LittleEndian::read_u32(&mem[44..48]) - 2) * sectors_per_cluster as u32) + start_cluster_area,
                _ => start_data_area,
        };
        let total_clusters = ((total_sectors - start_cluster_area) / sectors_per_cluster as u32) + 1;


        Fat {
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
        }
    }

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
    }
}
