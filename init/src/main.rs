#![no_std]
#![no_main]

use core::panic::PanicInfo;

// ── Syscall Numbers (must match kernel/src/syscall/dispatcher.rs) ─────────────
const SYS_YIELD: u64 = 1;
const SYS_SLEEP: u64 = 2;
const SYS_EXIT: u64 = 3;
const SYS_OPEN: u64 = 4;
const SYS_READ: u64 = 5;
const SYS_WRITE: u64 = 6;
const SYS_EXECVE: u64 = 7;
const SYS_IPC_SEND: u64 = 8;
const SYS_IPC_RECV: u64 = 9;
const SYS_SOCKET: u64 = 10;
const SYS_BIND: u64 = 11;
const SYS_SENDTO: u64 = 12;
const SYS_RECVFROM: u64 = 13;
const SYS_CLOSE: u64 = 14;
const SYS_TCP_SOCKET: u64 = 15;
const SYS_TCP_LISTEN: u64 = 16;
const SYS_TCP_ACCEPT: u64 = 17;
const SYS_TCP_CONNECT: u64 = 18;
const SYS_GETPID: u64 = 19;

#[repr(C)]
struct IpcMessage {
    sender: usize,
    type_: u32,
    data: [u8; 32],
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    let msg = b"Init Panic!\n";
    unsafe { syscall3(SYS_WRITE, 1, msg.as_ptr() as u64, msg.len() as u64) };
    loop {}
}

// ── Raw syscall wrappers ──────────────────────────────────────────────────────

unsafe fn syscall0(n: u64) -> i64 {
    let mut ret: i64;
    core::arch::asm!(
        "syscall",
        in("rax") n,
        out("rcx") _,
        out("r11") _,
        lateout("rax") ret,
        options(nostack)
    );
    ret
}

#[inline(always)]
unsafe fn syscall1(n: u64, a1: u64) -> i64 {
    let mut ret: i64;
    core::arch::asm!(
        "syscall",
        in("rax") n,
        in("rdi") a1,
        out("rcx") _,
        out("r11") _,
        lateout("rax") ret,
        options(nostack)
    );
    ret
}

#[inline(always)]
unsafe fn syscall2(n: u64, a1: u64, a2: u64) -> i64 {
    let mut ret: i64;
    core::arch::asm!(
        "syscall",
        in("rax") n,
        in("rdi") a1,
        in("rsi") a2,
        out("rcx") _,
        out("r11") _,
        lateout("rax") ret,
        options(nostack)
    );
    ret
}

#[inline(always)]
unsafe fn syscall3(n: u64, a1: u64, a2: u64, a3: u64) -> i64 {
    let mut ret: i64;
    core::arch::asm!(
        "syscall",
        in("rax") n,
        in("rdi") a1,
        in("rsi") a2,
        in("rdx") a3,
        out("rcx") _,
        out("r11") _,
        lateout("rax") ret,
        options(nostack)
    );
    ret
}

// ── High-level helpers ────────────────────────────────────────────────────────

fn print(s: &str) {
    unsafe { syscall3(SYS_WRITE, 1, s.as_ptr() as u64, s.len() as u64) };
}

fn print_bytes(b: &[u8]) {
    unsafe { syscall3(SYS_WRITE, 1, b.as_ptr() as u64, b.len() as u64) };
}

fn read_char() -> Option<u8> {
    let mut c = [0u8; 1];
    let ret = unsafe { syscall3(SYS_READ, 0, c.as_mut_ptr() as u64, 1) };
    if ret > 0 { Some(c[0]) } else { None }
}

fn yield_now() {
    unsafe { syscall0(SYS_YIELD) };
}

fn sleep_ms(ms: u64) -> i64 {
    unsafe { syscall1(SYS_SLEEP, ms) }
}

fn open_file(path: &str) -> i64 {
    unsafe { syscall2(SYS_OPEN, path.as_ptr() as u64, path.len() as u64) }
}

fn close_fd(fd: i64) -> i64 {
    unsafe { syscall1(SYS_CLOSE, fd as u64) }
}

fn getpid() -> i64 {
    unsafe { syscall0(SYS_GETPID) }
}

#[cfg(any(feature = "gate1-test", feature = "gate1-fault-test"))]
fn gate1_fail(marker: &str) -> ! {
    print(marker);
    // Remain in Ring 3 after emitting the failure marker.
    loop { core::hint::spin_loop(); }
}

#[cfg(feature = "gate1-test")]
fn run_gate1_acceptance() -> ! {
    let write_probe = b"XPARQ_TEST:GATE1:WRITE_OK\n";
    let written = unsafe { syscall3(SYS_WRITE, 1, write_probe.as_ptr() as u64, write_probe.len() as u64) };
    if written != write_probe.len() as i64 { gate1_fail("XPARQ_TEST:GATE1_FAIL:WRITE\n"); }

    if sleep_ms(2) != 0 { gate1_fail("XPARQ_TEST:GATE1_FAIL:SLEEP\n"); }

    if open_file("MISSING.TXT") != -2 { gate1_fail("XPARQ_TEST:GATE1_FAIL:OPEN_MISSING\n"); }
    if unsafe { syscall3(SYS_READ, 15, 0, 0) } != -9 { gate1_fail("XPARQ_TEST:GATE1_FAIL:BAD_FD\n"); }
    if unsafe { syscall0(0xFFFF) } != -38 { gate1_fail("XPARQ_TEST:GATE1_FAIL:UNKNOWN_SYSCALL\n"); }
    print("XPARQ_TEST:GATE1:ERRORS_OK\n");

    let fd = open_file("GATE1.TXT");
    if fd < 3 { gate1_fail("XPARQ_TEST:GATE1_FAIL:OPEN\n"); }
    let mut file_buf = [0u8; 64];
    let read = unsafe { syscall3(SYS_READ, fd as u64, file_buf.as_mut_ptr() as u64, file_buf.len() as u64) };
    let expected = b"XPARQ_GATE1_FILE_OK\n";
    if read != expected.len() as i64 || &file_buf[..expected.len()] != expected {
        gate1_fail("XPARQ_TEST:GATE1_FAIL:FILE_CONTENT\n");
    }
    if close_fd(fd) != 0 { gate1_fail("XPARQ_TEST:GATE1_FAIL:CLOSE\n"); }
    if unsafe { syscall3(SYS_READ, fd as u64, file_buf.as_mut_ptr() as u64, 1) } != -9 {
        gate1_fail("XPARQ_TEST:GATE1_FAIL:CLOSED_FD\n");
    }
    print("XPARQ_TEST:GATE1:FILE_OK\n");

    let pid = getpid();
    if pid < 0 { gate1_fail("XPARQ_TEST:GATE1_FAIL:GETPID\n"); }
    let mut payload = [0u8; 32];
    payload[..12].copy_from_slice(b"gate1-ipc-ok");
    if unsafe { syscall3(SYS_IPC_SEND, pid as u64, 0x471, payload.as_ptr() as u64) } != 0 {
        gate1_fail("XPARQ_TEST:GATE1_FAIL:IPC_SEND\n");
    }
    let mut message = IpcMessage { sender: 0, type_: 0, data: [0; 32] };
    if unsafe { syscall2(SYS_IPC_RECV, 0x471, &mut message as *mut IpcMessage as u64) } != 0 {
        gate1_fail("XPARQ_TEST:GATE1_FAIL:IPC_RECV\n");
    }
    if message.sender != pid as usize || message.type_ != 0x471 || message.data != payload {
        gate1_fail("XPARQ_TEST:GATE1_FAIL:IPC_CONTENT\n");
    }
    print("XPARQ_TEST:GATE1:IPC_OK\n");
    print("XPARQ_TEST:GATE1_PASS\n");
    unsafe { syscall0(SYS_EXIT) };
    gate1_fail("XPARQ_TEST:GATE1_FAIL:EXIT_RETURNED\n");
}

fn print_u64(v: u64) {
    if v == 0 {
        print("0");
        return;
    }
    let mut buf = [b'0'; 20];
    let mut n = v;
    let mut i = 20;
    while n > 0 {
        i -= 1;
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    print_bytes(&buf[i..]);
}

// ── Socket API wrappers ───────────────────────────────────────────────────────

fn socket_udp() -> i64 {
    unsafe { syscall0(SYS_SOCKET) }
}

fn socket_bind(sockfd: i64, port: u16) -> i64 {
    unsafe { syscall3(SYS_BIND, sockfd as u64, port as u64, 0) }
}

fn socket_sendto(sockfd: i64, buf: &[u8], dst_port: u16) -> i64 {
    // arg3 encodes: dst_port in upper 16 bits | len in lower 16 bits
    let arg3 = ((dst_port as u64) << 16) | (buf.len() as u64 & 0xFFFF);
    unsafe { syscall3(SYS_SENDTO, sockfd as u64, buf.as_ptr() as u64, arg3) }
}

fn socket_recvfrom(sockfd: i64, buf: &mut [u8]) -> i64 {
    unsafe { syscall3(SYS_RECVFROM, sockfd as u64, buf.as_mut_ptr() as u64, buf.len() as u64) }
}

fn socket_close(sockfd: i64) -> i64 {
    unsafe { syscall1(SYS_CLOSE, sockfd as u64) }
}

// ── Shell ─────────────────────────────────────────────────────────────────────

fn tcp_socket() -> i64 {
    unsafe { syscall0(SYS_TCP_SOCKET) }
}

fn tcp_listen(fd: i64, port: u16) -> i64 {
    unsafe { syscall2(SYS_TCP_LISTEN, fd as u64, port as u64) }
}

fn tcp_accept(fd: i64) -> i64 {
    unsafe { syscall1(SYS_TCP_ACCEPT, fd as u64) }
}

fn tcp_connect(fd: i64, ip: u32, port: u16) -> i64 {
    unsafe { syscall3(SYS_TCP_CONNECT, fd as u64, ip as u64, port as u64) }
}

fn cmd_net(args: &str) {
    let args = args.trim();

    // usage: net send <port> <message>
    //        net recv <port>
    if args.starts_with("send ") {
        let rest = &args[5..];
        let space = rest.bytes().position(|b| b == b' ');
        if let Some(sp) = space {
            let port_str = &rest[..sp];
            let msg = &rest[sp + 1..];
            let port: u16 = port_str.bytes().fold(0u16, |acc, b| {
                if b >= b'0' && b <= b'9' { acc * 10 + (b - b'0') as u16 } else { acc }
            });
            let sockfd = socket_udp();
            if sockfd < 0 {
                print("net: failed to create socket\n");
                return;
            }
            socket_bind(sockfd, port + 10000); // ephemeral src port
            let sent = socket_sendto(sockfd, msg.as_bytes(), port);
            socket_close(sockfd);
            if sent >= 0 {
                print("net: sent ");
                print_u64(sent as u64);
                print(" bytes\n");
            } else {
                print("net: send failed\n");
            }
        } else {
            print("usage: net send <port> <message>\n");
        }
    } else if args.starts_with("recv ") {
        let port_str = args[5..].trim();
        let port: u16 = port_str.bytes().fold(0u16, |acc, b| {
            if b >= b'0' && b <= b'9' { acc * 10 + (b - b'0') as u16 } else { acc }
        });
        let sockfd = socket_udp();
        if sockfd < 0 {
            print("net: failed to create socket\n");
            return;
        }
        let ret = socket_bind(sockfd, port);
        if ret < 0 {
            print("net: bind failed\n");
            socket_close(sockfd);
            return;
        }
        print("net: listening on UDP port ");
        print_u64(port as u64);
        print("... (blocking)\n");

        let mut buf = [0u8; 256];
        let n = socket_recvfrom(sockfd, &mut buf);
        socket_close(sockfd);

        if n > 0 {
            print("net: received ");
            print_u64(n as u64);
            print(" bytes: ");
            print_bytes(&buf[..n as usize]);
            print("\n");
        } else {
            print("net: recv failed\n");
        }
    } else {
        print("usage:\n  net send <port> <message>\n  net recv <port>\n");
    }
}

fn cmd_tcp(args: &str) {
    let args = args.trim();
    if args.starts_with("listen ") {
        let port_str = &args[7..].trim();
        let port = port_str.bytes().fold(0u16, |acc, b| {
            if b >= b'0' && b <= b'9' { acc * 10 + (b - b'0') as u16 } else { acc }
        });
        
        let fd = tcp_socket();
        if fd < 0 {
            print("tcp: failed to create socket\n");
            return;
        }
        
        let ret = tcp_listen(fd, port);
        if ret < 0 {
            print("tcp: listen failed\n");
            socket_close(fd);
            return;
        }
        
        print("tcp: listening on port ");
        print_u64(port as u64);
        print("\ntcp: waiting for connection (blocking)...\n");
        
        let child_fd = tcp_accept(fd);
        if child_fd < 0 {
            print("tcp: accept failed\n");
            socket_close(fd);
            return;
        }
        
        print("tcp: connection established! (fd=");
        print_u64(child_fd as u64);
        print(")\ntcp: sending hello message...\n");
        
        let msg = b"Hello from XPARQ OS!\n";
        unsafe { syscall3(SYS_WRITE, child_fd as u64, msg.as_ptr() as u64, msg.len() as u64) };
        
        socket_close(child_fd);
        socket_close(fd);
        print("tcp: closed.\n");
    } else if args.starts_with("connect ") {
        let rest = &args[8..].trim();
        let space_idx = rest.bytes().position(|b| b == b' ').unwrap_or(0);
        if space_idx == 0 {
            print("usage: tcp connect <ip_u32> <port>\n");
            return;
        }
        
        let ip_str = &rest[..space_idx];
        let port_str = &rest[space_idx + 1..];
        
        let ip = ip_str.bytes().fold(0u32, |acc, b| {
            if b >= b'0' && b <= b'9' { acc * 10 + (b - b'0') as u32 } else { acc }
        });
        
        let port = port_str.bytes().fold(0u16, |acc, b| {
            if b >= b'0' && b <= b'9' { acc * 10 + (b - b'0') as u16 } else { acc }
        });
        
        let fd = tcp_socket();
        if fd < 0 {
            print("tcp: failed to create socket\n");
            return;
        }
        
        print("tcp: connecting to ip ");
        print_u64(ip as u64);
        print(" port ");
        print_u64(port as u64);
        print("...\n");
        
        let ret = tcp_connect(fd, ip, port);
        if ret < 0 {
            print("tcp: connect failed\n");
            socket_close(fd);
            return;
        }
        
        print("tcp: connected! (fd=");
        print_u64(fd as u64);
        print(")\n");
        
        // Wait for server to send something
        let mut buf = [0u8; 128];
        let n = unsafe { syscall3(SYS_READ, fd as u64, buf.as_mut_ptr() as u64, buf.len() as u64) };
        if n > 0 {
            print("tcp: received: ");
            print_bytes(&buf[..n as usize]);
            print("\n");
        }
        
        socket_close(fd);
    } else {
        print("usage:\n  tcp listen <port>\n  tcp connect <ip_u32> <port>\n");
    }
}

#[cfg(all(feature = "gate1-test", not(any(feature = "gate1-input-test", feature = "gate1-fault-test"))))]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    run_gate1_acceptance();
}

#[cfg(feature = "gate1-fault-test")]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    print("XPARQ_TEST:FAULT:ARMED\n");
    unsafe {
        core::ptr::read_volatile(0x0000_7000_0000_0000usize as *const u64);
    }
    gate1_fail("XPARQ_TEST:GATE1_FAIL:PAGE_FAULT_RETURNED\n");
}

#[cfg(all(feature = "gate1-input-test", not(feature = "gate1-fault-test")))]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    print("XPARQ_TEST:INIT_READY\n");
    // Input verification is complete before this test init enters Ring 3.
    // Remain there without using the privileged HLT instruction.
    loop { core::hint::spin_loop(); }
}

#[cfg(not(any(feature = "gate1-test", feature = "gate1-input-test", feature = "gate1-fault-test")))]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    print("XPARQ_TEST:INIT_READY\n");
    print("\x1B[2J\x1B[H"); // Clear screen
    print("===========================================\n");
    print("  XPARQ OS User Space Shell (Phase 17)    \n");
    print("===========================================\n");
    print("Type 'help' for available commands.\n\n");

    let mut line_buf = [0u8; 256];
    let mut len: usize;

    loop {
        print("XPARQ> ");
        len = 0;

        // Read a line from stdin
        loop {
            if let Some(c) = read_char() {
                if c == b'\n' || c == b'\r' {
                    print("\n");
                    break;
                } else if c == 8 || c == 127 {
                    // Backspace
                    if len > 0 {
                        len -= 1;
                        print("\x08 \x08");
                    }
                } else if len < line_buf.len() - 1 {
                    let s = [c];
                    unsafe { syscall3(SYS_WRITE, 1, s.as_ptr() as u64, 1) };
                    line_buf[len] = c;
                    len += 1;
                }
            } else {
                yield_now();
            }
        }

        if len == 0 {
            continue;
        }

        let cmd_str = core::str::from_utf8(&line_buf[..len]).unwrap_or("");
        let (cmd, args) = if let Some(sp) = cmd_str.bytes().position(|b| b == b' ') {
            (&cmd_str[..sp], &cmd_str[sp + 1..])
        } else {
            (cmd_str, "")
        };

        match cmd {
            "help" => {
                print("Available commands:\n");
                print("  help               - Show this help\n");
                print("  echo <text>        - Print text\n");
                print("  clear              - Clear screen\n");
                print("  net send <port> <msg> - Send UDP datagram to gateway\n");
                print("  net recv <port>       - Receive one UDP datagram (blocking)\n");
                print("  tcp listen <port>     - Listen for TCP connection\n");
                print("  exit               - Exit shell\n");
            }
            "echo" => {
                print(args);
                print("\n");
            }
            "clear" => {
                print("\x1B[2J\x1B[H");
            }
            "net" => {
                cmd_net(args);
            }
            "tcp" => {
                cmd_tcp(args);
            }
            "exit" => {
                print("Goodbye!\n");
                unsafe { syscall0(SYS_EXIT) };
                loop {}
            }
            _ => {
                print("Unknown command: ");
                print(cmd);
                print("\nType 'help' for available commands.\n");
            }
        }
    }
}
