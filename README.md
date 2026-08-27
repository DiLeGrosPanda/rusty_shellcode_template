From my personal tooling ;<br>
One of the main issues is getting the strings not to be written to .rdata.<br>
The obf_str macro solves it, a crate-level proc macro may allow all strings to be automatically handled<br>
Msvc toolchain tested in production, x86_64-pc-windows-gnu should work though;<br><br>

Implementing a custom global_allocator allows core::alloc::{String,Vec,format} and more<br>


```powershell
cargo build --release
.\target\release\template.exe
python.exe .\extract_text_section.py .\target\release\template.exe payload.bin
#objcopy.exe -j .text -O binary .\target\release\template.exe payload.bin
.\DynWin32-ShellcodeLocalThread.ps1
```