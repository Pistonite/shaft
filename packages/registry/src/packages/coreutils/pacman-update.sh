#!/bin/bash
if [ "$(id -u)" -eq 0 ]; then
  echo "error: do not run pacman-update as root, it runs sudo internally"
  exit 1
fi

echo "== refreshing mirror list using reflector..."
sudo reflector -x 'cdnmirror|losangeles|niranjan|cicku|osbeck|fastly|frankfurt|theash|kescher' -l 20 -p https --sort score --save /etc/pacman.d/mirrorlist
echo "== syncing pacman database..."
sudo pacman -Syy
echo "== refreshing keys..."
sudo pacman-key --refresh-keys
echo "== syncing pacman database..."
sudo pacman -Syy

