echo "refreshing mirror list using reflector..."
sudo reflector -x 'cdnmirror|losangeles|niranjan|cicku' -l 20 -p https --sort rate --save /etc/pacman.d/mirrorlist
sudo pacman -Syy
