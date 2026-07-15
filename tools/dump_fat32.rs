use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::convert::TryInto;

fn main() {
    let mut f = File::open("build/x86-64/fat32.img").unwrap();
    let mut data = [0u8; 512];
    f.read_exact(&mut data).unwrap();
    
    let bps = u16::from_le_bytes(data[11..13].try_into().unwrap());
    let spc = data[13];
    let res = u16::from_le_bytes(data[14..16].try_into().unwrap());
    let fats = data[16];
    let roots = u16::from_le_bytes(data[17..19].try_into().unwrap());
    let spf32 = u32::from_le_bytes(data[36..40].try_into().unwrap());
    let root_clust = u32::from_le_bytes(data[44..48].try_into().unwrap());
    
    println!("BPS: {}", bps);
    println!("SPC: {}", spc);
    println!("Reserved: {}", res);
    println!("FATs: {}", fats);
    println!("RootEntries: {}", roots);
    println!("SPF32: {}", spf32);
    println!("RootCluster: {}", root_clust);
    
    let root_dir_sectors = ((roots as u32 * 32) + (bps as u32 - 1)) / bps as u32;
    let first_data_sector = res as u32 + (fats as u32 * spf32) + root_dir_sectors;
    let first_lba = first_data_sector + ((root_clust - 2) * spc as u32);
    
    println!("FirstDataLBA: {}", first_lba);
    
    f.seek(SeekFrom::Start((first_lba * 512) as u64)).unwrap();
    let mut root_data = [0u8; 64];
    f.read_exact(&mut root_data).unwrap();
    
    println!("Root Dir Data:");
    for i in (0..64).step_by(16) {
        for j in 0..16 {
            print!("{:02x} ", root_data[i + j]);
        }
        println!();
    }
}
