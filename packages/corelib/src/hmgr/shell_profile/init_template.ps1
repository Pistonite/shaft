# init/init.pwsh
# this file is managed by SHAFT, do not edit manually
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
# ===


