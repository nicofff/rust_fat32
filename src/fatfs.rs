
use std::fs::File;
use byteorder::{ByteOrder,LittleEndian};
use std::str;
//use std::io;
use std::io::Read;
use std::io::{Seek,SeekFrom};

#[derive(Debug)]
pub struct FatFileSystem {
	file: File,
	layout: FileSystemLayout
}

#[derive(Debug)]
struct FileSystemLayout {
	fat_start: usize,
	clusters_start: usize,
	sectors_per_cluster: usize,
	cluster_size: usize,
	root_dir: usize
}

#[derive(Debug)]
struct BootRecord {
	bytes_per_sector: usize,
	sectors_per_cluster: usize,
	reserved_sectors:usize,
	fat_copies:usize,
	sectors_per_fat:usize,
	root_directory_cluster_start:usize,
	fs_info_sector:usize
}

pub struct DirectoryEntry {
	pub name: String,
	subdir:bool,
	cluster: usize,
	filesize: usize
}

impl FatFileSystem {
	pub fn new(path: &str) -> FatFileSystem{
	    let mut file = File::open(path).expect("file not found");

		let boot_record = FatFileSystem::get_boot_record(&mut file);

		let layout = FileSystemLayout {
			fat_start : boot_record.reserved_sectors,
			clusters_start: (boot_record.reserved_sectors + (boot_record.fat_copies* boot_record.sectors_per_fat)) * boot_record.bytes_per_sector,
			sectors_per_cluster: boot_record.sectors_per_cluster,
			root_dir: boot_record.root_directory_cluster_start,
			cluster_size: boot_record.sectors_per_cluster * boot_record.bytes_per_sector
		};
		let fs = FatFileSystem {
			file: file,
			layout: layout
		};
		// println!("{:?}",boot_record);
		// println!("{:?}",fs.layout);
		fs
	}

	fn get_boot_record(file: &mut File) -> BootRecord{

		let mut boot_record = [0;256];
		file.read(&mut boot_record).expect("Couldn't read");

		let boot_record = BootRecord {
			bytes_per_sector : LittleEndian::read_u16(&boot_record[0xB..0xD]) as usize,
			sectors_per_cluster: boot_record[0xD] as usize,
			reserved_sectors : LittleEndian::read_u16(&boot_record[0xE..0x10]) as usize,
			fat_copies: boot_record[0x10] as usize,
			sectors_per_fat: LittleEndian::read_u32(&boot_record[0x24..0x28]) as usize,
			root_directory_cluster_start: LittleEndian::read_u32(&boot_record[0x2c..0x30]) as usize,
			fs_info_sector: LittleEndian::read_u16(&boot_record[0x30..0x32]) as usize,
		};
		boot_record
	}

	pub fn read_cluster(&mut self,cluster_number: usize) -> Vec<u8> {
		let mut ret = vec![0u8;self.layout.cluster_size];
		let seek_location = self.get_cluster_offset(cluster_number);
		self.file.seek(SeekFrom::Start(seek_location as u64)).unwrap();
		self.file.read(&mut ret).unwrap();
		ret
	}

	fn get_cluster_offset(&self,cluster_number: usize) -> usize {
		self.layout.clusters_start + (cluster_number)* self.layout.cluster_size
	}

	pub fn get_entries(&self,cluster: &Vec<u8>) -> Vec<DirectoryEntry> {
		let mut entries : Vec<DirectoryEntry> = Vec::new();
		let mut ix = 0;
		loop {
			let parsed = self.parse_directory_entry(cluster,ix);
			if let Some(entry) = parsed {
				if entry.name.chars().nth(0).unwrap() != '\u{53}' { // Not deleted
					entries.push(entry);
				}	
			} else {
				break
			}
			ix+=1;
		}
		entries
	}

	fn parse_directory_entry(&self,cluster: &Vec<u8>,index: usize) -> Option<DirectoryEntry> {
		if index * 32 > self.layout.cluster_size {
			panic!("entry offset greater than cluster size");
		} else {
			let entry = &cluster[index*32..(index+1)*32];
			if entry[0] == 0 {
				return None
			} else {
				
			let ret = DirectoryEntry{
				name: format!("{}.{}",str::from_utf8(&entry[0..8]).unwrap().trim_matches('\0'),str::from_utf8(&entry[8..11]).unwrap()),
				subdir:entry[12] & 0x10 != 0,
				cluster: LittleEndian::read_u16(&entry[0x14..0x16])as usize >> 16 + LittleEndian::read_u16(&entry[0x1A..0x1C]) as usize,
				filesize: LittleEndian::read_u32(&entry[0x1C..0x20]) as usize
			};
			return Some(ret);	
			}
			

		}
		
	}

}
