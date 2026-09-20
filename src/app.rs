use crate::{
    config::{self, Button, Config, Distribution, Language, Pattern},
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

const ACCENT: Color32 = Color32::from_rgb(239, 179, 74);

pub struct App {
    draft: Config,
    applied: Config,
    saved: Config,
    engine: Engine,
    hotkeys: Option<Hotkeys>,
    hotkey_error: Option<String>,
    taps: TapTracker,
    events: mpsc::Receiver<GlobalHotKeyEvent>,
    message: Option<(bool, String)>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Result<Self> {
        let mut style = (*cc.egui_ctx.style()).clone();
        style.visuals = egui::Visuals::dark();
        style.visuals.selection.bg_fill = Color32::from_rgb(120, 82, 28);
        style.spacing.item_spacing = egui::vec2(10.0, 9.0);
        cc.egui_ctx.set_style(style);

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

    fn status_panel(&self, ui: &mut egui::Ui, status: &Status) {
        let l = self.draft.language;
        let (label, color) = match status.phase {
            Phase::Idle => (l.text("ОЖИДАНИЕ", "IDLE"), Color32::GRAY),
            Phase::Countdown => (l.text("ПОДГОТОВКА", "COUNTDOWN"), ACCENT),
            Phase::Clicking => (l.text("КЛИКИ", "CLICKING"), Color32::LIGHT_GREEN),
            Phase::Break => (l.text("ПАУЗА", "BREAK"), ACCENT),
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
        self.process_events();
        let status = self.engine.status();
        if ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
            self.engine.stop();
            self.taps = TapTracker::default();
        }
        let running = status.running();
        let l = self.draft.language;

        egui::TopBottomPanel::top("heading").show(ctx, |ui| {
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.heading(RichText::new("LION").color(ACCENT).size(30.0));
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
            self.status_panel(ui, &status);
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
                            Color32::from_rgb(105, 76, 27)
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
                    ui.label(RichText::new(l.text("Не сохранено", "Unsaved")).color(ACCENT));
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
                    ui.colored_label(ACCENT, l.text("Не удалось применить хоткей. Используйте кнопки окна; активная привязка показана внизу.", "Could not apply hotkey. Use window controls; the active binding is shown below."));
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
                    ui.colored_label(ACCENT, l.text("Сеанс Wayland: клики и хоткеи ограничены XWayland. Для всего рабочего стола войдите в сеанс X11.", "Wayland session: input and hotkeys are limited to XWayland. Use an X11 session for full desktop support."));
                }
                ui.add_enabled_ui(!running, |ui| {
                    let c = &mut self.draft;
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
                            ui.label("F6 / Ctrl+Shift+F8");
                        });
                        egui::Grid::new("hotkeys").num_columns(2).show(ui, |ui| {
                            number(ui, l.text("Быстрых нажатий", "Quick taps"), &mut c.trigger_count, 1..=5);
                            number(ui, l.text("Интервал между нажатиями, мс", "Time between taps, ms"), &mut c.trigger_timeout_ms, 50..=5000);
                        });
                        ui.small(l.text("Новая привязка применяется кнопкой «Сохранить / применить» или при запуске.", "Apply a new binding with Save / apply or when starting."));
                    });
                });
                ui.separator();
                egui::CollapsingHeader::new(l.text("Справка и конфигурация", "Help and configuration")).show(ui, |ui| {
                    ui.label(l.text("1. Настройте интервалы и кнопку мыши.\n2. Нажмите F6 или «Запустить» и наведите курсор на цель.\n3. F6 переключает работу, Ctrl+Shift+F12 останавливает.\nEsc работает только в окне приложения. Закрытие окна останавливает движок.", "1. Set timing and mouse button.\n2. Press F6 or Start and point at your target.\n3. F6 toggles clicking; Ctrl+Shift+F12 stops it.\nEscape only works inside this window. Closing the window stops the engine."));
                    ui.label(l.text("Интервалы — паузы после отпускания кнопки, а не целевой CPS. Лимит времени отсчитывается после задержки старта. Разброс позиции работает для фиксированной точки или микро-движения курсора при включённых вариациях.", "Intervals are pauses after release, not a target CPS. The time limit starts after the start delay. Position variation works with a fixed point or cursor micro-move when random variation is enabled."));
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
                    ui.label("Windows · Linux/X11 · macOS (Accessibility permission)");
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
