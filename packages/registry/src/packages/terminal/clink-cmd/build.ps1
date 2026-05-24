$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
Push-Location $ScriptDir
try {
    $arch = "$env:CLINK_CMD_COMPILE_ARCH"
    if ($arch -eq "") {
        $arch = switch ($env:PROCESSOR_ARCHITECTURE) {
            "ARM64" { "arm64" }
            "AMD64" { "x64" }
            "x86"   { if ([System.Environment]::Is64BitOperatingSystem) { "x64" } else { "x86" } }
            default { throw "Unsupported architecture: $env:PROCESSOR_ARCHITECTURE" }
        }
    }
    Write-Host "Building for architecture: $arch"

    # all needs to be the part inside the string literal (so \ becomes \\)

    $CMD_EXECUTABLE="$env:CLINK_CMD_COMPILE_CMD_EXECUTABLE".Trim().replace("/", "\").replace("\", "\\")
    if ($CMD_EXECUTABLE -eq "") {
        throw "CLINK_CMD_COMPILE_CMD_EXECUTABLE not defined"
    }
    Write-Host "CMD_EXECUTABLE = $CMD_EXECUTABLE" 

    $clink_exe = "$env:CLINK_CMD_COMPILE_CLINK_EXECUTABLE".Trim()
    if ($clink_exe -eq "") {
        throw "CLINK_CMD_COMPILE_CLINK_EXECUTABLE not defined"
    }
    $init_cmd = "$env:CLINK_CMD_COMPILE_INIT_CMD".Trim()
    if ($init_cmd -eq "") {
        throw "CLINK_CMD_COMPILE_CLINK_EXECUTABLE not defined"
    }
    $clink_exe = $clink_exe.replace("/", "\").replace("\", "\\")
    $init_cmd = $init_cmd.replace("/", "\").replace("\", "\\")
    $CLINK_INJECT="\`"" + $clink_exe + "\`" inject --quiet && \`"" + $init_cmd + "\`""
    Write-Host "CLINK_INJECT = $CLINK_INJECT" 
    # quotes, slashes and spaces need to be escaped again for cmake parsing
    $CMD_EXECUTABLE = $CMD_EXECUTABLE.replace("\", "\\").replace("`"", "\`"").replace(" ", "\ ")
    $CLINK_INJECT = $CLINK_INJECT.replace("\", "\\").replace("`"", "\`"").replace(" ", "\ ")

    $PRINT_INSTEAD_OF_EXEC="$env:CLINK_CMD_COMPILE_PRINT_INSTEAD".Trim()
    if ($PRINT_INSTEAD_OF_EXEC -eq "") {
        $PRINT_INSTEAD_OF_EXEC = "0"
    }

    $CMAKELISTS_CONTENT="cmake_minimum_required(VERSION 3.20)`r`n" +`
    "project(clink-cmd C)`r`n" +`
    "add_executable(clink-cmd main.c)`r`n" +`
    "target_compile_definitions(clink-cmd PRIVATE " +`
    " CMD_EXECUTABLE=L\`"$CMD_EXECUTABLE\`";" +`
    " CLINK_INJECT=L\`"$CLINK_INJECT\`";" +`
    " PRINT_INSTEAD_OF_EXEC=$PRINT_INSTEAD_OF_EXEC" +`
    ")"
    $CMAKELISTS_CONTENT | Set-Content -Path CMakeLists.txt 
    Write-Host "Written CMakeLists.txt"

    # Remove build directory
    Remove-Item -Recurse -Force ./build -ErrorAction SilentlyContinue
    Remove-Item -Force clink-cmd.exe -ErrorAction SilentlyContinue

    # detect the Visual Studio version (2022 or 2026)
    $vsVersions = @(
        @{ Year = "2026"; Generator = "Visual Studio 18 2026" }
        @{ Year = "2022"; Generator = "Visual Studio 17 2022" }
    )
    $vsBasePaths = @(
        "${env:ProgramFiles}\Microsoft Visual Studio"
        "${env:ProgramFiles(x86)}\Microsoft Visual Studio"
    )
    $vsEditions = @("Community", "Professional", "Enterprise")
    $vsBuildToolsEditions = @("18", "17")
    $generator = $null
    foreach ($vs in $vsVersions) {
        foreach ($vsBasePath in $vsBasePaths) {
            foreach ($edition in $vsEditions) {
                $vsPath = Join-Path $vsBasePath "$($vs.Year)\$edition"
                if (Test-Path $vsPath) {
                    $generator = $vs.Generator
                    Write-Host "Found Visual Studio $($vs.Year) $edition at $vsPath"
                    break
                }
            }
            foreach ($edition in $vsBuildToolsEditions) {
                $vsPath = Join-Path $vsBasePath "$edition\BuildTools"
                if (Test-Path $vsPath) {
                    $generator = $vs.Generator
                    Write-Host "Found Visual Studio $($vs.Year) $edition at $vsPath"
                    break
                }
            }
            if ($generator) { break }
        }
        if ($generator) { break }
    }
    if (-not $generator) {
        $generator = $vsVersions[0].Generator
        Write-Host "Could not determine VS version, assuming $generator"
    }

    cmake -S . -B build -G $generator -A $arch
    cmake --build build --config Release
    $output = "build\Release\clink-cmd.exe"
    if (-not (Test-Path $output)) {
        throw "Build output not found, likely failed"
    }
    Copy-Item -Path $output -Destination clink-cmd.exe -Force
    Write-Host "Build output: clink-cmd.exe"

} finally {
    Pop-Location
}
