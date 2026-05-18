$env:PATH = "C:\Program Files\qemu;$env:PATH"
qemu-system-x86_64 -drive format=raw,file=build/x86-64/disk.img -boot order=c -nographic -m 128M 2>&1
