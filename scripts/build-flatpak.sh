#!/usr/bin/env sh
set -eu

APP_ID=io.github.lionautoclicker.LionAutoclicker
flatpak-builder --force-clean --user --install-deps-from=flathub build-dir "$APP_ID.yml"
flatpak build-export repo build-dir
flatpak build-bundle repo "$APP_ID.flatpak" "$APP_ID"
printf 'Created %s.flatpak\n' "$APP_ID"
