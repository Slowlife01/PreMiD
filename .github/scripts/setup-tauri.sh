#!/usr/bin/env bash
set -euo pipefail
set -x

has() {
    if [ "$#" -ne 1 ]; then
        err "Usage: has <command>"
    fi

    command -v "$1" >/dev/null 2>&1
}

err() {
    for _line in "$@"; do
        echo "$@" >&2
    done
    exit 1
}

if has apt-get; then
    # Tauri dependencies
    DEBIAN_TAURI_DEPS="libwebkit2gtk-4.0-dev build-essential curl wget libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev patchelf"

    sudo apt-get -y update
    sudo apt-get -y install $DEBIAN_TAURI_DEPS
elif has pacman; then
    # Tauri deps https://tauri.studio/guides/getting-started/setup/linux#1-system-dependencies
    ARCH_TAURI_DEPS="webkit2gtk base-devel curl wget openssl appmenu-gtk-module gtk3 libappindicator-gtk3 librsvg libvips patchelf"
    sudo pacman -Sy --needed $ARCH_TAURI_DEPS
elif has dnf; then
    # `webkit2gtk4.0-devel` also provides `webkit2gtk3-devel`, it's just under a different package in fedora versions >= 37.
    # https://koji.fedoraproject.org/koji/packageinfo?tagOrder=-blocked&packageID=26162#taglist
    # https://packages.fedoraproject.org/pkgs/webkitgtk/webkit2gtk4.0-devel/fedora-38.html#provides
    FEDORA_37_TAURI_WEBKIT="webkit2gtk4.0-devel"
    FEDORA_36_TAURI_WEBKIT="webkit2gtk3-devel"

    # Tauri dependencies
    # openssl is manually declared here as i don't think openssl and openssl-devel are actually dependant on eachother
    # openssl also has a habit of being missing from some of my fresh Fedora installs - i've had to install it at least twice
    FEDORA_TAURI_DEPS="openssl openssl-devel curl wget libappindicator-gtk3 librsvg2-devel patchelf"

    if ! sudo dnf install $FEDORA_37_TAURI_WEBKIT && ! sudo dnf install $FEDORA_36_TAURI_WEBKIT; then
        err 'We were unable to install the webkit2gtk4.0-devel/webkit2gtk3-devel package.'
    fi

    sudo dnf group install "C Development Tools and Libraries"
    sudo dnf install $FEDORA_TAURI_DEPS
fi
