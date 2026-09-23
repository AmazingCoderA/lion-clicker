use crate::{
    config::{self, Button, Config, Distribution, Language, Pattern, Theme},
    engine::{Engine, Phase, Status},
    hotkeys::{Hotkeys, TapTracker, STOP_HOTKEY},
};
use anyhow::Result;
use eframe::egui::{self, Color32, RichText};
use global_hotkey::{GlobalHotKeyEvent, HotKeyState};
use std::{
    sync::mpsc,
    time::{Duration, Instant},
};

const COMMON_HOTKEYS: &[&str] = &[
    "F1",
    "F2",
    "F3",
    "F4",
    "F5",
    "F6",
    "F7",
    "F8",
    "F9",
    "F10",
    "F11",
    "F12",
    "F13",
    "F14",
    "F15",
    "F16",
    "F17",
    "F18",
    "F19",
    "F20",
    "F21",
    "F22",
    "F23",
    "F24",
    "A",
    "B",
    "C",
    "D",
    "E",
    "G",
    "H",
    "J",
    "K",
    "L",
    "M",
    "N",
    "O",
    "P",
    "Q",
    "R",
    "S",
    "T",
    "U",
    "V",
    "W",
    "X",
    "Y",
    "Z",
    "0",
    "1",
    "2",
    "3",
    "4",
    "5",
    "6",
    "7",
    "8",
    "9",
    "Space",
    "Tab",
    "Enter",
    "Backspace",
    "Insert",
    "Delete",
    "Home",
    "End",
    "PageUp",
    "PageDown",
    "ArrowUp",
    "ArrowDown",
    "ArrowLeft",
    "ArrowRight",
    "Ctrl+Space",
    "Ctrl+F6",
    "Shift+F6",
    "Alt+F6",
    "Ctrl+Shift+F6",
    "Ctrl+Alt+F6",
];

pub struct App {
    draft: Config,
    applied: Config,
    saved: Config,
    engine: Engine,
    hotkeys: Option<Hotkeys>,
    hotkey_error: Option<String>,
    capturing_hotkey: bool,
    taps: TapTracker,
    events: mpsc::Receiver<GlobalHotKeyEvent>,
    message: Option<(bool, String)>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Result<Self> {
        let (config, message) = match config::load() {
            Ok(config) => (config, None),
            Err(error) => (
                Config::default(),
                Some((
                    true,
                    format!("Не удалось загрузить настройки / Could not load settings: {error:#}"),
                )),
            ),
        };
        apply_ui_style(&cc.egui_ctx, &config);
        let (hotkeys, hotkey_error) = match Hotkeys::new(&config.hotkey) {
            Ok(keys) => (Some(keys), None),
            Err(error) => (None, Some(format!("{error:#}"))),
        };
        // Wake the UI for global keys even when its window is unfocused/minimized.
        let (tx, events) = mpsc::channel();
        let context = cc.egui_ctx.clone();
        GlobalHotKeyEvent::set_event_handler(Some(move |event| {
            let _ = tx.send(event);
            context.request_repaint();
        }));
        Ok(Self {
            draft: config.clone(),
            applied: config.clone(),
            saved: config,
            engine: Engine::new()?,
            hotkeys,
            hotkey_error,
            capturing_hotkey: false,
            taps: TapTracker::default(),
            events,
            message,
        })
    }

    fn apply(&mut self) -> Result<()> {
        self.draft.validate()?;
        let result = match &mut self.hotkeys {
            Some(keys) => keys.rebind(&self.draft.hotkey),
            None => Hotkeys::new(&self.draft.hotkey).map(|keys| self.hotkeys = Some(keys)),
        };
        if let Err(error) = result {
            let error = format!("{error:#}");
            self.hotkey_error = Some(error.clone());
            anyhow::bail!(error);
        }
        self.hotkey_error = None;
        if self.applied.hotkey != self.draft.hotkey
            || self.applied.trigger_count != self.draft.trigger_count
            || self.applied.trigger_timeout_ms != self.draft.trigger_timeout_ms
        {
            self.taps = TapTracker::default();
        }
        self.applied = self.draft.clone();
        Ok(())
    }

    fn toggle(&mut self) {
        if self.engine.status().running() {
            self.engine.stop();
            return;
        }
        let result = self
            .apply()
            .and_then(|()| self.engine.start(self.draft.clone()));
        match result {
            Ok(()) => {
                self.message = None;
            }
            Err(error) => self.message = Some((true, format!("{error:#}"))),
        }
    }

    fn process_events(&mut self) {
        while let Ok(event) = self.events.try_recv() {
            let Some(keys) = &self.hotkeys else { continue };
            if event.id == keys.stop.id() && event.state == HotKeyState::Pressed {
                self.engine.stop();
                self.taps = TapTracker::default();
            } else if event.id == keys.trigger.id()
                && self.taps.event(
                    event.state == HotKeyState::Pressed,
                    Instant::now(),
                    self.applied.trigger_count,
                    Duration::from_millis(self.applied.trigger_timeout_ms),
                )
            {
                self.toggle();
            }
        }
    }

    fn save(&mut self) {
        match self.apply().and_then(|()| config::save(&self.draft)) {
            Ok(()) => {
                self.saved = self.draft.clone();
                self.message = Some((
                    false,
                    self.draft
                        .language
                        .text("Настройки сохранены", "Settings saved")
                        .into(),
                ));
            }
            Err(error) => self.message = Some((true, format!("{error:#}"))),
        }
    }

    fn status_panel(&self, ui: &mut egui::Ui, status: &Status, accent: Color32) {
        let l = self.draft.language;
        let (label, color) = match status.phase {
            Phase::Idle => (l.text("ОЖИДАНИЕ", "IDLE"), Color32::GRAY),
            Phase::Countdown => (l.text("ПОДГОТОВКА", "COUNTDOWN"), accent),
            Phase::Clicking => (l.text("КЛИКИ", "CLICKING"), Color32::LIGHT_GREEN),
            Phase::Break => (l.text("ПАУЗА", "BREAK"), accent),
        };
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("● {label}")).color(color).strong());
            ui.separator();
            ui.label(format!("{}: {}", l.text("Клики", "Clicks"), status.clicks));
            ui.separator();
            ui.label(format!(
                "{:.1} {}",
                status.elapsed.as_secs_f64(),
                l.text("с", "s")
            ));
            let cps = status.clicks as f64 / status.elapsed.as_secs_f64().max(0.001);
            ui.label(format!("{cps:.1} CPS"));
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        apply_ui_style(ctx, &self.draft);
        self.process_events();
        if self.capturing_hotkey {
            if let Some(binding) = capture_hotkey(ctx) {
                self.draft.hotkey = binding;
                self.capturing_hotkey = false;
            }
        }
        let status = self.engine.status();
        if ctx.input(|input| input.key_pressed(egui::Key::Escape))
            && !self.applied.hotkey.to_ascii_lowercase().contains("escape")
        {
            self.engine.stop();
            self.taps = TapTracker::default();
        }
        let running = status.running();
        let l = self.draft.language;
        let accent = accent_color(&self.draft);

        egui::TopBottomPanel::top("heading").show(ctx, |ui| {
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.heading(RichText::new("LION").color(accent).size(30.0));
                ui.heading("AutoClicker");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.selectable_value(&mut self.draft.language, Language::En, "EN");
                    ui.selectable_value(&mut self.draft.language, Language::Ru, "RU");
                });
            });
            ui.label(l.text(
                "Точное управление. Гибкие последовательности.",
                "Precise control. Flexible sequences.",
            ));
            ui.add_space(8.0);
            self.status_panel(ui, &status, accent);
            ui.add_space(8.0);
        });

        egui::TopBottomPanel::bottom("controls").show(ctx, |ui| {
            ui.add_space(7.0);
            ui.horizontal(|ui| {
                let text = if running {
                    l.text("■  Остановить", "■  Stop")
                } else {
                    l.text("▶  Запустить", "▶  Start")
                };
                if ui
                    .add_sized(
                        [175.0, 40.0],
                        egui::Button::new(RichText::new(text).strong()).fill(if running {
                            Color32::from_rgb(140, 48, 45)
                        } else {
                            darken(accent, 0.45)
                        }),
                    )
                    .clicked()
                {
                    self.toggle();
                }
                if ui
                    .add_enabled(
                        !running,
                        egui::Button::new(l.text("Сохранить / применить", "Save / apply")),
                    )
                    .clicked()
                {
                    self.save();
                }
                if self.draft != self.saved {
                    ui.label(RichText::new(l.text("Не сохранено", "Unsaved")).color(accent));
                }
            });
            let trigger = self
                .hotkeys
                .as_ref()
                .map(|_| self.applied.hotkey.as_str())
                .unwrap_or("—");
            ui.label(format!(
                "{}: {} ×{}  |  {}: {}  |  Esc: {}",
                l.text("Старт/стоп", "Toggle"),
                trigger,
                self.applied.trigger_count,
                l.text("Стоп", "Stop"),
                STOP_HOTKEY,
                l.text("стоп в окне", "stop in window")
            ));
            ui.add_space(4.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                if let Some(error) = &self.hotkey_error {
                    ui.colored_label(accent, l.text("Не удалось применить хоткей. Используйте кнопки окна; активная привязка показана внизу.", "Could not apply hotkey. Use window controls; the active binding is shown below."));
                    ui.label(error);
                    ui.separator();
                }
                if let Some(error) = &status.error {
                    ui.colored_label(Color32::LIGHT_RED, error);
                }
                if let Some((error, text)) = &self.message {
                    ui.colored_label(if *error { Color32::LIGHT_RED } else { Color32::LIGHT_GREEN }, text);
                }
                #[cfg(target_os = "linux")]
                if std::env::var("XDG_SESSION_TYPE").is_ok_and(|value| value == "wayland") {
                    ui.colored_label(accent, l.text("Сеанс Wayland: клики и хоткеи ограничены XWayland. Для всего рабочего стола войдите в сеанс X11.", "Wayland session: input and hotkeys are limited to XWayland. Use an X11 session for full desktop support."));
                }
                let mut start_hotkey_capture = false;
                let mut cancel_hotkey_capture = false;
                let capturing_hotkey = self.capturing_hotkey;
                ui.add_enabled_ui(!running, |ui| {
                    let c = &mut self.draft;
                    egui::CollapsingHeader::new(l.text("Внешний вид", "Appearance")).default_open(false).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(l.text("Тема", "Theme"));
                            ui.selectable_value(&mut c.theme, Theme::LionDark, l.text("Лев", "Lion"));
                            ui.selectable_value(&mut c.theme, Theme::Midnight, l.text("Полночь", "Midnight"));
                            ui.selectable_value(&mut c.theme, Theme::Forest, l.text("Лес", "Forest"));
                            ui.selectable_value(&mut c.theme, Theme::Light, l.text("Светлая", "Light"));
                            ui.selectable_value(&mut c.theme, Theme::HighContrast, l.text("Контраст", "Contrast"));
                        });
                        egui::Grid::new("appearance").num_columns(2).spacing([25.0, 9.0]).show(ui, |ui| {
                            number(ui, l.text("Акцент R", "Accent R"), &mut c.accent_r, 0..=255);
                            number(ui, l.text("Акцент G", "Accent G"), &mut c.accent_g, 0..=255);
                            number(ui, l.text("Акцент B", "Accent B"), &mut c.accent_b, 0..=255);
                            number(ui, l.text("Масштаб интерфейса, %", "UI scale, %"), &mut c.ui_scale_percent, 75..=200);
                        });
                        ui.checkbox(&mut c.compact_mode, l.text("Компактный режим", "Compact mode"));
                    });
                    ui.separator();
                    egui::CollapsingHeader::new(l.text("Клики и интервалы", "Clicks and timing")).default_open(true).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(l.text("Кнопка мыши", "Mouse button"));
                            ui.selectable_value(&mut c.button, Button::Left, l.text("Левая", "Left"));
                            ui.selectable_value(&mut c.button, Button::Right, l.text("Правая", "Right"));
                            ui.selectable_value(&mut c.button, Button::Middle, l.text("Средняя", "Middle"));
                        });
                        ui.horizontal(|ui| {
                            ui.label(l.text("Последовательность", "Pattern"));
                            ui.selectable_value(&mut c.pattern, Pattern::Single, l.text("Один", "Single"));
                            ui.selectable_value(&mut c.pattern, Pattern::Double, l.text("Два", "Double"));
                            ui.selectable_value(&mut c.pattern, Pattern::Triple, l.text("Три", "Triple"));
                            ui.selectable_value(&mut c.pattern, Pattern::Burst, l.text("Серия", "Burst"));
                        });
                        egui::Grid::new("timing").num_columns(2).spacing([25.0, 9.0]).show(ui, |ui| {
                            number(ui, l.text("Пауза после последовательности, мс", "Pause after sequence, ms"), &mut c.delay_ms, 1..=3_600_000);
                            number(ui, l.text("Удержание кнопки, мс", "Button hold, ms"), &mut c.hold_ms, 1..=60_000);
                            number(ui, l.text("Плавный разгон, мс", "Ramp-up time, ms"), &mut c.ramp_up_ms, 0..=3_600_000);
                            if c.pattern == Pattern::Burst {
                                number(ui, l.text("Кликов в серии", "Clicks per burst"), &mut c.burst_count, 1..=1000);
                            }
                            if c.pattern != Pattern::Single {
                                number(ui, l.text("Пауза внутри последовательности, мс", "Pause within sequence, ms"), &mut c.burst_interval_ms, 1..=60_000);
                            }
                        });
                    });
                    ui.separator();
                    egui::CollapsingHeader::new(l.text("Вариация интервалов и паузы", "Timing variation and breaks")).default_open(true).show(ui, |ui| {
                        ui.checkbox(&mut c.randomize, l.text("Случайные вариации", "Random variation"));
                        ui.add_enabled_ui(c.randomize, |ui| {
                            ui.horizontal(|ui| {
                                ui.selectable_value(&mut c.distribution, Distribution::Uniform, l.text("Равномерное", "Uniform"));
                                ui.selectable_value(&mut c.distribution, Distribution::Normal, l.text("Нормальное (ограниченное)", "Normal (bounded)"));
                            });
                            egui::Grid::new("jitter").num_columns(2).show(ui, |ui| {
                                number(ui, l.text("Разброс паузы ± мс", "Pause variation ± ms"), &mut c.delay_jitter_ms, 0..=60_000);
                                number(ui, l.text("Разброс удержания ± мс", "Hold variation ± ms"), &mut c.hold_jitter_ms, 0..=60_000);
                                number(ui, l.text("Разброс внутри серии ± мс", "Within-sequence variation ± ms"), &mut c.burst_jitter_ms, 0..=60_000);
                            });
                            ui.checkbox(&mut c.breaks, l.text("Случайные длинные паузы", "Random long breaks"));
                            ui.add_enabled_ui(c.breaks, |ui| {
                                egui::Grid::new("breaks").num_columns(2).show(ui, |ui| {
                                    number(ui, l.text("Вероятность на последовательность, %", "Probability per sequence, %"), &mut c.break_probability, 0..=100);
                                    number(ui, l.text("Минимальная пауза, мс", "Minimum break, ms"), &mut c.break_min_ms, 1..=3_600_000);
                                    number(ui, l.text("Максимальная пауза, мс", "Maximum break, ms"), &mut c.break_max_ms, 1..=3_600_000);
                                });
                            });
                        });
                        egui::Grid::new("scheduled_breaks").num_columns(2).show(ui, |ui| {
                            number(ui, l.text("Регулярная пауза каждые N кликов (0 = выкл)", "Scheduled pause every N clicks (0 = off)"), &mut c.pause_every_clicks, 0..=1_000_000_000);
                            number(ui, l.text("Регулярная пауза минимум, мс", "Scheduled pause min, ms"), &mut c.pause_every_min_ms, 1..=3_600_000);
                            number(ui, l.text("Регулярная пауза максимум, мс", "Scheduled pause max, ms"), &mut c.pause_every_max_ms, 1..=3_600_000);
                        });
                    });
                    ui.separator();
                    egui::CollapsingHeader::new(l.text("Позиция и лимиты", "Position and limits")).default_open(true).show(ui, |ui| {
                        ui.checkbox(&mut c.fixed_position, l.text("Фиксированная точка (иначе — текущий курсор)", "Fixed point (otherwise follow cursor)"));
                        ui.add_enabled_ui(c.fixed_position, |ui| {
                            ui.horizontal(|ui| {
                                ui.label("X");
                                ui.add(egui::DragValue::new(&mut c.x).range(-100_000..=100_000));
                                ui.label("Y");
                                ui.add(egui::DragValue::new(&mut c.y).range(-100_000..=100_000));
                                ui.label(l.text("Разброс ± px", "Variation ± px"));
                                ui.add(egui::DragValue::new(&mut c.radius_px).range(0..=1000));
                            });
                            ui.checkbox(&mut c.restore_cursor_after_fixed, l.text("Возвращать курсор после клика", "Restore cursor after click"));
                        });
                        ui.add_enabled_ui(!c.fixed_position && c.randomize, |ui| {
                            ui.checkbox(&mut c.cursor_tremor, l.text("Микро-движение вокруг текущего курсора", "Micro-move around current cursor"));
                            ui.add_enabled_ui(c.cursor_tremor, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(l.text("Разброс ± px", "Variation ± px"));
                                    ui.add(egui::DragValue::new(&mut c.tremor_px).range(0..=1000));
                                });
                            });
                        });
                        ui.add_enabled_ui(!c.fixed_position, |ui| {
                            ui.checkbox(&mut c.stop_on_cursor_move, l.text("Fail-safe: остановить при ручном движении курсора", "Fail-safe: stop on manual cursor movement"));
                            ui.add_enabled_ui(c.stop_on_cursor_move, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(l.text("Допуск, px", "Tolerance, px"));
                                    ui.add(egui::DragValue::new(&mut c.cursor_move_tolerance_px).range(0..=10_000));
                                });
                            });
                        });
                        egui::Grid::new("limits").num_columns(2).show(ui, |ui| {
                            number(ui, l.text("Задержка старта, мс", "Start delay, ms"), &mut c.start_delay_ms, 0..=60_000);
                            number(ui, l.text("Лимит кликов (0 = без лимита)", "Click limit (0 = unlimited)"), &mut c.max_clicks, 0..=1_000_000_000);
                            number(ui, l.text("Лимит времени, с (0 = без лимита)", "Time limit, s (0 = unlimited)"), &mut c.max_duration_s, 0..=604_800);
                        });
                    });
                    ui.separator();
                    egui::CollapsingHeader::new(l.text("Горячие клавиши", "Hotkeys")).default_open(true).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(l.text("Старт/стоп", "Toggle"));
                            ui.add(egui::TextEdit::singleline(&mut c.hotkey).desired_width(200.0));
                            if ui.button(l.text("Выбрать клавишу", "Pick key")).clicked() {
                                start_hotkey_capture = true;
                            }
                            if capturing_hotkey && ui.button(l.text("Отмена", "Cancel")).clicked() {
                                cancel_hotkey_capture = true;
                            }
                        });
                        egui::ComboBox::from_id_salt("hotkey_picker")
                            .selected_text(c.hotkey.as_str())
                            .width(220.0)
                            .show_ui(ui, |ui| {
                                for key in COMMON_HOTKEYS {
                                    ui.selectable_value(&mut c.hotkey, (*key).to_owned(), *key);
                                }
                            });
                        if capturing_hotkey {
                            ui.colored_label(accent, l.text("Нажмите любую клавишу или сочетание с Ctrl/Alt/Shift/Super...", "Press any key or a Ctrl/Alt/Shift/Super shortcut..."));
                        }
                        egui::Grid::new("hotkeys").num_columns(2).show(ui, |ui| {
                            number(ui, l.text("Быстрых нажатий", "Quick taps"), &mut c.trigger_count, 1..=5);
                            number(ui, l.text("Интервал между нажатиями, мс", "Time between taps, ms"), &mut c.trigger_timeout_ms, 50..=5000);
                        });
                        ui.small(l.text("Можно выбрать из списка, нажать «Выбрать клавишу» или вручную ввести редкую клавишу вроде AudioVolumeUp. Новая привязка применяется кнопкой «Сохранить / применить» или при запуске.", "Choose from the list, press Pick key, or type a rare key like AudioVolumeUp manually. Apply a new binding with Save / apply or when starting."));
                    });
                });
                if start_hotkey_capture {
                    self.capturing_hotkey = true;
                    self.message = None;
                }
                if cancel_hotkey_capture {
                    self.capturing_hotkey = false;
                }
                ui.separator();
                egui::CollapsingHeader::new(l.text("Справка и конфигурация", "Help and configuration")).show(ui, |ui| {
                    ui.label(l.text("1. Настройте интервалы и кнопку мыши.\n2. Нажмите хоткей или «Запустить» и наведите курсор на цель.\n3. Хоткей переключает работу, Ctrl+Shift+F12 останавливает.\nEsc работает только в окне приложения, если не назначен хоткеем. Закрытие окна останавливает движок.", "1. Set timing and mouse button.\n2. Press the hotkey or Start and point at your target.\n3. The hotkey toggles clicking; Ctrl+Shift+F12 stops it.\nEscape only works inside this window unless bound as the hotkey. Closing the window stops the engine."));
                    ui.label(l.text("Интервалы — паузы после отпускания кнопки, а не целевой CPS. Лимит времени отсчитывается после задержки старта. Разгон начинает медленнее и плавно выходит на заданные интервалы. Fail-safe останавливает сессию, если курсор сдвинули вручную дальше допуска.", "Intervals are pauses after release, not target CPS. The time limit starts after the start delay. Ramp-up starts slower and fades into the configured timing. Fail-safe stops a session when the cursor is manually moved beyond tolerance."));
                    ui.label(l.text("Изменения используются при следующем старте. «Сохранить / применить» записывает настройки на диск. Для чтения внешних изменений нажмите «Перечитать».", "Changes are used on the next start. Save / apply writes settings to disk. Use Reload to read external edits."));
                    if let Ok(path) = config::config_path() {
                        ui.label(path.display().to_string());
                    }
                    ui.horizontal(|ui| {
                        if ui.add_enabled(!running, egui::Button::new(l.text("Перечитать", "Reload"))).clicked() {
                            match config::load() {
                                Ok(config) => {
                                    self.saved = config.clone();
                                    self.draft = config;
                                    if let Err(error) = self.apply() {
                                        self.message = Some((true, format!("{error:#}")));
                                    } else {
                                        self.message = None;
                                    }
                                }
                                Err(error) => self.message = Some((true, format!("{error:#}"))),
                            }
                        }
                        if ui.add_enabled(!running, egui::Button::new(l.text("Настройки по умолчанию", "Defaults"))).clicked() {
                            self.draft = Config { language: l, ..Config::default() };
                        }
                    });
                    ui.label("Windows · Linux/X11 · macOS (Accessibility permission) · FreeBSD/OpenBSD experimental");
                    ui.hyperlink_to("GPL-3.0-only · Lion AutoClicker 0.1.0", "https://www.gnu.org/licenses/gpl-3.0.html");
                });
            });
        });
        // Worker statistics need repainting; idle hotkeys wake the UI via their handler.
        if running || self.engine.status().running() {
            ctx.request_repaint_after(Duration::from_millis(33));
        }
    }
}

fn accent_color(config: &Config) -> Color32 {
    Color32::from_rgb(config.accent_r, config.accent_g, config.accent_b)
}

fn darken(color: Color32, factor: f32) -> Color32 {
    Color32::from_rgb(
        (color.r() as f32 * factor).round() as u8,
        (color.g() as f32 * factor).round() as u8,
        (color.b() as f32 * factor).round() as u8,
    )
}

fn apply_ui_style(ctx: &egui::Context, config: &Config) {
    let accent = accent_color(config);
    let mut style = (*ctx.style()).clone();
    style.visuals = match config.theme {
        Theme::Light => egui::Visuals::light(),
        _ => egui::Visuals::dark(),
    };
    match config.theme {
        Theme::LionDark => {
            style.visuals.panel_fill = Color32::from_rgb(15, 13, 10);
            style.visuals.window_fill = Color32::from_rgb(22, 19, 14);
            style.visuals.extreme_bg_color = Color32::from_rgb(8, 7, 6);
        }
        Theme::Midnight => {
            style.visuals.panel_fill = Color32::from_rgb(8, 12, 25);
            style.visuals.window_fill = Color32::from_rgb(12, 17, 34);
            style.visuals.extreme_bg_color = Color32::from_rgb(5, 8, 18);
        }
        Theme::Forest => {
            style.visuals.panel_fill = Color32::from_rgb(8, 20, 15);
            style.visuals.window_fill = Color32::from_rgb(12, 31, 23);
            style.visuals.extreme_bg_color = Color32::from_rgb(4, 12, 9);
        }
        Theme::Light => {
            style.visuals.panel_fill = Color32::from_rgb(247, 242, 232);
            style.visuals.window_fill = Color32::from_rgb(255, 250, 240);
            style.visuals.extreme_bg_color = Color32::from_rgb(235, 229, 217);
        }
        Theme::HighContrast => {
            style.visuals.panel_fill = Color32::BLACK;
            style.visuals.window_fill = Color32::BLACK;
            style.visuals.extreme_bg_color = Color32::BLACK;
            style.visuals.override_text_color = Some(Color32::WHITE);
            style.visuals.widgets.noninteractive.fg_stroke.color = Color32::WHITE;
            style.visuals.widgets.inactive.fg_stroke.color = Color32::WHITE;
            style.visuals.widgets.hovered.fg_stroke.color = Color32::WHITE;
            style.visuals.widgets.active.fg_stroke.color = Color32::WHITE;
        }
    }
    style.visuals.selection.bg_fill = darken(accent, 0.55);
    style.visuals.selection.stroke.color = accent;
    style.visuals.hyperlink_color = accent;
    style.visuals.warn_fg_color = accent;
    style.visuals.widgets.hovered.bg_stroke.color = accent;
    style.visuals.widgets.active.bg_stroke.color = accent;
    style.visuals.collapsing_header_frame = true;
    style.visuals.slider_trailing_fill = true;
    style.spacing.item_spacing = if config.compact_mode {
        egui::vec2(6.0, 5.0)
    } else {
        egui::vec2(10.0, 9.0)
    };
    ctx.set_zoom_factor(config.ui_scale_percent as f32 / 100.0);
    ctx.set_style(style);
}

fn capture_hotkey(ctx: &egui::Context) -> Option<String> {
    ctx.input(|input| {
        for event in &input.events {
            if let egui::Event::Key {
                key,
                pressed: true,
                repeat: false,
                modifiers,
                ..
            } = event
            {
                let Some(key) = egui_key_to_hotkey(*key) else {
                    continue;
                };
                let mut parts = Vec::new();
                if modifiers.ctrl {
                    parts.push("Ctrl");
                }
                if modifiers.alt {
                    parts.push("Alt");
                }
                if modifiers.shift {
                    parts.push("Shift");
                }
                if modifiers.mac_cmd {
                    parts.push("Super");
                }
                parts.push(key);
                return Some(parts.join("+"));
            }
        }
        None
    })
}

fn egui_key_to_hotkey(key: egui::Key) -> Option<&'static str> {
    Some(match key {
        egui::Key::ArrowDown => "ArrowDown",
        egui::Key::ArrowLeft => "ArrowLeft",
        egui::Key::ArrowRight => "ArrowRight",
        egui::Key::ArrowUp => "ArrowUp",
        egui::Key::Escape => "Escape",
        egui::Key::Tab => "Tab",
        egui::Key::Backspace => "Backspace",
        egui::Key::Enter => "Enter",
        egui::Key::Space => "Space",
        egui::Key::Insert => "Insert",
        egui::Key::Delete => "Delete",
        egui::Key::Home => "Home",
        egui::Key::End => "End",
        egui::Key::PageUp => "PageUp",
        egui::Key::PageDown => "PageDown",
        egui::Key::Comma => "Comma",
        egui::Key::Backslash | egui::Key::Pipe => "Backslash",
        egui::Key::Slash | egui::Key::Questionmark => "Slash",
        egui::Key::Backtick => "Backquote",
        egui::Key::Minus => "Minus",
        egui::Key::Period => "Period",
        egui::Key::Plus | egui::Key::Equals => "Equal",
        egui::Key::Semicolon | egui::Key::Colon => "Semicolon",
        egui::Key::Quote => "Quote",
        egui::Key::OpenBracket | egui::Key::OpenCurlyBracket => "BracketLeft",
        egui::Key::CloseBracket | egui::Key::CloseCurlyBracket => "BracketRight",
        egui::Key::Exclamationmark => "1",
        egui::Key::Num0 => "0",
        egui::Key::Num1 => "1",
        egui::Key::Num2 => "2",
        egui::Key::Num3 => "3",
        egui::Key::Num4 => "4",
        egui::Key::Num5 => "5",
        egui::Key::Num6 => "6",
        egui::Key::Num7 => "7",
        egui::Key::Num8 => "8",
        egui::Key::Num9 => "9",
        egui::Key::A => "A",
        egui::Key::B => "B",
        egui::Key::C => "C",
        egui::Key::D => "D",
        egui::Key::E => "E",
        egui::Key::F => "F",
        egui::Key::G => "G",
        egui::Key::H => "H",
        egui::Key::I => "I",
        egui::Key::J => "J",
        egui::Key::K => "K",
        egui::Key::L => "L",
        egui::Key::M => "M",
        egui::Key::N => "N",
        egui::Key::O => "O",
        egui::Key::P => "P",
        egui::Key::Q => "Q",
        egui::Key::R => "R",
        egui::Key::S => "S",
        egui::Key::T => "T",
        egui::Key::U => "U",
        egui::Key::V => "V",
        egui::Key::W => "W",
        egui::Key::X => "X",
        egui::Key::Y => "Y",
        egui::Key::Z => "Z",
        egui::Key::F1 => "F1",
        egui::Key::F2 => "F2",
        egui::Key::F3 => "F3",
        egui::Key::F4 => "F4",
        egui::Key::F5 => "F5",
        egui::Key::F6 => "F6",
        egui::Key::F7 => "F7",
        egui::Key::F8 => "F8",
        egui::Key::F9 => "F9",
        egui::Key::F10 => "F10",
        egui::Key::F11 => "F11",
        egui::Key::F12 => "F12",
        egui::Key::F13 => "F13",
        egui::Key::F14 => "F14",
        egui::Key::F15 => "F15",
        egui::Key::F16 => "F16",
        egui::Key::F17 => "F17",
        egui::Key::F18 => "F18",
        egui::Key::F19 => "F19",
        egui::Key::F20 => "F20",
        egui::Key::F21 => "F21",
        egui::Key::F22 => "F22",
        egui::Key::F23 => "F23",
        egui::Key::F24 => "F24",
        _ => return None,
    })
}

fn number<N: egui::emath::Numeric>(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut N,
    range: std::ops::RangeInclusive<N>,
) {
    ui.label(label);
    ui.add(egui::DragValue::new(value).range(range).speed(1.0));
    ui.end_row();
}
