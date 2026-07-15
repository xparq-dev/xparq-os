// XPARQ OS - Phase 20.5: Secure User Memory Validation
// Boundary checks and safe copying between Ring 3 and Ring 0

use core::slice;

/// Check if the pointer and length safely reside within User Space (lower half)
pub fn validate_user_ptr(ptr: u64, size: u64) -> bool {
    let end = ptr.saturating_add(size);
    // User space limit in a standard 4-level paging x86_64 setup
    end < 0x0000_7FFF_FFFF_FFFF
}

/// Safely copy data from user-space memory to a kernel buffer
pub fn copy_from_user(dst: &mut [u8], user_ptr: u64, len: usize) -> Result<(), i64> {
    if !validate_user_ptr(user_ptr, len as u64) {
        return Err(-22); // -EINVAL (or -EFAULT)
    }
    
    if dst.len() < len {
        return Err(-22);
    }

    // In a full implementation, this should catch Page Faults gracefully.
    // For now, we perform a validated slice copy.
    let user_slice = unsafe { slice::from_raw_parts(user_ptr as *const u8, len) };
    dst[..len].copy_from_slice(user_slice);
    
    Ok(())
}

/// Safely copy data from a kernel buffer to user-space memory
pub fn copy_to_user(user_ptr: u64, src: &[u8], len: usize) -> Result<(), i64> {
    if !validate_user_ptr(user_ptr, len as u64) {
        return Err(-22); // -EINVAL (or -EFAULT)
    }
    
    if src.len() < len {
        return Err(-22);
    }

    // In a full implementation, this should catch Page Faults gracefully.
    let user_slice = unsafe { slice::from_raw_parts_mut(user_ptr as *mut u8, len) };
    user_slice.copy_from_slice(&src[..len]);
    
    Ok(())
}
