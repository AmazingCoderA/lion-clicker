# Flathub Submission

This project is prepared for Flathub, but the app is not published there yet.

## What Is Ready

- Flatpak app id: `io.github.lionautoclicker.LionAutoclicker`.
- Local/CI manifest: `io.github.lionautoclicker.LionAutoclicker.yml`.
- Flathub PR manifest template: `flathub/io.github.lionautoclicker.LionAutoclicker.yml`.
- AppStream metadata: `packaging/io.github.lionautoclicker.LionAutoclicker.metainfo.xml`.
- Desktop file and icon are under `packaging/`.
- Cargo dependencies are pinned in `cargo-sources.json` for offline Flatpak builds.

## Sandbox Permissions

Requested permissions are intentionally small:

- `--socket=x11`: required for global hotkeys and mouse input automation.
- `--share=ipc`: required with X11 for correct desktop integration.
- `--device=dri`: required for OpenGL rendering through `eframe`/`glow`.

No filesystem, network, session bus, or system bus permissions are requested at runtime.

Wayland is not requested because desktop-wide input automation and global hotkeys are not available there in the same way. Users should run an X11 session or XWayland-compatible environment.

## PR Steps

1. Fork `https://github.com/flathub/flathub`.
2. Create a new branch named `io.github.lionautoclicker.LionAutoclicker`.
3. Copy `flathub/io.github.lionautoclicker.LionAutoclicker.yml` to the Flathub PR root.
4. Copy `cargo-sources.json` to the Flathub PR root.
5. Replace `REPLACE_WITH_RELEASE_COMMIT` with the source commit to build.
6. Test locally with `flatpak-builder --force-clean build-dir io.github.lionautoclicker.LionAutoclicker.yml`.
7. Open a PR to `flathub/flathub`.

## Likely Review Questions

- Why X11 permission is needed: global hotkeys and mouse automation require X11/XWayland.
- Why Wayland is not supported: Wayland blocks desktop-wide synthetic input by design.
- Whether the app is safe: it is user-controlled, has visible controls, limits, and an emergency stop hotkey.
- Screenshots: Flathub may ask for a real app screenshot in AppStream metadata before final acceptance.
