// XPARQ OS - Phase 6: Syscall Dispatcher
// Validates and routes system calls from user-space

use crate::task::TASK_MANAGER;

// Syscall Numbers
pub const SYS_YIELD: u64 = 1;
pub const SYS_SLEEP: u64 = 2;
pub const SYS_EXIT: u64 = 3;
pub const SYS_OPEN: u64 = 4;
pub const SYS_READ: u64 = 5;
pub const SYS_WRITE: u64 = 6;
pub const SYS_EXECVE: u64 = 7;
pub const SYS_IPC_SEND: u64 = 8;
pub const SYS_IPC_RECV: u64 = 9;
// Phase 17: Networking Syscalls
pub const SYS_SOCKET: u64 = 10;   // create UDP socket -> sockfd
pub const SYS_BIND: u64 = 11;     // bind(sockfd, port) -> 0 or errno
pub const SYS_SENDTO: u64 = 12;   // sendto(sockfd, buf_ptr, len, dst_ip_u32, dst_port) (arg4/5 via later extension)
pub const SYS_RECVFROM: u64 = 13; // recvfrom(sockfd, buf_ptr, len) -> bytes_read
pub const SYS_CLOSE: u64 = 14;    // close(fd)
pub const SYS_TCP_SOCKET: u64 = 15; // create TCP socket -> fd
pub const SYS_TCP_LISTEN: u64 = 16; // listen on TCP socket
pub const SYS_TCP_ACCEPT: u64 = 17;
pub const SYS_TCP_CONNECT: u64 = 18;

// Syscall assembly entry point
// The CPU jumps here with:
// RCX = RIP at the time of syscall
// R11 = RFLAGS at the time of syscall
// RSP = User Stack
core::arch::global_asm!(
    r#"
    .global syscall_entry
    syscall_entry:
        // 1. Swap GS to get access to Kernel GS Base (CpuLocal)
        swapgs
        
        // 2. Save User RSP to CpuLocal.user_rsp (offset 8) -- needed for kernel stack switch
        mov gs:[8], rsp
        
        // 3. Load Kernel RSP from CpuLocal.kernel_rsp (offset 0)
        mov rsp, gs:[0]
        
        // 4. Save User RIP, RSP, RFLAGS in CpuLocal for save_user_context() to read
        mov gs:[16], rcx       // user_rip
        mov gs:[24], r11       // user_rflags
        // user_rsp was already saved in step 2 at offset 8
        
        // Save caller-saved registers that the C ABI clobbers
        push rdi
        push rsi
        push rdx
        push r8
        push r9
        push r10

        // RAX contains sys_num, which must be preserved across save_user_context
        push rax

        // Save the user context (RIP/RSP/RFLAGS) into the current Task struct
        call save_user_context

        pop rax

        // We need to pass args to syscall_handler_inner
        // sys_num is in RAX (needs to be 1st arg: RDI)
        // arg1 is in RDI (needs to be 2nd arg: RSI)
        // arg2 is in RSI (needs to be 3rd arg: RDX)
        // arg3 is in RDX (needs to be 4th arg: RCX)
        pop r10
        pop r9
        pop r8
        pop rdx
        pop rsi
        pop rdi

        push rdi
        push rsi
        push rdx
        push r8
        push r9
        push r10
        mov rcx, rdx
        mov rdx, rsi
        mov rsi, rdi
        mov rdi, rax
        
        call syscall_handler_inner
        
        // Restore caller-saved registers
        pop r10
        pop r9
        pop r8
        pop rdx
        pop rsi
        pop rdi

        // RAX contains the syscall return value, must preserve it!
        push rax

        // Load User RIP/RSP/RFLAGS from the current Task struct
        call restore_user_context

        pop rax

        // 5. Restore User RIP and RFLAGS from CpuLocal (updated by restore_user_context)
        mov r11, gs:[24]
        mov rcx, gs:[16]
        
        // 6. Restore User RSP from CpuLocal
        mov rsp, gs:[8]
        
        // 7. Swap GS back to User
        swapgs
        
        sysretq
    "#
);

#[repr(C, packed)]
pub struct CpuLocal {
    pub kernel_rsp: u64,  // Offset 0
    pub user_rsp: u64,    // Offset 8
    pub user_rip: u64,    // Offset 16
    pub user_rflags: u64, // Offset 24
}

#[no_mangle]
pub static mut CPU_LOCAL: CpuLocal = CpuLocal {
    kernel_rsp: 0,
    user_rsp: 0,
    user_rip: 0,
    user_rflags: 0,
};

pub fn init_syscalls() {
    unsafe {
        let ptr = &raw const CPU_LOCAL as u64;
        // Set MSR_KERNEL_GS_BASE (0xC0000102)
        core::arch::asm!(
            "wrmsr",
            in("ecx") 0xC0000102u32,
            in("eax") (ptr & 0xFFFFFFFF) as u32,
            in("edx") (ptr >> 32) as u32,
        );
        // Set MSR_GS_BASE (0xC0000101) so both are initialized
        core::arch::asm!(
            "wrmsr",
            in("ecx") 0xC0000101u32,
            in("eax") (ptr & 0xFFFFFFFF) as u32,
            in("edx") (ptr >> 32) as u32,
        );
    }
}

/// Called from syscall_entry after saving RIP/RSP/RFLAGS into CPU_LOCAL.
/// Copies those values into the current Task struct for safe per-task storage.
#[no_mangle]
pub extern "C" fn save_user_context() {
    unsafe {
        let rip = CPU_LOCAL.user_rip;
        let rsp = CPU_LOCAL.user_rsp;
        let rflags = CPU_LOCAL.user_rflags;

        let cpu_id = crate::cpu::id::current_cpu_id();
        if let Some(task_id) = crate::cpu::CPUS[cpu_id].lock().current_task {
            let mut mgr = crate::task::TASK_MANAGER.lock();
            if let Some(task) = mgr.pool.get_task_mut(task_id) {
                task.saved_user_rip = rip;
                task.saved_user_rsp = rsp;
                task.saved_user_rflags = rflags;
            }
        }
    }
}

/// Called from syscall_entry just before sysretq.
/// Copies the per-task saved user context back into CPU_LOCAL so that sysretq
/// returns to the correct user-space address even after a blocking sleep.
#[no_mangle]
pub extern "C" fn restore_user_context() {
    unsafe {
        let cpu_id = crate::cpu::id::current_cpu_id();
        if let Some(task_id) = crate::cpu::CPUS[cpu_id].lock().current_task {
            let mgr = crate::task::TASK_MANAGER.lock();
            if let Some(task) = mgr.pool.get_task(task_id) {
                CPU_LOCAL.user_rip    = task.saved_user_rip;
                CPU_LOCAL.user_rsp    = task.saved_user_rsp;
                CPU_LOCAL.user_rflags = task.saved_user_rflags;
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn syscall_handler_inner(sys_num: u64, arg1: u64, arg2: u64, arg3: u64) -> i64 {
    unsafe {
        crate::uart_puts(b"  -> [syscall_handler] sys_num: ");
        let mut n_buf = [b'0'; 2];
        let n = sys_num as u8;
        let n1 = n >> 4;
        let n2 = n & 0xF;
        n_buf[0] = if n1 < 10 { b'0' + n1 } else { b'A' + (n1 - 10) };
        n_buf[1] = if n2 < 10 { b'0' + n2 } else { b'A' + (n2 - 10) };
        crate::uart_puts(&n_buf);
        crate::uart_puts(b"\n");
    }
    match sys_num {
        SYS_YIELD => {
            sys_yield();
            0
        }
        SYS_SLEEP => {
            sys_sleep(arg1);
            0
        }
        SYS_EXIT => {
            sys_exit();
            0
        }
        SYS_OPEN    => sys_open(arg1, arg2),
        SYS_READ    => {
            unsafe { crate::uart_puts(b"  -> [syscall_dispatcher] dispatching SYS_READ\n"); }
            sys_read(arg1, arg2, arg3)
        },
        SYS_WRITE   => sys_write(arg1, arg2, arg3),
        SYS_EXECVE  => sys_execve(arg1, arg2),
        SYS_IPC_SEND => sys_ipc_send(arg1, arg2, arg3),
        SYS_IPC_RECV => sys_ipc_recv(arg1, arg2),
        // Phase 17
        SYS_SOCKET   => sys_socket(),
        SYS_BIND     => sys_bind(arg1 as i64, arg2 as u16),
        SYS_SENDTO   => sys_sendto(arg1 as i64, arg2, arg3),
        SYS_RECVFROM => sys_recvfrom(arg1 as i64, arg2, arg3),
        SYS_CLOSE    => sys_close(arg1 as i64),
        SYS_TCP_SOCKET => sys_tcp_socket(),
        SYS_TCP_LISTEN => sys_tcp_listen(arg1 as i64, arg2 as u16),
        SYS_TCP_ACCEPT => sys_tcp_accept(arg1 as i64),
        SYS_TCP_CONNECT => sys_tcp_connect(arg1 as i64, arg2 as u32, arg3 as u16),
        _            => -38, // -ENOSYS
    }
}

pub fn sys_yield() {
    // In cooperative mode, triggering an interrupt is safer for context switching.
    // For now, since we have the scheduler preemption working via LAPIC,
    // we can either let it run, or trigger INT 32 directly (software interrupt)
    unsafe {
        core::arch::asm!("int 32"); // Trigger timer interrupt to force a reschedule
    }
}

pub fn sys_sleep(ms: u64) {
    let mut manager = TASK_MANAGER.lock();
    manager.sleep_current_task(ms);
    drop(manager);
    
    // Now trigger a reschedule since we put ourselves to sleep
    unsafe {
        core::arch::asm!("int 32");
    }
}

pub fn sys_exit() {
    // Current task has exited
    // For now, we just halt it
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

// Pointer validation moved to memory::user

pub fn sys_open(_path_ptr: u64, _path_len: u64) -> i64 {
    // For Phase 10, we focus on SYS_READ and SYS_EXECVE.
    // Full VFS SYS_OPEN will be implemented later.
    -38 // -ENOSYS
}

pub fn sys_read(fd: u64, buf_ptr: u64, len: u64) -> i64 {
    unsafe { crate::uart_puts(b"  -> [sys_read] started\n"); }
    if fd == 0 {
        // Use a fixed 256-byte stack buffer (no heap in kernel)
        let capped = (len as usize).min(256);
        let mut tmp_buf = [0u8; 256];
        
        use crate::input::InputDevice;
        let mut read = 0;
        
        while read < capped {
            if let Some(byte) = crate::input::KEYBOARD_DEVICE.read_event() {
                tmp_buf[read] = byte;
                read += 1;
            } else {
                if read == 0 {
                    // Block until input is available
                    unsafe { crate::uart_puts(b"  -> [sys_read] blocking...\n"); }
                    crate::input::KEYBOARD_DEVICE.wait_queue.lock().block_current(crate::task::state::BlockReason::Input);
                    unsafe { crate::uart_puts(b"  -> [sys_read] woke up!\n"); }
                    // After waking up, the loop will retry reading
                } else {
                    break;
                }
            }
        }
        if read > 0 {
            if crate::memory::user::copy_to_user(buf_ptr, &tmp_buf[..read], read).is_err() {
                return -22;
            }
        }
        unsafe { crate::uart_puts(b"  -> [sys_read] returning\n"); }
        return read as i64;
    } else if fd >= 3 && fd < 16 {
        let mut task_mgr = crate::task::TASK_MANAGER.lock();
        let cpu_id = crate::cpu::id::current_cpu_id();
        let current_id_opt = crate::cpu::CPUS[cpu_id].lock().current_task;
        
        if let Some(task_id) = current_id_opt {
            if let Some(task) = task_mgr.pool.get_task_mut(task_id) {
                let handle = task.fd_table[fd as usize];
                drop(task_mgr); // Release lock before blocking/reading
                
                if handle.is_valid() {
                    let pool = crate::objects::OBJECT_POOL.lock();
                    let variant = pool.get_variant(handle.object_id());
                    drop(pool);
                    
                    if let Some(var) = variant {
                        match var {
                            crate::objects::ObjectVariant::File(ref node) => {
                                // 4KB read buffer on the stack
                                let capped = (len as usize).min(4096);
                                let mut tmp_buf = [0u8; 4096];
                                let vfs_mgr = crate::fs::VFS_MANAGER.lock();
                                if let Ok(bytes) = vfs_mgr.read_file(node, &mut tmp_buf[..capped]) {
                                    if crate::memory::user::copy_to_user(buf_ptr, &tmp_buf[..bytes], bytes).is_ok() {
                                        return bytes as i64;
                                    }
                                }
                                return -5; // EIO
                            }
                            crate::objects::ObjectVariant::UdpSocket(idx) => {
                                let mut sock_mgr = crate::net::socket::SOCKET_MANAGER.lock();
                                if let Some(dg) = sock_mgr.try_recv(idx) {
                                    let clen = dg.len.min(len as usize);
                                    if crate::memory::user::copy_to_user(buf_ptr, &dg.data, clen).is_ok() {
                                        return clen as i64;
                                    }
                                }
                                return -11; // EAGAIN
                            }
                            _ => return -38, // ENOSYS
                        }
                    }
                }
            }
        }
    }
    
    -9 // -EBADF
}

pub fn sys_write(fd: u64, buf_ptr: u64, len: u64) -> i64 {
    // Fixed 4KB stack buffer — no heap allocator in kernel
    let capped = (len as usize).min(4096);
    let mut tmp_buf = [0u8; 4096];
    if crate::memory::user::copy_from_user(&mut tmp_buf, buf_ptr, capped).is_err() {
        return -22; // EINVAL
    }

    if fd == 1 || fd == 2 {
        unsafe {
            crate::uart_puts(b"[sys_write] len: ");
            if len > 0 { crate::uart_puts(b"got data\n"); }
        }
        if let Ok(s) = core::str::from_utf8(&tmp_buf[..capped]) {
            unsafe { crate::uart_puts(s.as_bytes()) };
            crate::desktop::DESKTOP_MANAGER.lock().write_to_terminal(s);
            crate::input::INPUT_MANAGER.wait_queue.lock().wake_one(); 
            return len as i64;
        } else {
            unsafe { crate::uart_puts(b"[sys_write] UTF8 ERROR\n") };
            return len as i64;
        }
    } else if fd >= 3 && fd < 16 {
        let mut task_mgr = crate::task::TASK_MANAGER.lock();
        let cpu_id = crate::cpu::id::current_cpu_id();
        let current_id_opt = crate::cpu::CPUS[cpu_id].lock().current_task;
        
        if let Some(task_id) = current_id_opt {
            if let Some(task) = task_mgr.pool.get_task_mut(task_id) {
                let handle = task.fd_table[fd as usize];
                drop(task_mgr); 
                
                if handle.is_valid() {
                    let pool = crate::objects::OBJECT_POOL.lock();
                    let variant = pool.get_variant(handle.object_id());
                    drop(pool);
                    
                    if let Some(var) = variant {
                        match var {
                            crate::objects::ObjectVariant::File(_) => {
                                return -30; // EROFS
                            }
                            _ => return -38, // ENOSYS
                        }
                    }
                }
            }
        }
    }
    -9 // -EBADF
}

pub fn sys_execve(path_ptr: u64, path_len: u64) -> i64 {
    unsafe { crate::uart_puts(b"  -> [sys_execve] started\n"); }
    if path_len > 256 { return -22; } // path too long
    let mut tmp_buf = [0u8; 256];
    if crate::memory::user::copy_from_user(&mut tmp_buf, path_ptr, path_len as usize).is_err() {
        return -22; // EINVAL
    }

    let path = match core::str::from_utf8(&tmp_buf[..path_len as usize]) {
        Ok(s) => s,
        Err(_) => return -22,
    };

    
    unsafe { crate::uart_puts(b"  -> [sys_execve] opening node\n"); }
    let vfs_manager = crate::fs::VFS_MANAGER.lock();
    let node = match vfs_manager.open(path) {
        Some(n) => n,
        None => return -2, // -ENOENT
    };

    
    // Calculate pages needed for the entire file (max 1MB for this phase to avoid exhaustion)
    if node.size > 1024 * 1024 {
        return -27; // -EFBIG
    }

    
    unsafe { crate::uart_puts(b"  -> [sys_execve] allocating frames\n"); }
    let pages_needed = (node.size as u64 + 4095) / 4096;
    let mut frame_alloc = crate::memory::frame::FRAME_ALLOCATOR.lock();
    let file_buffer_addr = match frame_alloc.allocate_frames(pages_needed) {
        Some(addr) => addr,
        None => return -12, // -ENOMEM
    };
    drop(frame_alloc);

    unsafe { crate::uart_puts(b"  -> [sys_execve] reading file\n"); }
    // Read the file into the physically contiguous buffer
    let file_buffer = unsafe {
        core::slice::from_raw_parts_mut(file_buffer_addr as *mut u8, node.size as usize)
    };

    if vfs_manager.read_file(&node, file_buffer).is_err() {
        return -5; // -EIO
    }
    drop(vfs_manager);

    unsafe { crate::uart_puts(b"  -> [sys_execve] cloning pml4\n"); }
    let new_pml4 = match crate::memory::mapper::clone_kernel_pml4() {
        Some(addr) => addr,
        None => return -12, // -ENOMEM
    };
    
    let mut mapper = crate::memory::mapper::Mapper::new(new_pml4);
    
    unsafe { crate::uart_puts(b"  -> [sys_execve] loading ELF\n"); }
    let entry_point = match crate::task::elf::load_elf(file_buffer, &mut mapper) {
        Ok(ep) => ep,
        Err(_) => return -8, // -ENOEXEC
    };

    unsafe { crate::uart_puts(b"  -> [sys_execve] mapping stack\n"); }
    // Allocate 8KB User Stack (2 pages), Guard Page is implicitly unmapped below it
    let mut alloc = crate::memory::frame::FRAME_ALLOCATOR.lock();
    let stack_frames = alloc.allocate_frames(2).unwrap();
    drop(alloc);

    let user_stack_vaddr = 0x8000_0000;
    use xparq_hal::x86_64::paging::PageTableFlags;
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;
    
    let _ = mapper.map_page(user_stack_vaddr / 4096, stack_frames / 4096, flags);
    let _ = mapper.map_page((user_stack_vaddr + 4096) / 4096, (stack_frames + 4096) / 4096, flags);

    unsafe { crate::uart_puts(b"  -> [sys_execve] assigning new pml4 to task\n"); }
    let cpu_id = crate::cpu::id::current_cpu_id();
    let cpu = crate::cpu::CPUS[cpu_id].lock();
    if let Some(task_id) = cpu.current_task {
        let mut manager = TASK_MANAGER.lock();
        if let Some(task) = manager.pool.get_task_mut(task_id) {
            task.pml4_addr = new_pml4;
        }
    }
    drop(cpu);

    unsafe { crate::uart_puts(b"  -> [sys_execve] switching cr3\n"); }
    // Switch to new PML4 immediately so returning to user space works
    unsafe {
        core::arch::asm!("mov cr3, {}", in(reg) new_pml4);
    }

    unsafe { crate::uart_puts(b"  -> [sys_execve] setting user rip/rsp\n"); }
    // Reset User RIP and RSP on the CpuLocal
    unsafe {
        crate::uart_puts(b"  -> [sys_execve] setting user rip: ");
        let mut ep_buf = [b'0'; 16];
        let mut ep = entry_point;
        for i in 0..16 {
            let nibble = (ep >> (60 - i * 4)) & 0xF;
            ep_buf[i] = if nibble < 10 { b'0' + nibble as u8 } else { b'A' + (nibble - 10) as u8 };
        }
        crate::uart_puts(&ep_buf);
        crate::uart_puts(b"\n");
        CPU_LOCAL.user_rip = entry_point;
        CPU_LOCAL.user_rsp = user_stack_vaddr + 8192; // Top of the 8KB stack
    }

    unsafe { crate::uart_puts(b"  -> [sys_execve] done\n"); }
    // When we return, syscall_entry will pop our modified RIP/RSP and jump to the new ELF!
    0
}

pub fn sys_ipc_send(target_pid: u64, type_: u64, data_ptr: u64) -> i64 {
    if !crate::memory::user::validate_user_ptr(data_ptr, 32) {
        return -22; // -EINVAL
    }

    use crate::ipc::Message;
    use crate::task::id::TaskId;
    use crate::task::state::BlockReason;

    let target = TaskId::new(target_pid as usize);
    
    let cpu_id = crate::cpu::id::current_cpu_id();
    let current_task_id = match crate::cpu::CPUS[cpu_id].lock().current_task {
        Some(id) => id,
        None => return -22,
    };
    let mut manager = TASK_MANAGER.lock();

    let slice = unsafe { core::slice::from_raw_parts(data_ptr as *const u8, 32) };
    let mut data = [0u8; 32];
    data.copy_from_slice(slice);

    let message = Message {
        sender: current_task_id,
        type_: type_ as u32,
        data,
    };

    let target_task = match manager.pool.get_task_mut(target) {
        Some(t) => t,
        None => return -3, // -ESRCH
    };

    let mut mailbox = target_task.ipc_mailbox.lock();
    
    // Check backpressure
    if mailbox.messages.count == 32 { // Full
        if mailbox.mode == crate::ipc::IpcMode::Blocking {
            mailbox.wait_queue.block_current(BlockReason::Ipc);
            return -11; // -EAGAIN (retry in userspace)
        } else {
            return -11; // -EAGAIN
        }
    }

    mailbox.messages.push(message);
    mailbox.wait_queue.wake_one(); // Wake up target if it was waiting
    
    0
}

pub fn sys_ipc_recv(type_filter: u64, buf_ptr: u64) -> i64 {
    if !crate::memory::user::validate_user_ptr(buf_ptr, core::mem::size_of::<crate::ipc::Message>() as u64) {
        return -22; // -EINVAL
    }

    use crate::task::state::BlockReason;
    
    let cpu_id = crate::cpu::id::current_cpu_id();
    let current_task_id = match crate::cpu::CPUS[cpu_id].lock().current_task {
        Some(id) => id,
        None => return -22,
    };
    let mut manager = TASK_MANAGER.lock();

    let task = match manager.pool.get_task_mut(current_task_id) {
        Some(t) => t,
        None => return -22,
    };

    let mut mailbox = task.ipc_mailbox.lock();
    
    if mailbox.messages.count == 0 {
        if mailbox.mode == crate::ipc::IpcMode::Blocking {
            mailbox.wait_queue.block_current(BlockReason::Ipc);
            return -11; // -EAGAIN to prompt user-space retry
        } else {
            return -11; // -EAGAIN
        }
    }

    // Ideally we should filter by `type_filter`, but we just pop the first message for simplicity
    if let Some(msg) = mailbox.messages.pop() {
        mailbox.wait_queue.wake_one(); // Wake up any blocked senders

        unsafe {
            core::ptr::write(buf_ptr as *mut crate::ipc::Message, msg);
        }
        return 0;
    }

    -11
}

// ── Phase 17: Socket Syscall Implementations ──────────────────────────────────

/// SYS_SOCKET — Allocate a new UDP socket.
/// Returns socket file descriptor (>= 100) on success, or negative errno.
pub fn sys_socket() -> i64 {
    let sock_idx = crate::net::socket::SOCKET_MANAGER.lock().alloc_socket();
    if sock_idx < 0 {
        return sock_idx; // EMFILE from socket pool
    }
    
    // Allocate FD in the current task
    let mut task_mgr = crate::task::TASK_MANAGER.lock();
    let cpu_id = crate::cpu::id::current_cpu_id();
    let current_id_opt = crate::cpu::CPUS[cpu_id].lock().current_task;
    
    if let Some(task_id) = current_id_opt {
        if let Some(task) = task_mgr.pool.get_task_mut(task_id) {
            for i in 3..16 { // Reserve FD 0, 1, 2
                if !task.fd_table[i].is_valid() {
                    let mut pool = crate::objects::OBJECT_POOL.lock();
                    if let Some(obj_id) = pool.allocate(crate::objects::ObjectVariant::UdpSocket(sock_idx as usize)) {
                        task.fd_table[i] = crate::objects::Handle::new(obj_id, crate::objects::HandleRights::ALL);
                        return i as i64;
                    }
                }
            }
        }
    }
    
    // Fallback if FD table full (should free socket too, but simplified for now)
    crate::net::socket::SOCKET_MANAGER.lock().close_socket(sock_idx as usize);
    -24 // -EMFILE
}

/// Get underlying socket index from task FD
fn get_sock_idx(fd: i64) -> Option<usize> {
    if fd < 3 || fd >= 16 { return None; }
    let task_mgr = crate::task::TASK_MANAGER.lock();
    let cpu_id = crate::cpu::id::current_cpu_id();
    let current_id_opt = crate::cpu::CPUS[cpu_id].lock().current_task;
    
    if let Some(task_id) = current_id_opt {
        if let Some(task) = task_mgr.pool.get_task(task_id) {
            let handle = task.fd_table[fd as usize];
            if handle.is_valid() {
                let pool = crate::objects::OBJECT_POOL.lock();
                if let Some(crate::objects::ObjectVariant::UdpSocket(idx)) = pool.get_variant(handle.object_id()) {
                    return Some(idx);
                }
            }
        }
    }
    None
}

/// SYS_BIND — Bind a socket to a local UDP port.
/// arg1: sockfd, arg2: port (u16 as u64)
pub fn sys_bind(sockfd: i64, port: u16) -> i64 {
    if let Some(sock_idx) = get_sock_idx(sockfd) {
        crate::net::socket::SOCKET_MANAGER.lock().bind_socket(sock_idx, port)
    } else {
        -9 // EBADF
    }
}

/// SYS_SENDTO — Send a UDP datagram.
pub fn sys_sendto(sockfd: i64, buf_ptr: u64, arg3: u64) -> i64 {
    let sock_idx = match get_sock_idx(sockfd) {
        Some(idx) => idx,
        None => return -9, // EBADF
    };

    let dst_port = (arg3 >> 16) as u16;
    let payload_len = (arg3 & 0xFFFF) as usize;

    if !crate::memory::user::validate_user_ptr(buf_ptr, payload_len as u64) {
        return -22; // -EINVAL
    }

    let sock_mgr = crate::net::socket::SOCKET_MANAGER.lock();
    let idx = sock_idx;
    if idx >= crate::net::socket::MAX_SOCKETS || !sock_mgr.sockets[idx].in_use {
        return -9; // -EBADF
    }
    let src_port = sock_mgr.sockets[idx].bound_port;
    drop(sock_mgr);

    let mut payload_buf = [0u8; 1500];
    let payload_len_capped = payload_len.min(1500);
    if crate::memory::user::copy_from_user(&mut payload_buf, buf_ptr, payload_len_capped).is_err() {
        return -22;
    }
    let payload = &payload_buf[..payload_len_capped];

    // Default destination: QEMU user net gateway
    let dst_ip: [u8; 4] = [10, 0, 2, 2];
    let our_mac = crate::net::NETWORK_MANAGER.lock().mac_address;
    // Destination MAC: broadcast for simplicity (ARP not yet integrated into sendto path)
    let dst_mac: [u8; 6] = [0xFF; 6];

    let mut udp_buf = [0u8; 1500];
    let udp_len = crate::net::udp::UdpPacket::serialize(
        crate::net::OUR_IP,
        dst_ip,
        src_port,
        dst_port,
        payload,
        &mut udp_buf,
    );
    if udp_len == 0 {
        return -22;
    }

    let ip_pkt = crate::net::ipv4::Ipv4Packet {
        protocol: crate::net::ipv4::IP_PROTOCOL_UDP,
        src_ip: crate::net::OUR_IP,
        dest_ip: dst_ip,
        payload: &udp_buf[0..udp_len],
    };
    let mut ip_buf = [0u8; 1500];
    let ip_len = ip_pkt.to_bytes(&mut ip_buf);
    if ip_len == 0 {
        return -22;
    }

    let mut eth_buf = [0u8; 1514];
    let eth_len = crate::net::ethernet::EthernetFrame::to_bytes(
        dst_mac,
        our_mac,
        crate::net::ethernet::ETHERTYPE_IPV4,
        &ip_buf[0..ip_len],
        &mut eth_buf,
    );

    let mut e1000 = xparq_hal::x86_64::e1000::E1000_DRIVER.lock();
    use xparq_hal::connectivity::ConnectivityDriver;
    match e1000.send(&eth_buf[0..eth_len]) {
        Ok(n) => n as i64,
        Err(_) => -5, // -EIO
    }
}

/// SYS_RECVFROM — Blocking receive from a UDP socket.
/// arg1: sockfd, arg2: buf_ptr, arg3: buf_len
/// Returns number of bytes written into buf on success, negative errno on failure.
/// Blocks the calling task if no data is available.
pub fn sys_recvfrom(sockfd: i64, buf_ptr: u64, buf_len: u64) -> i64 {
    if !crate::memory::user::validate_user_ptr(buf_ptr, buf_len) {
        return -22; // -EINVAL
    }

    let sock_idx = match get_sock_idx(sockfd) {
        Some(idx) => idx,
        None => return -9, // EBADF
    };

    let idx = sock_idx;
    if idx >= crate::net::socket::MAX_SOCKETS {
        return -9; // -EBADF
    }

    loop {
        {
            let mut sock_mgr = crate::net::socket::SOCKET_MANAGER.lock();
            if !sock_mgr.sockets[idx].in_use {
                return -9; // -EBADF
            }
            if let Some(datagram) = sock_mgr.sockets[idx].pop_datagram() {
                let copy_len = datagram.len.min(buf_len as usize);
                if crate::memory::user::copy_to_user(buf_ptr, &datagram.data[..datagram.len], copy_len).is_ok() {
                    return copy_len as i64;
                }
                return -22;
            }
        } // release lock before blocking

        // No data yet — block until delivery wakes us
        {
            let mut sock_mgr = crate::net::socket::SOCKET_MANAGER.lock();
            if !sock_mgr.sockets[idx].in_use {
                return -9;
            }
            sock_mgr.sockets[idx].wait_queue.block_current(
                crate::task::state::BlockReason::Input,
            );
        }
        // After being woken, loop back and try again
    }
}

pub fn sys_close(fd: i64) -> i64 {
    if fd < 3 || fd >= 16 { return -9; }
    
    let mut task_mgr = crate::task::TASK_MANAGER.lock();
    let cpu_id = crate::cpu::id::current_cpu_id();
    let current_id_opt = crate::cpu::CPUS[cpu_id].lock().current_task;
    
    if let Some(task_id) = current_id_opt {
        if let Some(task) = task_mgr.pool.get_task_mut(task_id) {
            let handle = task.fd_table[fd as usize];
            if !handle.is_valid() {
                return -9; // EBADF
            }
            task.fd_table[fd as usize] = crate::objects::Handle::INVALID;
            drop(task_mgr); 
            
            let mut pool = crate::objects::OBJECT_POOL.lock();
            let variant = pool.get_variant(handle.object_id());
            pool.release(handle.object_id());
            drop(pool);
            
            if let Some(var) = variant {
                match var {
                    crate::objects::ObjectVariant::None => return -9, 
                    crate::objects::ObjectVariant::File(_) => return 0, 
                    crate::objects::ObjectVariant::UdpSocket(idx) => {
                        return crate::net::socket::SOCKET_MANAGER.lock().close_socket(idx);
                    }
                    crate::objects::ObjectVariant::TcpSocket(idx) => {
                        return crate::net::tcp::TCP_SOCKET_MANAGER.lock().close_socket(idx);
                    }
                }
            }
        }
    }
    -9 // EBADF
}

// ── Phase 18: TCP Socket Syscalls ─────────────────────────────────────────────

pub fn sys_tcp_socket() -> i64 {
    let sock_idx = crate::net::tcp::TCP_SOCKET_MANAGER.lock().alloc_socket();
    if sock_idx < 0 { return sock_idx; }
    
    let mut task_mgr = crate::task::TASK_MANAGER.lock();
    let cpu_id = crate::cpu::id::current_cpu_id();
    let current_id_opt = crate::cpu::CPUS[cpu_id].lock().current_task;
    
    if let Some(task_id) = current_id_opt {
        if let Some(task) = task_mgr.pool.get_task_mut(task_id) {
            for i in 3..16 {
                if !task.fd_table[i].is_valid() {
                    let mut pool = crate::objects::OBJECT_POOL.lock();
                    if let Some(obj_id) = pool.allocate(crate::objects::ObjectVariant::TcpSocket(sock_idx as usize)) {
                        task.fd_table[i] = crate::objects::Handle::new(obj_id, crate::objects::HandleRights::ALL);
                        return i as i64;
                    }
                }
            }
        }
    }
    
    crate::net::tcp::TCP_SOCKET_MANAGER.lock().close_socket(sock_idx as usize);
    -24 // -EMFILE
}

pub fn sys_tcp_listen(fd: i64, port: u16) -> i64 {
    if fd < 3 || fd >= 16 { return -9; }
    
    let task_mgr = crate::task::TASK_MANAGER.lock();
    let cpu_id = crate::cpu::id::current_cpu_id();
    let current_id_opt = crate::cpu::CPUS[cpu_id].lock().current_task;
    
    if let Some(task_id) = current_id_opt {
        if let Some(task) = task_mgr.pool.get_task(task_id) {
            let handle = task.fd_table[fd as usize];
            if handle.is_valid() {
                let pool = crate::objects::OBJECT_POOL.lock();
                if let Some(crate::objects::ObjectVariant::TcpSocket(idx)) = pool.get_variant(handle.object_id()) {
                    drop(pool);
                    drop(task_mgr);
                    let mut tcp_mgr = crate::net::tcp::TCP_SOCKET_MANAGER.lock();
                    
                    let res = tcp_mgr.bind(idx, port);
                    if res < 0 { return res; }
                    
                    return tcp_mgr.listen(idx);
                }
            }
        }
    }
    -9 // EBADF
}

pub fn sys_tcp_accept(fd: i64) -> i64 {
    if fd < 3 || fd >= 16 { return -9; }

    loop {
        let mut task_mgr = crate::task::TASK_MANAGER.lock();
        let cpu_id = crate::cpu::id::current_cpu_id();
        let current_id_opt = crate::cpu::CPUS[cpu_id].lock().current_task;

        let mut block_reason = None;

        if let Some(task_id) = current_id_opt {
            if let Some(task) = task_mgr.pool.get_task_mut(task_id) {
                let handle = task.fd_table[fd as usize];
                if handle.is_valid() {
                    let pool = crate::objects::OBJECT_POOL.lock();
                    let maybe_tcp = pool.get_variant(handle.object_id());
                    drop(pool);
                    if let Some(crate::objects::ObjectVariant::TcpSocket(idx)) = maybe_tcp {
                    drop(task_mgr); // drop before locking network

                    let mut tcp_mgr = crate::net::tcp::TCP_SOCKET_MANAGER.lock();
                    if let Some(child_idx) = tcp_mgr.accept(idx) {
                        // Found a child socket! Map it to a new FD.
                        drop(tcp_mgr);
                        
                        let mut task_mgr2 = crate::task::TASK_MANAGER.lock();
                        if let Some(task2) = task_mgr2.pool.get_task_mut(task_id) {
                            for i in 3..16 {
                                if !task2.fd_table[i].is_valid() {
                                    let mut pool = crate::objects::OBJECT_POOL.lock();
                                    if let Some(obj_id) = pool.allocate(crate::objects::ObjectVariant::TcpSocket(child_idx)) {
                                        task2.fd_table[i] = crate::objects::Handle::new(obj_id, crate::objects::HandleRights::ALL);
                                        return i as i64;
                                    }
                                }
                            }
                        }
                        
                        // No FD available, close child
                        crate::net::tcp::TCP_SOCKET_MANAGER.lock().close_socket(child_idx);
                        return -24; // EMFILE
                    }
                    
                    // No child ready, prepare to block
                    block_reason = Some(idx);
                    }
                }
            }
        }

        if let Some(idx) = block_reason {
            let mut tcp_mgr = crate::net::tcp::TCP_SOCKET_MANAGER.lock();
            tcp_mgr.sockets[idx].wait_queue.block_current(crate::task::state::BlockReason::Network);
            drop(tcp_mgr);
        } else {
            return -9; // EBADF
        }
    }
}

pub fn sys_tcp_connect(fd: i64, ip_u32: u32, port: u16) -> i64 {
    if fd < 3 || fd >= 16 { return -9; }
    
    let ip = ip_u32.to_be_bytes();
    
    let task_mgr = crate::task::TASK_MANAGER.lock();
    let cpu_id = crate::cpu::id::current_cpu_id();
    let current_id_opt = crate::cpu::CPUS[cpu_id].lock().current_task;
    
    if let Some(task_id) = current_id_opt {
        if let Some(task) = task_mgr.pool.get_task(task_id) {
            let handle = task.fd_table[fd as usize];
            if handle.is_valid() {
                let pool = crate::objects::OBJECT_POOL.lock();
                if let Some(crate::objects::ObjectVariant::TcpSocket(idx)) = pool.get_variant(handle.object_id()) {
                    drop(pool);
                    drop(task_mgr);
                
                    let mut tcp_mgr = crate::net::tcp::TCP_SOCKET_MANAGER.lock();
                    let res = tcp_mgr.connect(idx, ip, port);
                    if res < 0 { return res; }
                
                    // Block until connected
                    tcp_mgr.sockets[idx].wait_queue.block_current(crate::task::state::BlockReason::Network);
                    drop(tcp_mgr);
                
                    // After waking up, check state
                    let state = crate::net::tcp::TCP_SOCKET_MANAGER.lock().sockets[idx].state;
                    if state == crate::net::tcp::TcpState::Established {
                        return 0;
                    } else {
                        return -111; // ECONNREFUSED
                    }
                }
            }
        }
    }
    -9
}
