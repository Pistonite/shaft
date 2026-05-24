# environment variables, set for testing the build
# $CLINK="C:\path\to\clink_x64.exe"; . ./build-env.ps1
$env:CLINK_CMD_COMPILE_ARCH="x64"
$env:CLINK_CMD_COMPILE_CMD_EXECUTABLE="$env:COMSPEC"
$env:CLINK_CMD_COMPILE_CLINK_EXECUTABLE=$CLINK
$env:CLINK_CMD_COMPILE_INIT_CMD=(Resolve-Path test_init.cmd).Path
$env:CLINK_CMD_COMPILE_PRINT_INSTEAD="0"
