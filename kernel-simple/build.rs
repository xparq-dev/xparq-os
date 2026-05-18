// Build script to add PVH ELF note to x86_64 kernel binary

fn main() {
    #[cfg(target_arch = "x86_64")]
    {
        // Tell cargo to rerun if the linker script changes
        println!("cargo:rerun-if-changed=linker-x86_64.ld");
    }
    #[cfg(target_arch = "aarch64")]
    {
        println!("cargo:rerun-if-changed=linker-arm64.ld");
    }
}
