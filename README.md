# Lion AutoClicker

[![Windows](https://img.shields.io/badge/Download-Windows.exe-0078D4?style=for-the-badge&logo=windows)](https://github.com/AmazingCoderA/lion-clicker/releases/latest/download/lion-autoclicker-windows-x86_64.exe)
[![Linux](https://img.shields.io/badge/Download-Linux_binary-FCC624?style=for-the-badge&logo=linux&logoColor=black)](https://github.com/AmazingCoderA/lion-clicker/releases/latest/download/lion-autoclicker-linux-x86_64)
[![Flatpak](https://img.shields.io/badge/Download-Flatpak-4A90D9?style=for-the-badge&logo=flatpak)](https://github.com/AmazingCoderA/lion-clicker/releases/latest/download/lion-autoclicker.flatpak)
[![Flathub](https://img.shields.io/badge/Flathub-prepared-orange?style=for-the-badge&logo=flathub)](FLATHUB.md)
[![RU](https://img.shields.io/badge/README-RU-red?style=for-the-badge)](#русский)
[![EN](https://img.shields.io/badge/README-EN-blue?style=for-the-badge)](#english)

[![Build](https://github.com/AmazingCoderA/lion-clicker/actions/workflows/build.yml/badge.svg)](https://github.com/AmazingCoderA/lion-clicker/actions/workflows/build.yml)

## Русский

Lion AutoClicker - графический автокликер на Rust для Windows и Linux/X11. Поддерживает точные интервалы, серии кликов, случайный разброс, лимиты и глобальные горячие клавиши.

### Скачать

- Windows: `lion-autoclicker-windows-x86_64.exe` из [последнего релиза](https://github.com/AmazingCoderA/lion-clicker/releases/latest).
- Linux: `lion-autoclicker-linux-x86_64` или `lion-autoclicker-linux-aarch64` из релиза.
- Flatpak: `lion-autoclicker.flatpak` из релиза.
- Arch Linux: скачайте `PKGBUILD`, затем выполните `makepkg -si`.

### Кнопки

| Кнопка | Что делает |
| --- | --- |
| Start / Stop | Запускает или останавливает автокликер |
| Mouse button | Выбор левой, правой или средней кнопки мыши |
| Interval | Задержка между кликами |
| Click pattern | Одинарный, двойной, тройной клик или серия |
| Hotkey | Глобальная клавиша запуска и остановки |
| Save | Сохраняет настройки в файл |
| Apply | Применяет настройки без перезапуска |

### Управление

- `F6` по умолчанию: старт/стоп.
- `Ctrl+Shift+F12`: аварийная остановка.
- `Esc`: остановка, если окно приложения в фокусе.

### Сборка

```sh
cargo build --release --locked
cargo test --locked
```

Linux работает через X11. В Wayland используйте XWayland или X11-сессию.

## English

Lion AutoClicker is a Rust desktop autoclicker for Windows and Linux/X11. It supports precise intervals, click patterns, random variation, safety limits, and global hotkeys.

### Download

- Windows: `lion-autoclicker-windows-x86_64.exe` from the [latest release](https://github.com/AmazingCoderA/lion-clicker/releases/latest).
- Linux: `lion-autoclicker-linux-x86_64` or `lion-autoclicker-linux-aarch64` from the release.
- Flatpak: `lion-autoclicker.flatpak` from the release.
- Arch Linux: download `PKGBUILD`, then run `makepkg -si`.

### Buttons

| Button | Purpose |
| --- | --- |
| Start / Stop | Starts or stops the autoclicker |
| Mouse button | Selects left, right, or middle mouse button |
| Interval | Delay between clicks |
| Click pattern | Single, double, triple, or burst clicks |
| Hotkey | Global start and stop shortcut |
| Save | Saves settings to disk |
| Apply | Applies settings without restarting |

### Controls

- `F6` by default: start/stop.
- `Ctrl+Shift+F12`: emergency stop.
- `Esc`: stop while the app window is focused.

### Build

```sh
cargo build --release --locked
cargo test --locked
```

Linux input automation uses X11. On Wayland, use XWayland or an X11 session.

License: GNU GPL 3.0 only.
