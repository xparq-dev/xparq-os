// XPARQ OS - Phase 8: ELF Loader
// Minimal ELF64 Parser and Loader for User Space

use crate::memory::mapper::Mapper;
use xparq_hal::x86_64::paging::PageTableFlags;

const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];

#[repr(C, packed)]
struct Elf64Ehdr {
    e_ident: [u8; 16],
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: u64,
    e_phoff: u64,
    e_shoff: u64,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

#[repr(C, packed)]
struct Elf64Phdr {
    p_type: u32,
    p_flags: u32,
    p_offset: u64,
    p_vaddr: u64,
    p_paddr: u64,
    p_filesz: u64,
    p_memsz: u64,
    p_align: u64,
}

const PT_LOAD: u32 = 1;

/// Loads an ELF file from a byte slice into the given Page Table (Mapper).
/// Returns the entry point Virtual Address if successful.
pub fn load_elf(elf_data: &[u8], mapper: &mut Mapper) -> Result<u64, &'static str> {
    if elf_data.len() < core::mem::size_of::<Elf64Ehdr>() {
        return Err("ELF file too small");
    }

    let ehdr = unsafe { &*(elf_data.as_ptr() as *const Elf64Ehdr) };

    // Verify Magic
    if ehdr.e_ident[0..4] != ELF_MAGIC {
        return Err("Invalid ELF magic");
    }

    // Verify 64-bit (Class = 2)
    if ehdr.e_ident[4] != 2 {
        return Err("Not a 64-bit ELF");
    }

    // Verify Little Endian (Data = 1)
    if ehdr.e_ident[5] != 1 {
        return Err("Not Little Endian ELF");
    }

    // Verify Executable (Type = 2)
    if ehdr.e_type != 2 {
        return Err("Not an Executable ELF");
    }

    // Verify x86_64 Machine (Machine = 0x3E)
    if ehdr.e_machine != 0x3E {
        return Err("Not an x86_64 ELF");
    }

    let phoff = ehdr.e_phoff as usize;
    let phnum = ehdr.e_phnum as usize;
    let phentsize = ehdr.e_phentsize as usize;

    if elf_data.len() < phoff + (phnum * phentsize) {
        return Err("ELF program headers out of bounds");
    }

    // Parse Program Headers
    for i in 0..phnum {
        let phdr_offset = phoff + (i * phentsize);
        let phdr = unsafe { &*(elf_data.as_ptr().add(phdr_offset) as *const Elf64Phdr) };

        if phdr.p_type == PT_LOAD {
            // Memory size can be larger than file size (e.g. for .bss)
            let vaddr = phdr.p_vaddr;
            let memsz = phdr.p_memsz;
            let filesz = phdr.p_filesz;
            let offset = phdr.p_offset as usize;

            if memsz == 0 { continue; }

            // Page align addresses
            let start_page = vaddr / 4096;
            let end_page = (vaddr + memsz + 4095) / 4096;
            let pages = end_page - start_page;

            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;

            // Allocate and Map pages
            for p in 0..pages {
                let vaddr_page = start_page + p;
                let vaddr = vaddr_page * 4096;
                
                if mapper.translate(vaddr).is_none() {
                    let frame = {
                        let mut alloc = crate::memory::frame::FRAME_ALLOCATOR.lock();
                        alloc.allocate_frame()
                    }.ok_or("Out of physical memory")?;
                    
                    // Zero the newly allocated frame to ensure .bss is zeroed and no garbage leaks
                    unsafe {
                        core::ptr::write_bytes(frame as *mut u8, 0, 4096);
                    }
                    
                    mapper.map_page(vaddr_page, frame / 4096, flags.clone())?;
                }
            }

            // Copy data to physical memory
            // Note: In a real system, you'd map the frames to a temporary kernel virtual address
            // to copy data into them, OR you'd identity map them if physical = virtual for kernel.
            // Since our kernel currently identity maps the first 256MB, and the bump allocator 
            // runs from 16MB to 32MB, we CAN access the allocated physical frames directly using 
            // the physical address as a virtual pointer!
            for p in 0..pages {
                // Get the physical address mapped to this virtual page
                let phys_addr = mapper.translate((start_page + p) * 4096).unwrap();
                let virt_start = (start_page + p) * 4096;
                
                let page_offset = if virt_start < vaddr {
                    (vaddr - virt_start) as usize
                } else {
                    0
                };
                
                let data_offset = if virt_start < vaddr {
                    0
                } else {
                    (virt_start - vaddr) as usize
                };
                
                if data_offset < filesz as usize {
                    let copy_len = core::cmp::min(4096 - page_offset, (filesz as usize) - data_offset);
                    let src_offset = offset + data_offset;
                    
                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            elf_data.as_ptr().add(src_offset),
                            (phys_addr as *mut u8).add(page_offset),
                            copy_len
                        );
                    }
                }
            }
        }
    }

    Ok(ehdr.e_entry)
}
