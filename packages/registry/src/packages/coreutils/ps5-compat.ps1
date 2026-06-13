# coreutils shell wrapper isn't supported for PS5
# so we fallback to removing the conflicting aliases only
# which means some forms of commands (especially glob) won't work
Remove-Item Alias:cat -Force
Remove-Item Alias:cp -Force
Remove-Item Alias:dir -Force
Remove-Item Alias:echo -Force
Remove-Item Alias:ls -Force
Remove-Item Alias:mv -Force
Remove-Item Alias:pwd -Force
Remove-Item Alias:rm -Force
Remove-Item Alias:rmdir -Force
Remove-Item Alias:sort -Force
Remove-Item Alias:sleep -Force
Remove-Item Alias:tee -Force
Remove-Item Function:mkdir -Force

