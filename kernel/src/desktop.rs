// XPARQ OS - Phase 3.5 Desktop GUI
use crate::hal::x86_64::display::X86Display;
use crate::hal::input::{InputEvent, InputEventData};
use crate::hal;

use spin::Mutex;
use crate::task::wait_queue::WaitQueue;
#[cfg(feature = "gate1-gui-test")]
use core::sync::atomic::{AtomicBool, Ordering};

pub const MAX_WINDOWS: usize = 4;

pub static DESKTOP_MANAGER: Mutex<DesktopManager> = Mutex::new(DesktopManager::new());

#[cfg(feature = "gate1-gui-test")]
static GUI_MOUSE_SEEN: AtomicBool = AtomicBool::new(false);
#[cfg(feature = "gate1-gui-test")]
static GUI_DRAG_SEEN: AtomicBool = AtomicBool::new(false);
#[cfg(feature = "gate1-gui-test")]
static GUI_KEYBOARD_SEEN: AtomicBool = AtomicBool::new(false);
#[cfg(feature = "gate1-gui-test")]
static GUI_TERMINAL_REDRAW_PENDING: AtomicBool = AtomicBool::new(false);

#[inline(always)]
unsafe fn dbg_serial(s: &[u8]) {
    for &b in s {
        // Wait for TX ready
        loop {
            let status: u8;
            core::arch::asm!("in al, dx", out("al") status, in("dx") 0x3FDu16, options(nomem, nostack));
            if status & 0x20 != 0 { break; }
        }
        if b == b'\n' {
            core::arch::asm!("out dx, al", in("al") b'\r', in("dx") 0x3F8u16, options(nomem, nostack));
            loop {
                let status: u8;
                core::arch::asm!("in al, dx", out("al") status, in("dx") 0x3FDu16, options(nomem, nostack));
                if status & 0x20 != 0 { break; }
            }
        }
        core::arch::asm!("out dx, al", in("al") b, in("dx") 0x3F8u16, options(nomem, nostack));
    }
}

pub struct Window {
    pub id: u32,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub title: &'static str,
    pub content_buffer: [u8; 4096],
    pub content_len: usize,
    pub bg_color: u32,
    pub active: bool,
    pub is_minimized: bool,
    pub is_maximized: bool,
    pub prev_bounds: (u32, u32, u32, u32),
}

impl Window {
    pub fn draw(&self, display: &mut X86Display) {
        unsafe { dbg_serial(b"    [win] draw start\n"); }
        // Draw drop shadow
        if self.active {
            let shadow_color = 0; // Black
            for dy in -5i32..(self.height as i32 + 5) {
                for dx in -5i32..(self.width as i32 + 5) {
                    // Only draw outside the window frame
                    if dx < 0 || dx >= self.width as i32 || dy < 0 || dy >= self.height as i32 {
                        let dist_x = if dx < 0 { -dx } else if dx >= self.width as i32 { dx - self.width as i32 + 1 } else { 0 };
                        let dist_y = if dy < 0 { -dy } else if dy >= self.height as i32 { dy - self.height as i32 + 1 } else { 0 };
                        let dist = dist_x + dist_y;
                        
                        if dist <= 5 {
                            let px = (self.x as i32 + dx) as u32;
                            let py = (self.y as i32 + dy) as u32;
                            if px < display.get_width() && py < display.get_height() {
                                let alpha = 120 - (dist as u32 * 20); // 120, 100, 80, 60, 40, 20
                                let bg = display.get_pixel(px, py);
                                let blended = X86Display::alpha_blend(bg, shadow_color, alpha);
                                display.set_pixel(px, py, blended);
                            }
                        }
                    }
                }
            }
        }

        // Draw window frame
        let frame_color = if self.active {
            X86Display::make_pixel(45, 45, 48) // Dark grey
        } else {
            X86Display::make_pixel(30, 30, 30) // Darker grey
        };

        // Draw title bar
        for dy in 0..20 {
            for dx in 0..self.width {
                // Rounded top corners (radius 4)
                if (dx < 4 && dy < 4 && dx + dy < 3) || (dx >= self.width - 4 && dy < 4 && (self.width - dx - 1) + dy < 3) {
                    continue; // Skip corner pixels
                }
                display.set_pixel(self.x + dx, self.y + dy, frame_color);
            }
        }

        // Draw background
        for dy in 20..self.height {
            for dx in 0..self.width {
                // Rounded bottom corners
                if (dx < 4 && dy >= self.height - 4 && dx + (self.height - dy - 1) < 3) ||
                   (dx >= self.width - 4 && dy >= self.height - 4 && (self.width - dx - 1) + (self.height - dy - 1) < 3) {
                    continue; // Skip corner pixels
                }
                display.set_pixel(self.x + dx, self.y + dy, self.bg_color);
            }
        }

        // Draw title text
        let mut cx = self.x + 10;
        let cy = self.y + 2;
        let text_color = X86Display::make_pixel(220, 220, 220);
        for byte in self.title.bytes() {
            display.draw_char(cx, cy, byte, text_color, frame_color);
            cx += 8;
        }

        // Draw buttons (Apple style: Close, Min, Max)
        let close_color = X86Display::make_pixel(255, 95, 86); // Red
        let min_color = X86Display::make_pixel(255, 189, 46); // Yellow
        let max_color = X86Display::make_pixel(39, 201, 63); // Green

        let btn_y = self.y + 5;
        let btn_radius = 5;

        // Draw circles for buttons
        for dy in 0..10 {
            for dx in 0..10 {
                let dist_sq = (dx as i32 - btn_radius) * (dx as i32 - btn_radius) + (dy as i32 - btn_radius) * (dy as i32 - btn_radius);
                if dist_sq <= btn_radius * btn_radius {
                    display.set_pixel(self.x + self.width - 55 + dx, btn_y + dy, min_color);
                    display.set_pixel(self.x + self.width - 35 + dx, btn_y + dy, max_color);
                    display.set_pixel(self.x + self.width - 15 + dx, btn_y + dy, close_color);
                }
            }
        }

        // Draw content text
        let mut cx = self.x + 10;
        let mut cy = self.y + 30;
        for i in 0..self.content_len {
            let byte = self.content_buffer[i];
            if byte == b'\n' {
                cx = self.x + 10;
                cy += 16;
                continue;
            }
            display.draw_char(cx, cy, byte, text_color, self.bg_color);
            cx += 8;
            if cx + 8 > self.x + self.width - 10 {
                cx = self.x + 10;
                cy += 16;
            }
        }
    }
    
    pub fn write_str(&mut self, s: &str) {
        let mut i = 0;
        let bytes = s.as_bytes();
        while i < bytes.len() {
            let byte = bytes[i];
            if byte == 0x08 || byte == 127 { // Backspace or DEL
                if self.content_len > 0 {
                    self.content_len -= 1;
                }
            } else if byte == 0x1B && i + 3 < bytes.len() && bytes[i+1] == b'[' && bytes[i+2] == b'2' && bytes[i+3] == b'J' {
                // ANSI Clear Screen \x1B[2J
                self.content_len = 0;
                i += 3; // skip [2J
                // Optional: also skip \x1B[H if it follows
                if i + 3 < bytes.len() && bytes[i+1] == 0x1B && bytes[i+2] == b'[' && bytes[i+3] == b'H' {
                    i += 3;
                }
            } else {
                if self.content_len >= 4096 {
                    // Scroll up by shifting (simple approach for now)
                    let shift = 256;
                    for j in 0..(4096 - shift) {
                        self.content_buffer[j] = self.content_buffer[j + shift];
                    }
                    self.content_len -= shift;
                }
                self.content_buffer[self.content_len] = byte;
                self.content_len += 1;
            }
            i += 1;
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl Rect {
    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.w && self.x + self.w > other.x &&
        self.y < other.y + other.h && self.y + self.h > other.y
    }
}

pub struct DesktopManager {
    windows: [Option<Window>; MAX_WINDOWS],
    draw_order: [usize; MAX_WINDOWS], // Indices into `windows` array, drawn first to last
    drag_target: Option<usize>,
    drag_last_x: u32,
    drag_last_y: u32,
    pub mouse_x: u32,
    pub mouse_y: u32,
    pub width: u32,
    pub height: u32,
    pub needs_redraw: bool,
    initialized: bool,
    pub dirty_rects: arrayvec::ArrayVec<Rect, 16>,
}

impl DesktopManager {
    pub const fn new() -> Self {
        Self {
            windows: [None, None, None, None],
            draw_order: [0, 1, 2, 3],
            drag_target: None,
            drag_last_x: 0,
            drag_last_y: 0,
            mouse_x: 0,
            mouse_y: 0,
            width: 1024,
            height: 768,
            needs_redraw: true,
            initialized: false,
            dirty_rects: arrayvec::ArrayVec::new_const(),
        }
    }

    pub fn add_dirty_rect(&mut self, rect: Rect) {
        if self.dirty_rects.is_full() {
            self.needs_redraw = true; // Fallback to full redraw
        } else {
            let _ = self.dirty_rects.try_push(rect);
        }
    }

    pub fn init(&mut self, display: &mut X86Display) {
        if self.initialized {
            return;
        }
        let width = display.get_width();
        let height = display.get_height();
        
        self.width = width;
        self.height = height;
        self.mouse_x = width / 2;
        self.mouse_y = height / 2;

        if width > 0 && height > 0 {
            // Calculate pages needed for double buffer
            let size = (width * height * 4) as u64;
            let pages = (size + 4095) / 4096;
            
            let mut alloc = crate::memory::frame::FRAME_ALLOCATOR.lock();
            if let Some(buffer_addr) = alloc.allocate_frames(pages) {
                // We have a double buffer!
                display.enable_double_buffering(buffer_addr as usize);
                
                // Map the buffer if necessary (Assuming identity mapping for 0-32MB region)
                // For this phase, we assume the kernel has access to the physical address.
            }
        }


        let mut term_window = Window {
            id: 1,
            x: 100,
            y: 100,
            width: 400,
            height: 300,
            title: "XPARQ OS Terminal",
            content_buffer: [0; 4096],
            content_len: 0,
            bg_color: X86Display::make_pixel(0, 0, 0),
            active: true,
            is_minimized: false,
            is_maximized: false,
            prev_bounds: (0, 0, 0, 0),
        };
        term_window.write_str("Welcome to XPARQ OS!\n\nThis is a no_std graphical desktop\nenvironment.\n\nType commands in the shell to interact.\n\n> ");
        self.windows[0] = Some(term_window);
        
        let mut sys_window = Window {
            id: 2,
            x: 350,
            y: 150,
            width: 300,
            height: 200,
            title: "System Monitor",
            content_buffer: [0; 4096],
            content_len: 0,
            bg_color: X86Display::make_pixel(32, 32, 32),
            active: false,
            is_minimized: false,
            is_maximized: false,
            prev_bounds: (0, 0, 0, 0),
        };
        sys_window.write_str("CPU: 1%\nRAM: 12MB / 512MB\nUptime: 0:01:23\n\nAll systems nominal.");
        self.windows[1] = Some(sys_window);

        // Initialize draw order so active is on top
        self.draw_order = [2, 3, 1, 0];
        self.initialized = true;
        self.needs_redraw = true;
    }

    fn bring_to_front(&mut self, window_index: usize) {
        // Find the index in draw_order
        let mut order_idx = 0;
        for i in 0..MAX_WINDOWS {
            if self.draw_order[i] == window_index {
                order_idx = i;
                break;
            }
        }

        // Shift elements down
        for i in order_idx..MAX_WINDOWS - 1 {
            self.draw_order[i] = self.draw_order[i + 1];
        }
        
        // Place the target at the top (last rendered)
        self.draw_order[MAX_WINDOWS - 1] = window_index;
    }

    pub fn process_event(&mut self, event: InputEvent) {
        match (event.event_kind, event.data) {
            (crate::hal::input::InputEventKind::MouseMove, InputEventData::Mouse { x: dx, y: dy, .. }) => {
                #[cfg(feature = "gate1-gui-test")]
                if !GUI_MOUSE_SEEN.swap(true, Ordering::AcqRel) {
                    unsafe { dbg_serial(b"XPARQ_TEST:GUI_MOUSE_OK\n"); }
                }
                let old_mouse_rect = Rect { x: self.mouse_x.saturating_sub(10), y: self.mouse_y.saturating_sub(10), w: 20, h: 20 };
                self.add_dirty_rect(old_mouse_rect);
                
                let new_x = self.mouse_x as i32 + dx;
                let new_y = self.mouse_y as i32 + dy;
                
                if new_x >= 0 && new_x < self.width as i32 {
                    self.mouse_x = new_x as u32;
                }
                if new_y >= 0 && new_y < self.height as i32 {
                    self.mouse_y = new_y as u32;
                }

                // Handle Window Dragging
                let new_mouse_rect = Rect { x: self.mouse_x.saturating_sub(10), y: self.mouse_y.saturating_sub(10), w: 20, h: 20 };
                self.add_dirty_rect(new_mouse_rect);

                if let Some(target_idx) = self.drag_target {
                    // Collect old position BEFORE mutably borrowing windows
                    let old_win_rect = if let Some(win) = &self.windows[target_idx] {
                        Some(Rect { x: win.x, y: win.y, w: win.width, h: win.height })
                    } else {
                        None
                    };
                    
                    if let Some(old_rect) = old_win_rect {
                        self.add_dirty_rect(old_rect);
                    }
                    
                    // Now mutably borrow to update position
                    let new_win_rect = if let Some(win) = &mut self.windows[target_idx] {
                        let mx_i = self.mouse_x as i32;
                        let my_i = self.mouse_y as i32;
                        let lx_i = self.drag_last_x as i32;
                        let ly_i = self.drag_last_y as i32;

                        let delta_x = mx_i - lx_i;
                        let delta_y = my_i - ly_i;

                        let mut new_win_x = win.x as i32 + delta_x;
                        let mut new_win_y = win.y as i32 + delta_y;

                        // Basic bounds checking
                        if new_win_x < 0 { new_win_x = 0; }
                        if new_win_y < 0 { new_win_y = 0; }
                        if new_win_x + win.width as i32 > self.width as i32 { new_win_x = (self.width - win.width) as i32; }
                        if new_win_y + win.height as i32 > self.height as i32 { new_win_y = (self.height - win.height) as i32; }

                        win.x = new_win_x as u32;
                        win.y = new_win_y as u32;
                        
                        Some(Rect { x: win.x, y: win.y, w: win.width, h: win.height })
                    } else {
                        None
                    };
                    
                    if let Some(new_rect) = new_win_rect {
                        self.add_dirty_rect(new_rect);
                    }
                    
                    self.drag_last_x = self.mouse_x;
                    self.drag_last_y = self.mouse_y;
                    #[cfg(feature = "gate1-gui-test")]
                    if !GUI_DRAG_SEEN.swap(true, Ordering::AcqRel) {
                        unsafe { dbg_serial(b"XPARQ_TEST:GUI_DRAG_OK\n"); }
                    }
                }
            },
            (crate::hal::input::InputEventKind::MouseDown | crate::hal::input::InputEventKind::MouseUp, InputEventData::Mouse { buttons, .. }) => {
                let pressed = event.event_kind == crate::hal::input::InputEventKind::MouseDown;
                let button = if buttons.contains(crate::hal::input::MouseButtons::LEFT) { 0 } else { 1 };
                if button == 0 {
                    if pressed {
                        // Iterate backwards through draw_order (top to bottom)
                        let mut clicked_idx = None;
                        for i in (0..MAX_WINDOWS).rev() {
                            let idx = self.draw_order[i];
                            if let Some(win) = &mut self.windows[idx] {
                                if win.is_minimized { continue; }
                                if self.mouse_x >= win.x && self.mouse_x <= win.x + win.width &&
                                   self.mouse_y >= win.y && self.mouse_y <= win.y + win.height {
                                    
                                    // Check if clicked on title bar buttons
                                    if self.mouse_y >= win.y + 4 && self.mouse_y <= win.y + 16 {
                                        let mx = self.mouse_x;
                                        if mx >= win.x + win.width - 16 && mx <= win.x + win.width - 4 {
                                            // Close button clicked
                                            self.windows[idx] = None;
                                            break;
                                        } else if mx >= win.x + win.width - 32 && mx <= win.x + win.width - 20 {
                                            // Maximize button clicked
                                            if win.is_maximized {
                                                win.x = win.prev_bounds.0;
                                                win.y = win.prev_bounds.1;
                                                win.width = win.prev_bounds.2;
                                                win.height = win.prev_bounds.3;
                                                win.is_maximized = false;
                                            } else {
                                                win.prev_bounds = (win.x, win.y, win.width, win.height);
                                                win.x = 0;
                                                win.y = 0;
                                                win.width = self.width;
                                                win.height = self.height - 50; // Leave room for dock
                                                win.is_maximized = true;
                                            }
                                            clicked_idx = Some(idx);
                                            break;
                                        } else if mx >= win.x + win.width - 48 && mx <= win.x + win.width - 36 {
                                            // Minimize button clicked
                                            win.is_minimized = true;
                                            win.active = false;
                                            break;
                                        }
                                    }

                                    clicked_idx = Some(idx);
                                    
                                    // Check if clicked on title bar for dragging (top 20px)
                                    if self.mouse_y <= win.y + 20 && !win.is_maximized {
                                        self.drag_target = Some(idx);
                                        self.drag_last_x = self.mouse_x;
                                        self.drag_last_y = self.mouse_y;
                                    }
                                    break;
                                }
                            }
                        }

                        // Dock click handling
                        if clicked_idx.is_none() {
                            let dock_w = 400;
                            let dock_h = 40;
                            let dock_x = (self.width - dock_w) / 2;
                            let dock_y = self.height - dock_h - 10;
                            
                            if self.mouse_x >= dock_x && self.mouse_x <= dock_x + dock_w &&
                               self.mouse_y >= dock_y && self.mouse_y <= dock_y + dock_h {
                                
                                let mut dock_item_x = dock_x + 10;
                                for i in 0..MAX_WINDOWS {
                                    if let Some(win) = &mut self.windows[i] {
                                        if self.mouse_x >= dock_item_x && self.mouse_x <= dock_item_x + 80 {
                                            if win.is_minimized {
                                                win.is_minimized = false;
                                                clicked_idx = Some(i);
                                            } else if win.active {
                                                win.is_minimized = true;
                                                win.active = false;
                                            } else {
                                                clicked_idx = Some(i);
                                            }
                                            break;
                                        }
                                        dock_item_x += 90;
                                    }
                                }
                            }
                        }

                        if let Some(idx) = clicked_idx {
                            self.needs_redraw = true; // For click, just do a full redraw for simplicity (z-order changes)
                            // Deactivate all
                            for w_opt in &mut self.windows {
                                if let Some(w) = w_opt {
                                    w.active = false;
                                }
                            }
                            // Activate clicked
                            if let Some(w) = &mut self.windows[idx] {
                                w.active = true;
                            }
                            self.bring_to_front(idx);
                        }
                    } else {
                        // Mouse up
                        self.drag_target = None;
                    }
                }
            },
            (crate::hal::input::InputEventKind::KeyDown, data) => {
                if let InputEventData::Key { keycode, modifiers, .. } = data {
                    // Check if Terminal window (id 1) is active
                    let mut terminal_active = false;
                    for win_opt in &self.windows {
                        if let Some(w) = win_opt {
                            if w.id == 1 && w.active {
                                terminal_active = true;
                                break;
                            }
                        }
                    }

                    if terminal_active {
                        if let Some(c) = crate::hal::input::utils::keycode_to_char(keycode, modifiers) {
                            let mut buf = [0; 4];
                            let s = c.encode_utf8(&mut buf);
                            use crate::input::InputDevice;
                            for &b in s.as_bytes() {
                                crate::input::KEYBOARD_DEVICE.push_event(b);
                            }
                            #[cfg(feature = "gate1-gui-test")]
                            if !GUI_KEYBOARD_SEEN.swap(true, Ordering::AcqRel) {
                                unsafe { dbg_serial(b"XPARQ_TEST:GUI_KEYBOARD_OK\n"); }
                            }
                            crate::input::KEYBOARD_DEVICE.wait_queue.lock().wake_one();
                        }
                    }
                }
            },
            _ => {}
        }
    }

    pub fn draw(&mut self, display: &mut X86Display) {
        if self.needs_redraw {
            display.clear_clip_rect();
            self.draw_internal(display);
            self.needs_redraw = false;
            self.dirty_rects.clear();
            
            unsafe { dbg_serial(b"  [draw] flush\n"); }
            display.flush();
        } else if !self.dirty_rects.is_empty() {
            // Need to copy out the rects since we borrow self mutably in draw_internal
            let rects_copy = self.dirty_rects.clone();
            self.dirty_rects.clear();
            
            for i in 0..rects_copy.len() {
                let rect = rects_copy[i];
                display.set_clip_rect(rect.x, rect.y, rect.w, rect.h);
                self.draw_internal(display);
            }
            display.clear_clip_rect();
            
            unsafe { dbg_serial(b"  [draw] flush (dirty rects)\n"); }
            display.flush();
        }
    }
    
    fn draw_internal(&mut self, display: &mut X86Display) {
        unsafe { dbg_serial(b"  [draw_int] gradient start\n"); }
        let (clip_x, clip_y, clip_w, clip_h) = display.get_clip_rect().unwrap_or((0, 0, self.width, self.height));
        let max_y = (clip_y + clip_h).min(self.height);
        let max_x = (clip_x + clip_w).min(self.width);
        
        // Draw Desktop Background (Gradient from Deep Purple to Midnight Blue)
        for dy in clip_y..max_y {
            let ratio = (dy * 255) / self.height;
            // From Deep Purple (45, 10, 60) to Deep Ocean Blue (10, 20, 60)
            let r = 45 - ((35 * ratio) / 255) as u8;
            let g = 10 + ((10 * ratio) / 255) as u8;
            let b = 60;
            let row_color = X86Display::make_pixel(r, g, b);
            for dx in clip_x..max_x {
                display.set_pixel(dx, dy, row_color);
            }
        }
        unsafe { dbg_serial(b"  [draw_int] gradient done\n"); }

        // Draw Windows in Z-Order (Back to Front)
        unsafe { dbg_serial(b"  [draw_int] windows start\n"); }
        for &idx in &self.draw_order {
            if let Some(win) = &mut self.windows[idx] {
                if !win.is_minimized {
                    win.draw(display);
                }
            }
        }
        unsafe { dbg_serial(b"  [draw_int] windows done\n"); }

        // Draw Floating Dock Taskbar (Glassmorphism)
        let dock_w = 400;
        let dock_h = 45;
        let dock_x = (self.width - dock_w) / 2;
        let dock_y = self.height - dock_h - 15;
        
        let dock_bg_color = X86Display::make_pixel(30, 30, 30); // Dark grey base
        let dock_border_color = X86Display::make_pixel(100, 100, 120);

        for dy in 0..dock_h {
            for dx in 0..dock_w {
                // Rounded corners for dock (radius 8)
                if (dx < 8 && dy < 8 && dx + dy < 6) ||
                   (dx >= dock_w - 8 && dy < 8 && (dock_w - dx - 1) + dy < 6) ||
                   (dx < 8 && dy >= dock_h - 8 && dx + (dock_h - dy - 1) < 6) ||
                   (dx >= dock_w - 8 && dy >= dock_h - 8 && (dock_w - dx - 1) + (dock_h - dy - 1) < 6) {
                    continue;
                }

                let px = dock_x + dx;
                let py = dock_y + dy;
                let bg_pixel = display.get_pixel(px, py);

                if dx == 0 || dy == 0 || dx == dock_w - 1 || dy == dock_h - 1 {
                    let blended = X86Display::alpha_blend(bg_pixel, dock_border_color, 180);
                    display.set_pixel(px, py, blended);
                } else {
                    let blended = X86Display::alpha_blend(bg_pixel, dock_bg_color, 140);
                    display.set_pixel(px, py, blended);
                }
            }
        }

        // Draw active indicators on dock
        let mut dock_item_x = dock_x + 10;
        let text_color = X86Display::make_pixel(255, 255, 255);
        for win_opt in &self.windows {
            if let Some(win) = win_opt {
                let indicator_color = if win.active {
                    X86Display::make_pixel(0, 120, 215)
                } else {
                    X86Display::make_pixel(80, 80, 80)
                };

                // Draw button (rounded)
                for dy in 5..40 {
                    for dx in 0..80 {
                        // Soft rounding for dock buttons
                        if (dx < 3 && dy < 8) || (dx >= 77 && dy < 8) || (dx < 3 && dy >= 37) || (dx >= 77 && dy >= 37) {
                            if dx + (dy - 5) < 3 || (79 - dx) + (dy - 5) < 3 || dx + (39 - dy) < 3 || (79 - dx) + (39 - dy) < 3 {
                                continue;
                            }
                        }
                        
                        let px = dock_item_x + dx;
                        let py = dock_y + dy;
                        let bg = display.get_pixel(px, py);
                        
                        // Active gets solid, inactive gets transparent
                        let alpha = if win.active { 200 } else { 100 };
                        let blended = X86Display::alpha_blend(bg, indicator_color, alpha);
                        display.set_pixel(px, py, blended);
                    }
                }

                // Draw first 8 chars of title
                let mut cx = dock_item_x + 8;
                let cy = dock_y + 15;
                for (i, byte) in win.title.bytes().enumerate() {
                    if i >= 8 { break; }
                    display.draw_char(cx, cy, byte, text_color, indicator_color);
                    cx += 8;
                }

                // Draw active dot below text
                if win.active {
                    for dy in 0..3 {
                        for dx in 0..3 {
                            display.set_pixel(dock_item_x + 38 + dx, dock_y + 35 + dy, X86Display::make_pixel(255, 255, 255));
                        }
                    }
                }
                
                dock_item_x += 90;
            }
        }

        // Draw Mouse Cursor (Crosshair for now)
        let mouse_color = X86Display::make_pixel(255, 255, 255);
        for i in 0..10 {
            if self.mouse_x + i < self.width {
                display.set_pixel(self.mouse_x + i, self.mouse_y, mouse_color);
            }
            if self.mouse_x >= i {
                display.set_pixel(self.mouse_x - i, self.mouse_y, mouse_color);
            }
            if self.mouse_y + i < self.height {
                display.set_pixel(self.mouse_x, self.mouse_y + i, mouse_color);
            }
            if self.mouse_y >= i {
                display.set_pixel(self.mouse_x, self.mouse_y - i, mouse_color);
            }
        }
    }
    
    pub fn write_to_terminal(&mut self, s: &str) {
        if let Some(win) = &mut self.windows[0] {
            win.write_str(s);
            let rect = Rect { x: win.x, y: win.y, w: win.width, h: win.height };
            self.add_dirty_rect(rect);
            #[cfg(feature = "gate1-gui-test")]
            if GUI_KEYBOARD_SEEN.load(Ordering::Acquire) {
                GUI_TERMINAL_REDRAW_PENDING.store(true, Ordering::Release);
            }
        }
    }
}

fn pop_input_event_irq_safe() -> Option<InputEvent> {
    let flags: u64;
    unsafe {
        core::arch::asm!("pushfq", "pop {}", out(reg) flags, options(nomem));
        core::arch::asm!("cli", options(nomem, nostack));
    }
    let event = crate::input::INPUT_MANAGER.event_queue.lock().pop();
    if flags & (1 << 9) != 0 {
        unsafe { core::arch::asm!("sti", options(nomem, nostack)); }
    }
    event
}

/// Drain pending PS/2 events and render any resulting desktop changes on the
/// current kernel/syscall thread. This is the active GUI execution model while
/// privilege-frame-compatible preemption remains unavailable.
pub fn pump_events_and_redraw() -> usize {
    let mut processed = 0;
    while let Some(event) = pop_input_event_irq_safe() {
        DESKTOP_MANAGER.lock().process_event(event);
        processed += 1;
    }

    let mut display_guard = crate::hal::x86_64::DISPLAY.lock();
    if let Some(display) = display_guard.as_mut() {
        let mut desktop = DESKTOP_MANAGER.lock();
        if desktop.needs_redraw || !desktop.dirty_rects.is_empty() {
            desktop.draw(display);
            #[cfg(feature = "gate1-gui-test")]
            if GUI_TERMINAL_REDRAW_PENDING.swap(false, Ordering::AcqRel) {
                unsafe { dbg_serial(b"XPARQ_TEST:GUI_TERMINAL_REDRAW_OK\n"); }
            }
        }
    }
    processed
}
