# -- Bind Ctrl+D to exit
Set-PSReadlineKeyHandler -Key ctrl+d -Function ViExit
# -- Command Prediction
if ($PSVersionTable.PSVersion.Major -ne 5) {
  Set-PSReadLineOption -Color @{
    InlinePrediction = $PSStyle.Italic + $PSStyle.Foreground.Black
  }
} else {
  Remove-Item Alias:curl -Force
}
# -- WSL
function wsl {
  # start in the linux home instead of /mnt/c/Users/<user> when launched from USERPROFILE.
  # '~' is single-quoted so PowerShell does not expand it into a Windows path
  if ($args.Count -eq 0 -and (Get-Location).ProviderPath -eq $env:USERPROFILE) {
    wsl.exe '~'
  } else {
    wsl.exe @args
  }
}
# ===
