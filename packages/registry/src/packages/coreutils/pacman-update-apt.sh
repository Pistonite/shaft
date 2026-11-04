#!/bin/bash
# similar functionality of pacman-update but for APT
if [ "$(id -u)" -eq 0 ]; then
  echo "error: do not run pacman-update as root, it runs sudo internally"
  exit 1
fi

sudo apt update
sudo apt full-upgrade
sudo apt autoremove
