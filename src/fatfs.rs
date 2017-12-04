
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
	fat_start: u16,
	clusters_start: u32,
	sectors_per_cluster: u8,
	cluster_size: u16,
	root_dir: u32
}

#[derive(Debug)]
struct BootRecord {
	bytes_per_sector: u16,
	sectors_per_cluster: u8,
	reserved_sectors:u16,
	fat_copies:u8,
	sectors_per_fat:u32,
	root_directory_cluster_start:u32,
	fs_info_sector:u16
}
impl FatFileSystem {
	pub fn new(path: &str) -> FatFileSystem{
	    let mut file = File::open(path).expect("file not found");

		let boot_record = FatFileSystem::get_boot_record(&mut file);

		let layout = FileSystemLayout {
			fat_start : boot_record.reserved_sectors,
			clusters_start: ((boot_record.reserved_sectors as u32 + (boot_record.fat_copies as u32 * boot_record.sectors_per_fat)) * boot_record.bytes_per_sector	as u32),
			sectors_per_cluster: boot_record.sectors_per_cluster,
			root_dir: boot_record.root_directory_cluster_start,
			cluster_size: boot_record.sectors_per_cluster as u16 * boot_record.bytes_per_sector
		};
		let fs = FatFileSystem {
			file: file,
			layout: layout
		};
		println!("{:?}",boot_record);
		println!("{:?}",fs.layout);
		fs
	}

	fn get_boot_record(file: &mut File) -> BootRecord{

		let mut boot_record = [0;256];
		let read = file.read(&mut boot_record).expect("Couldn't read");

		let boot_record = BootRecord {
			bytes_per_sector : LittleEndian::read_u16(&boot_record[0xB..0xD]),
			sectors_per_cluster: boot_record[0xD],
			reserved_sectors : LittleEndian::read_u16(&boot_record[0xE..0x10]),
			fat_copies: boot_record[0x10],
			sectors_per_fat: LittleEndian::read_u32(&boot_record[0x24..0x28]),
			root_directory_cluster_start: LittleEndian::read_u32(&boot_record[0x2c..0x30]),
			fs_info_sector: LittleEndian::read_u16(&boot_record[0x30..0x32]),
		};
		boot_record
	}

	pub fn read_cluster(&mut self,cluster_number: u32) -> Vec<u8> {
		let mut ret = vec![0u8;self.layout.cluster_size as usize];
		let seek_location = self.layout.clusters_start + (cluster_number -2)* self.layout.cluster_size as u32;
		self.file.seek(SeekFrom::Start(seek_location as u64));
		let read = self.file.read(&mut ret);
		println!("{:?}",str::from_utf8(&ret[0..7]));
		ret
	}



}
