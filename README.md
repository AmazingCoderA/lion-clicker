# Lion AutoClicker

[![Windows](https://img.shields.io/badge/Download-Windows.exe-0078D4?style=for-the-badge&logo=windows)](https://github.com/AmazingCoderA/lion-clicker/releases/latest/download/lion-autoclicker-windows-x86_64.exe)
[![Linux](https://img.shields.io/badge/Download-Linux_binary-FCC624?style=for-the-badge&logo=linux&logoColor=black)](https://github.com/AmazingCoderA/lion-clicker/releases/latest/download/lion-autoclicker-linux-x86_64)
[![Flatpak](https://img.shields.io/badge/Download-Flatpak-4A90D9?style=for-the-badge&logo=flatpak)](https://github.com/AmazingCoderA/lion-clicker/releases/latest/download/lion-autoclicker.flatpak)
[![Flathub](https://img.shields.io/badge/Flathub-prepared-orange?style=for-the-badge&logo=flathub)](FLATHUB.md)
[![RU](https://img.shields.io/badge/README-RU-red?style=for-the-badge)](#русский)
[![EN](https://img.shields.io/badge/README-EN-blue?style=for-the-badge)](#english)

[![Build](https://github.com/AmazingCoderA/lion-clicker/actions/workflows/build.yml/badge.svg)](https://github.com/AmazingCoderA/lion-clicker/actions/workflows/build.yml)

## Русский

Lion AutoClicker - графический автокликер на Rust для Windows, Linux/X11 и экспериментальных BSD/macOS сборок. Поддерживает точные интервалы, серии кликов, случайный разброс, лимиты, темы, кастомизацию и глобальные горячие клавиши.

Возможности: одиночный/двойной/тройной клик, серия, разброс интервалов, разброс удержания, разброс внутри серии, длинные случайные паузы, регулярные паузы через N кликов, плавный разгон, фиксированная точка с радиусом, возврат курсора, микро-движение вокруг текущего курсора, fail-safe остановка при ручном движении курсора, лимит кликов и времени.

### Скачать

- Windows: `lion-autoclicker-windows-x86_64.exe` из [последнего релиза](https://github.com/AmazingCoderA/lion-clicker/releases/latest).
- Linux: `lion-autoclicker-linux-x86_64`, `lion-autoclicker-linux-aarch64` или, если собралось, `lion-autoclicker-linux-armv7`.
- Windows ARM64, macOS ARM64/x86_64, FreeBSD x86_64 и OpenBSD x86_64 публикуются как experimental, если CI смог их собрать.
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
| Pick key | Выбор хоткея через следующее нажатие клавиши |
| Theme / Accent | Темы, RGB-акцент, масштаб и компактный режим |
| Save | Сохраняет настройки в файл |
| Apply | Применяет настройки без перезапуска |

### Управление

- `F6` по умолчанию: старт/стоп.
- Хоткей можно выбрать из списка, захватить кнопкой `Выбрать клавишу` или ввести вручную, например `AudioVolumeUp`.
- `Ctrl+Shift+F12`: аварийная остановка.
- `Esc`: остановка, если окно приложения в фокусе.

### Сборка

```sh
cargo build --release --locked
cargo test --locked
```

Linux работает через X11. В Wayland используйте XWayland или X11-сессию.

## English

Lion AutoClicker is a Rust desktop autoclicker for Windows, Linux/X11, and experimental BSD/macOS builds. It supports precise intervals, click patterns, random variation, safety limits, themes, customization, and global hotkeys.

Features: single/double/triple clicks, burst mode, interval variation, hold variation, within-burst variation, random long breaks, scheduled pauses every N clicks, ramp-up, fixed point radius, cursor restore, micro-move around the current cursor, fail-safe stop on manual cursor movement, click limit, and time limit.

### Download

- Windows: `lion-autoclicker-windows-x86_64.exe` from the [latest release](https://github.com/AmazingCoderA/lion-clicker/releases/latest).
- Linux: `lion-autoclicker-linux-x86_64`, `lion-autoclicker-linux-aarch64`, or `lion-autoclicker-linux-armv7` when available.
- Windows ARM64, macOS ARM64/x86_64, FreeBSD x86_64, and OpenBSD x86_64 are published as experimental builds when CI can build them.
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
| Pick key | Sets the hotkey from the next key press |
| Theme / Accent | Themes, RGB accent, UI scale, and compact mode |
| Save | Saves settings to disk |
| Apply | Applies settings without restarting |

### Controls

- `F6` by default: start/stop.
- The hotkey can be selected from the list, captured with `Pick key`, or typed manually, for example `AudioVolumeUp`.
- `Ctrl+Shift+F12`: emergency stop.
- `Esc`: stop while the app window is focused.

### Build

```sh
cargo build --release --locked
cargo test --locked
```

Linux input automation uses X11. On Wayland, use XWayland or an X11 session.

License: GNU GPL 3.0 only.
