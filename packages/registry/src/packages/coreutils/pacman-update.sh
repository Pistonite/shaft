#!/bin/bash
if [ "$(id -u)" -ne 0 ]; then
  echo "error: please run sudo pacman-update"
  exit 1
fi

echo "== refreshing mirror list using reflector..."
reflector -x 'cdnmirror|losangeles|niranjan|cicku' -l 20 -p https --sort rate --save /etc/pacman.d/mirrorlist
echo "== syncing pacman database..."
pacman -Syy
echo "== refreshing keys..."
pacman-key --refresh-keys
echo "== syncing pacman database..."
pacman -Syy
