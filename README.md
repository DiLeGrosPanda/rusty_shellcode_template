From my personal tooling ;<br>
One of the main issue is to keep the strings inside the .text !
Tested with msvc toolchain

```powershell
cargo build --release
.\target\release\template.exe
python.exe .\extract_text_section.py .\target\release\template.exe payload.bin
#objcopy.exe -j .text -O binary .\target\release\template.exe payload.bin
.\DynWin32-ShellcodeLocalThread.ps1
```