use std::env;
use std::fs::OpenOptions;
use std::io::Read;
use fatfs::{FileSystem, FsOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: fat32-injector <image.img> <file_to_inject> [target_name]");
        std::process::exit(1);
    }

    let img_path = &args[1];
    let file_to_inject = &args[2];
    
    // Get target name, default to the file's basename
    let target_name = if args.len() >= 4 {
        args[3].clone()
    } else {
        let path = std::path::Path::new(file_to_inject);
        path.file_name().unwrap().to_string_lossy().to_string()
    };
    
    // Ensure the target name is uppercase for simple FAT32 8.3 compatibility
    let target_name = target_name.to_ascii_uppercase();

    println!("Injecting '{}' into '{}' as '{}'", file_to_inject, img_path, target_name);

    // Read the source file
    let mut source_file = std::fs::File::open(file_to_inject)?;
    let mut source_data = Vec::new();
    source_file.read_to_end(&mut source_data)?;

    // Open or create the FAT32 image
    let is_new = !std::path::Path::new(img_path).exists();
    let img_file = OpenOptions::new().read(true).write(true).create(true).open(img_path)?;
    
    if is_new {
        // Set size to 34MB to guarantee FAT32 (>65525 clusters)
        img_file.set_len(34 * 1024 * 1024)?;
        let mut buf_stream = fscommon::BufStream::new(img_file.try_clone()?);
        let options = fatfs::FormatVolumeOptions::new().fat_type(fatfs::FatType::Fat32);
        fatfs::format_volume(&mut buf_stream, options)?;
    }
    let buf_stream = fscommon::BufStream::new(img_file);
    
    // Mount the FAT32 filesystem
    let fs = FileSystem::new(buf_stream, FsOptions::new())?;
    
    // Get root directory
    let root = fs.root_dir();
    
    // Remove the file if it already exists
    let _ = root.remove(&target_name);

    // Create the new file
    let mut target_file = root.create_file(&target_name)?;
    
    // Write data
    use std::io::Write;
    target_file.write_all(&source_data)?;
    
    println!("Successfully injected {} bytes.", source_data.len());
    Ok(())
}
