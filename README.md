# Lion AutoClicker

[![Build](https://github.com/AmazingCoderA/lion-clicker/actions/workflows/build.yml/badge.svg)](https://github.com/AmazingCoderA/lion-clicker/actions/workflows/build.yml)

Автокликер на Rust с графическим интерфейсом для Windows, Linux/X11 и macOS.

## Возможности

- Левая, правая и средняя кнопки; одиночный, двойной, тройной клик и серия.
- Равномерный или ограниченный нормальный разброс интервалов.
- Случайные паузы, фиксированные координаты и разброс позиции без случайного дрейфа.
- Лимиты по кликам и времени, задержка перед стартом.
- Глобальная клавиша старт/стоп с 1-5 быстрыми нажатиями.
- Безопасная глобальная остановка `Ctrl+Shift+F12`; `Esc` останавливает из окна.
- Атомарное сохранение проверенного TOML-конфига; русский и английский интерфейс.

## Важно

Используйте программу только там, где автоматизация разрешена. В Linux глобальные клавиши и эмуляция ввода работают через X11. В сеансе Wayland они доступны только для XWayland-приложений; для всего рабочего стола выберите сеанс X11.

## Обычная сборка

Требуется Rust 1.88 или новее.

```sh
cargo build --release --locked
cargo test --locked
```

Результат: `target/release/lion-autoclicker` (`.exe` в Windows). Для Windows также доступен скрипт `powershell -ExecutionPolicy Bypass -File scripts/build-windows.ps1`.

## Автосборка

GitHub Actions запускается автоматически при каждом `push` и `pull request`, а также вручную из вкладки **Actions**. Workflow проверяет форматирование, Clippy и тесты, затем публикует артефакты:

- `lion-autoclicker-windows` — Windows `.exe`.
- `lion-autoclicker-linux` — Linux-бинарник.
- `lion-autoclicker.flatpak` — Flatpak-пакет.

Релиз создаётся автоматически после отправки тега вида `v0.1.0`. В него входят архивы Windows x86_64, Linux x86_64/ARM64 и macOS x86_64/ARM64, а также Flatpak и `PKGBUILD` для Arch Linux.

## Buttons / Кнопки

| Русский | English | Назначение / Purpose |
| --- | --- | --- |
| Старт / Стоп | Start / Stop | Запустить или остановить автокликер / Start or stop clicking |
| Кнопка мыши | Mouse button | Левая, правая или средняя / Left, right, or middle |
| Интервал | Interval | Задержка между кликами / Delay between clicks |
| Серия кликов | Click pattern | Одинарный, двойной, тройной или Burst / Single, double, triple, or burst |
| Горячая клавиша | Hotkey | Глобальная клавиша запуска и остановки / Global start-stop shortcut |
| Сохранить | Save | Сохранить настройки / Save settings |
| Применить | Apply | Применить настройки без закрытия окна / Apply without closing |

## Flatpak

Установите Flatpak, `flatpak-builder`, Flathub и SDK:

```sh
flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo
flatpak install flathub org.freedesktop.Platform//25.08 org.freedesktop.Sdk//25.08 org.freedesktop.Sdk.Extension.rust-stable//25.08
chmod +x scripts/build-flatpak.sh
./scripts/build-flatpak.sh
flatpak install --user ./io.github.lionautoclicker.LionAutoclicker.flatpak
```

Файл `cargo-sources.json` фиксирует исходники Cargo для воспроизводимой офлайн-сборки внутри Flatpak.

## Arch Linux / AUR

Для локальной установки из готового исходного пакета:

```sh
curl -LO https://github.com/AmazingCoderA/lion-clicker/releases/latest/download/PKGBUILD
makepkg -si
```

`PKGBUILD` также можно использовать для публикации в AUR и установки командой `yay -S lion-autoclicker` после публикации пакета в AUR.

## Настройки

Конфигурация создаётся после первого сохранения:

- Windows: `%APPDATA%\Lion\LionAutoclicker\config\settings.toml`
- Linux: `~/.config/lionautoclicker/settings.toml`; в Flatpak внутри `~/.var/app/io.github.lionautoclicker.LionAutoclicker/config/`

Приложение проверяет диапазоны и неизвестные поля. Внешние изменения загружаются кнопкой **Перечитать**.

## Управление

- `F6` по умолчанию: старт/стоп.
- `Ctrl+Shift+F12`: безусловная глобальная остановка.
- `Esc`: остановка, когда окно приложения в фокусе.
- Закрытие окна прерывает ожидание и гарантированно отпускает кнопку мыши.

Лицензия: GNU GPL 3.0 only.
