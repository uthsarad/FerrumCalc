/// FerrumCalc – User Interface
///
/// Renders the entire calculator GUI using egui. Implements:
///   - Mode switching (Standard / Scientific / Programmer)
///   - Dynamic keypad layouts
///   - Collapsible history sidebar
///   - Dark/light theme with Fluent-inspired aesthetics
///   - Full keyboard input handling

use eframe::egui;
use crate::calculator::parser;
use crate::calculator::state::{AngleMode, CalculatorMode, CalculatorState, NumBase};

// ── Color Palette ────────────────────────────────────────────────────────────

struct Palette;

impl Palette {
    // Dark theme
    const DARK_BG:          egui::Color32 = egui::Color32::from_rgb(24, 24, 32);
    const DARK_SURFACE:     egui::Color32 = egui::Color32::from_rgb(32, 33, 44);
    const DARK_CARD:        egui::Color32 = egui::Color32::from_rgb(40, 42, 56);
    const DARK_BTN:         egui::Color32 = egui::Color32::from_rgb(50, 53, 70);
    const DARK_BTN_HOVER:   egui::Color32 = egui::Color32::from_rgb(65, 68, 90);
    const DARK_BTN_OP:      egui::Color32 = egui::Color32::from_rgb(55, 58, 80);
    const DARK_BTN_OP_HOVER:egui::Color32 = egui::Color32::from_rgb(75, 78, 105);
    const DARK_TEXT:        egui::Color32 = egui::Color32::from_rgb(235, 235, 245);
    const DARK_TEXT_DIM:    egui::Color32 = egui::Color32::from_rgb(160, 165, 185);
    const DARK_SIDEBAR:     egui::Color32 = egui::Color32::from_rgb(28, 28, 38);

    // Light theme
    const LIGHT_BG:          egui::Color32 = egui::Color32::from_rgb(243, 243, 248);
    const LIGHT_SURFACE:     egui::Color32 = egui::Color32::from_rgb(255, 255, 255);
    const LIGHT_CARD:        egui::Color32 = egui::Color32::from_rgb(248, 248, 252);
    const LIGHT_BTN:         egui::Color32 = egui::Color32::from_rgb(235, 237, 244);
    const LIGHT_BTN_HOVER:   egui::Color32 = egui::Color32::from_rgb(220, 222, 232);
    const LIGHT_BTN_OP:      egui::Color32 = egui::Color32::from_rgb(225, 228, 240);
    const LIGHT_BTN_OP_HOVER:egui::Color32 = egui::Color32::from_rgb(210, 213, 228);
    const LIGHT_TEXT:        egui::Color32 = egui::Color32::from_rgb(30, 30, 40);
    const LIGHT_TEXT_DIM:    egui::Color32 = egui::Color32::from_rgb(100, 105, 120);
    const LIGHT_SIDEBAR:     egui::Color32 = egui::Color32::from_rgb(238, 238, 244);

    // Accent colors (shared)
    const ACCENT:       egui::Color32 = egui::Color32::from_rgb(88, 101, 242);  // Soft indigo
    const ACCENT_HOVER: egui::Color32 = egui::Color32::from_rgb(108, 121, 255);
    const ERROR:        egui::Color32 = egui::Color32::from_rgb(237, 66, 69);
    const ACCENT_TEXT:  egui::Color32 = egui::Color32::from_rgb(255, 255, 255);
}

// ── Theme helpers ────────────────────────────────────────────────────────────

struct Theme {
    bg: egui::Color32,
    surface: egui::Color32,
    card: egui::Color32,
    btn: egui::Color32,
    btn_hover: egui::Color32,
    btn_op: egui::Color32,
    btn_op_hover: egui::Color32,
    text: egui::Color32,
    text_dim: egui::Color32,
    sidebar: egui::Color32,
}

impl Theme {
    fn dark() -> Self {
        Self {
            bg: Palette::DARK_BG,
            surface: Palette::DARK_SURFACE,
            card: Palette::DARK_CARD,
            btn: Palette::DARK_BTN,
            btn_hover: Palette::DARK_BTN_HOVER,
            btn_op: Palette::DARK_BTN_OP,
            btn_op_hover: Palette::DARK_BTN_OP_HOVER,
            text: Palette::DARK_TEXT,
            text_dim: Palette::DARK_TEXT_DIM,
            sidebar: Palette::DARK_SIDEBAR,
        }
    }

    fn light() -> Self {
        Self {
            bg: Palette::LIGHT_BG,
            surface: Palette::LIGHT_SURFACE,
            card: Palette::LIGHT_CARD,
            btn: Palette::LIGHT_BTN,
            btn_hover: Palette::LIGHT_BTN_HOVER,
            btn_op: Palette::LIGHT_BTN_OP,
            btn_op_hover: Palette::LIGHT_BTN_OP_HOVER,
            text: Palette::LIGHT_TEXT,
            text_dim: Palette::LIGHT_TEXT_DIM,
            sidebar: Palette::LIGHT_SIDEBAR,
        }
    }

    fn current(dark: bool) -> Self {
        if dark { Self::dark() } else { Self::light() }
    }
}

// ── Main App ─────────────────────────────────────────────────────────────────

pub struct FerrumCalcApp {
    pub state: CalculatorState,
}

impl Default for FerrumCalcApp {
    fn default() -> Self {
        Self {
            state: CalculatorState::default(),
        }
    }
}

impl FerrumCalcApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Set up custom fonts
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "mono_display".to_owned(),
            std::sync::Arc::new(egui::FontData::from_static(
                include_bytes!("../assets/JetBrainsMono-Regular.ttf"),
            )),
        );
        fonts
            .families
            .entry(egui::FontFamily::Name("Display".into()))
            .or_default()
            .push("mono_display".to_owned());
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "mono_display".to_owned());

        cc.egui_ctx.set_fonts(fonts);

        // Apply initial visual style
        let mut style = (*cc.egui_ctx.style()).clone();
        style.visuals.window_corner_radius = egui::CornerRadius::same(12);
        style.visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(8);
        style.visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(8);
        style.visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(8);
        style.visuals.widgets.active.corner_radius = egui::CornerRadius::same(8);
        style.spacing.item_spacing = egui::vec2(6.0, 6.0);
        cc.egui_ctx.set_style(style);

        // Restore persisted state (history + preferences) from a previous session.
        // Falls back to defaults on first launch or if the stored data is unreadable.
        let state = cc
            .storage
            .and_then(|storage| eframe::get_value::<CalculatorState>(storage, eframe::APP_KEY))
            .map(|mut state| {
                // Start every session with a clean slate: saved preferences and
                // history are kept, but the working expression and result are
                // cleared so the calculator always opens fresh.
                state.input.clear();
                state.result_display = "0".to_string();
                state.has_error = false;
                state.just_evaluated = false;
                state
            })
            .unwrap_or_default();

        Self { state }
    }
}

impl eframe::App for FerrumCalcApp {
    /// Persist the full calculator state (history and preferences) to disk.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.state);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let theme = Theme::current(self.state.dark_mode);

        // Apply theme to visuals
        self.apply_theme(ctx, &theme);

        // Handle keyboard input
        self.handle_keyboard(ctx);

        // History sidebar
        if self.state.history_open {
            self.render_history_sidebar(ctx, &theme);
        }

        // Main panel
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme.bg).inner_margin(egui::Margin::same(16)))
            .show(ctx, |ui| {
                // Top toolbar: mode selector + theme toggle + history toggle
                self.render_toolbar(ui, &theme);

                ui.add_space(8.0);

                if self.state.mode == CalculatorMode::Graph {
                    // Graph mode (draft): function display + live plot + compact keypad.
                    let use_degrees = self.state.angle_mode == AngleMode::Degrees;
                    self.render_graph_display(ui, &theme, use_degrees);
                    ui.add_space(8.0);
                    self.render_graph_canvas(ui, &theme, use_degrees);
                    ui.add_space(8.0);
                    self.render_graph_keypad(ui, &theme);
                } else {
                    // Display area
                    self.render_display(ui, &theme);

                    ui.add_space(8.0);

                    // Programmer mode: base selector & conversions
                    if self.state.mode == CalculatorMode::Programmer {
                        self.render_base_selector(ui, &theme);
                        ui.add_space(4.0);
                    }

                    // Scientific mode: angle mode toggle
                    if self.state.mode == CalculatorMode::Scientific {
                        self.render_angle_toggle(ui, &theme);
                        ui.add_space(4.0);
                    }

                    // Keypad
                    self.render_keypad(ui, &theme);
                }
            });
    }
}

// ── Rendering methods ────────────────────────────────────────────────────────

impl FerrumCalcApp {
    fn apply_theme(&self, ctx: &egui::Context, theme: &Theme) {
        let mut visuals = if self.state.dark_mode {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        visuals.panel_fill = theme.bg;
        visuals.window_fill = theme.surface;
        visuals.override_text_color = Some(theme.text);
        visuals.widgets.noninteractive.bg_fill = theme.surface;
        visuals.widgets.inactive.bg_fill = theme.btn;
        visuals.widgets.hovered.bg_fill = theme.btn_hover;
        visuals.widgets.active.bg_fill = Palette::ACCENT;
        visuals.widgets.inactive.weak_bg_fill = theme.btn;
        visuals.widgets.hovered.weak_bg_fill = theme.btn_hover;
        visuals.widgets.active.weak_bg_fill = Palette::ACCENT;
        visuals.window_corner_radius = egui::CornerRadius::same(12);
        visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(8);
        visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(8);
        visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(8);
        visuals.widgets.active.corner_radius = egui::CornerRadius::same(8);
        ctx.set_visuals(visuals);
    }

    // ── Toolbar ──

    fn render_toolbar(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        ui.horizontal(|ui| {
            if self.state.history_open {
                // Compact mode: use a ComboBox for mode switching
                egui::ComboBox::from_id_salt("mode_select")
                    .width(90.0)
                    .selected_text(self.state.mode.label())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.state.mode, CalculatorMode::Standard, "Standard");
                        ui.selectable_value(&mut self.state.mode, CalculatorMode::Scientific, "Scientific");
                        ui.selectable_value(&mut self.state.mode, CalculatorMode::Programmer, "Programmer");
                        ui.selectable_value(&mut self.state.mode, CalculatorMode::Graph, "Graph");
                    });
            } else {
                // Wide mode: show all mode buttons
                for mode in &[CalculatorMode::Standard, CalculatorMode::Scientific, CalculatorMode::Programmer, CalculatorMode::Graph] {
                    let selected = self.state.mode == *mode;
                    let btn_color = if selected { Palette::ACCENT } else { theme.btn };
                    let text_color = if selected { Palette::ACCENT_TEXT } else { theme.text_dim };

                    let btn = egui::Button::new(
                        egui::RichText::new(mode.label())
                            .color(text_color)
                            .size(11.0)
                    )
                    .fill(btn_color)
                    .corner_radius(egui::CornerRadius::same(6))
                    .min_size(egui::vec2(0.0, 28.0));

                    if ui.add(btn).clicked() {
                        self.state.mode = *mode;
                    }
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // History toggle
                let history_icon = if self.state.history_open { "⏪" } else { "🕐" };
                let history_btn = egui::Button::new(
                    egui::RichText::new(history_icon).size(16.0).color(theme.text_dim)
                )
                .fill(egui::Color32::TRANSPARENT)
                .min_size(egui::vec2(28.0, 28.0));
                if ui.add(history_btn).clicked() {
                    self.state.toggle_history();
                }

                // Theme toggle
                let theme_icon = if self.state.dark_mode { "☀" } else { "🌙" };
                let theme_btn = egui::Button::new(
                    egui::RichText::new(theme_icon).size(16.0).color(theme.text_dim)
                )
                .fill(egui::Color32::TRANSPARENT)
                .min_size(egui::vec2(28.0, 28.0));
                if ui.add(theme_btn).clicked() {
                    self.state.dark_mode = !self.state.dark_mode;
                }
            });
        });
    }

    // ── Display ──

    fn render_display(&self, ui: &mut egui::Ui, theme: &Theme) {
        egui::Frame::new()
            .fill(theme.card)
            .corner_radius(egui::CornerRadius::same(12))
            .inner_margin(egui::Margin::symmetric(16, 12))
            .show(ui, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Max), |ui| {
                    ui.label(
                        egui::RichText::new(&self.state.input)
                            .color(theme.text_dim)
                            .family(egui::FontFamily::Name("Display".into()))
                            .size(16.0),
                    );
                    ui.add_space(4.0);
                    let result_color = if self.state.has_error {
                        Palette::ERROR
                    } else {
                        theme.text
                    };
                    ui.label(
                        egui::RichText::new(&self.state.result_display)
                            .color(result_color)
                            .family(egui::FontFamily::Name("Display".into()))
                            .size(32.0),
                    );
                });

                // Programmer mode: show base conversions
                if self.state.mode == CalculatorMode::Programmer && !self.state.has_error {
                    if let Ok(val) = self.state.result_display.parse::<f64>() {
                        if val.fract() == 0.0 {
                            let int_val = val as i64;
                            ui.add_space(6.0);
                            ui.separator();
                            ui.add_space(4.0);
                            let conversions = [
                                ("HEX", format!("{:X}", int_val)),
                                ("DEC", format!("{}", int_val)),
                                ("OCT", format!("{:o}", int_val)),
                                ("BIN", format!("{:b}", int_val)),
                            ];
                            for (label, value) in &conversions {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(*label)
                                            .color(Palette::ACCENT)
                                            .size(10.0)
                                            .strong(),
                                    );
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.label(
                                                egui::RichText::new(value)
                                                    .color(theme.text_dim)
                                                    .family(egui::FontFamily::Name("Display".into()))
                                                    .size(12.0),
                                            );
                                        },
                                    );
                                });
                            }
                        }
                    }
                }
            });
    }

    // ── Base Selector (Programmer mode) ──

    fn render_base_selector(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        ui.horizontal(|ui| {
            for base in &[NumBase::Hex, NumBase::Dec, NumBase::Oct, NumBase::Bin] {
                let selected = self.state.base == *base;
                let btn_color = if selected { Palette::ACCENT } else { theme.btn };
                let text_color = if selected { Palette::ACCENT_TEXT } else { theme.text_dim };

                let btn = egui::Button::new(
                    egui::RichText::new(base.label())
                        .color(text_color)
                        .size(11.0)
                        .strong()
                )
                .fill(btn_color)
                .corner_radius(egui::CornerRadius::same(6))
                .min_size(egui::vec2(0.0, 24.0));

                if ui.add(btn).clicked() {
                    self.state.base = *base;
                }
            }
        });
    }

    // ── Angle Mode Toggle (Scientific mode) ──

    fn render_angle_toggle(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        ui.horizontal(|ui| {
            for mode in &[AngleMode::Radians, AngleMode::Degrees] {
                let selected = self.state.angle_mode == *mode;
                let btn_color = if selected { Palette::ACCENT } else { theme.btn };
                let text_color = if selected { Palette::ACCENT_TEXT } else { theme.text_dim };

                let btn = egui::Button::new(
                    egui::RichText::new(mode.label())
                        .color(text_color)
                        .size(11.0)
                        .strong()
                )
                .fill(btn_color)
                .corner_radius(egui::CornerRadius::same(6))
                .min_size(egui::vec2(0.0, 24.0));

                if ui.add(btn).clicked() {
                    self.state.angle_mode = *mode;
                }
            }
        });
    }

    // ── Graph Mode (draft) ──

    /// Compact "y = f(x)" header for graph mode, with a DRAFT badge and an
    /// inline parse-error message when the expression can't be evaluated.
    fn render_graph_display(&self, ui: &mut egui::Ui, theme: &Theme, use_degrees: bool) {
        egui::Frame::new()
            .fill(theme.card)
            .corner_radius(egui::CornerRadius::same(12))
            .inner_margin(egui::Margin::symmetric(16, 10))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("GRAPH")
                            .color(Palette::ACCENT)
                            .size(11.0)
                            .strong(),
                    );
                    // Small "DRAFT" badge to flag this as an experimental feature.
                    egui::Frame::new()
                        .fill(Palette::ACCENT.gamma_multiply(0.25))
                        .corner_radius(egui::CornerRadius::same(4))
                        .inner_margin(egui::Margin::symmetric(5, 1))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new("DRAFT").color(theme.text_dim).size(9.0).strong(),
                            );
                        });
                });
                ui.add_space(2.0);

                let expr = if self.state.input.trim().is_empty() {
                    "…".to_string()
                } else {
                    self.state.input.clone()
                };
                ui.label(
                    egui::RichText::new(format!("y = {}", expr))
                        .color(theme.text)
                        .family(egui::FontFamily::Name("Display".into()))
                        .size(18.0),
                );

                if let Some(err) = self.graph_error(use_degrees) {
                    ui.add_space(2.0);
                    ui.label(egui::RichText::new(err).color(Palette::ERROR).size(11.0));
                }
            });
    }

    /// Returns a parse/evaluation error if the current expression fails for
    /// every sampled `x`. Domain errors at isolated points (e.g. `1/x` at 0)
    /// don't count, so a valid function is never flagged.
    fn graph_error(&self, use_degrees: bool) -> Option<String> {
        if self.state.input.trim().is_empty() {
            return None;
        }
        let mut first_err = None;
        for &x in &[0.5_f64, 1.0, 2.0, -1.0, 3.0] {
            match parser::evaluate_at(&self.state.input, use_degrees, x) {
                Ok(v) if v.is_finite() => return None,
                Ok(_) => {}
                Err(e) => {
                    if first_err.is_none() {
                        first_err = Some(e);
                    }
                }
            }
        }
        first_err
    }

    /// Draws the plot: a fixed [-10, 10] × [-10, 10] grid with axes and the
    /// sampled curve of `y = f(x)`.
    fn render_graph_canvas(&self, ui: &mut egui::Ui, theme: &Theme, use_degrees: bool) {
        // Give the plot roughly half the remaining height, leaving the rest for
        // the keypad; clamp so neither collapses on small windows.
        let total_h = ui.available_height();
        let plot_h = (total_h * 0.5).max(120.0).min((total_h - 120.0).max(120.0));
        let width = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(egui::vec2(width, plot_h), egui::Sense::hover());

        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, egui::CornerRadius::same(12), theme.card);

        let (xmin, xmax, ymin, ymax) = (-10.0_f64, 10.0, -10.0, 10.0);
        let to_screen = |wx: f64, wy: f64| {
            let nx = ((wx - xmin) / (xmax - xmin)) as f32;
            let ny = ((wy - ymin) / (ymax - ymin)) as f32;
            egui::pos2(rect.left() + nx * rect.width(), rect.bottom() - ny * rect.height())
        };

        let grid = egui::Stroke::new(1.0, theme.text_dim.gamma_multiply(0.22));
        let axis = egui::Stroke::new(1.5, theme.text_dim.gamma_multiply(0.7));

        for i in (xmin as i32)..=(xmax as i32) {
            let x = i as f64;
            let s = if i == 0 { axis } else { grid };
            painter.line_segment([to_screen(x, ymin), to_screen(x, ymax)], s);
        }
        for j in (ymin as i32)..=(ymax as i32) {
            let y = j as f64;
            let s = if j == 0 { axis } else { grid };
            painter.line_segment([to_screen(xmin, y), to_screen(xmax, y)], s);
        }

        // Sample the curve one point per horizontal pixel.
        if !self.state.input.trim().is_empty() {
            let curve = egui::Stroke::new(2.0, Palette::ACCENT);
            let n = rect.width().max(2.0) as usize;
            let mut prev: Option<(f64, f64)> = None;
            for px in 0..=n {
                let wx = xmin + (px as f64 / n as f64) * (xmax - xmin);
                match parser::evaluate_at(&self.state.input, use_degrees, wx) {
                    Ok(wy) if wy.is_finite() => {
                        if let Some((pwx, pwy)) = prev {
                            // Skip the segment across a likely asymptote / discontinuity.
                            if (wy - pwy).abs() <= (ymax - ymin) * 1.5 {
                                painter.line_segment([to_screen(pwx, pwy), to_screen(wx, wy)], curve);
                            }
                        }
                        prev = Some((wx, wy));
                    }
                    _ => prev = None,
                }
            }
        }

        painter.text(
            rect.left_top() + egui::vec2(8.0, 6.0),
            egui::Align2::LEFT_TOP,
            "x, y ∈ [−10, 10]",
            egui::FontId::proportional(10.0),
            theme.text_dim,
        );
    }

    /// Compact keypad for graph mode. Advanced functions can also be typed on
    /// the keyboard (letters are accepted in graph mode).
    fn render_graph_keypad(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        let rows: &[&[(&str, ButtonKind)]] = &[
            &[("x", ButtonKind::Variable), ("(", ButtonKind::Paren), (")", ButtonKind::Paren), ("^", ButtonKind::Operator), ("√", ButtonKind::SciFunc)],
            &[("sin", ButtonKind::SciFunc), ("cos", ButtonKind::SciFunc), ("tan", ButtonKind::SciFunc), ("π", ButtonKind::Constant), ("e", ButtonKind::Constant)],
            &[("7", ButtonKind::Digit), ("8", ButtonKind::Digit), ("9", ButtonKind::Digit), ("÷", ButtonKind::Operator), ("⌫", ButtonKind::Backspace)],
            &[("4", ButtonKind::Digit), ("5", ButtonKind::Digit), ("6", ButtonKind::Digit), ("×", ButtonKind::Operator), ("C", ButtonKind::Clear)],
            &[("1", ButtonKind::Digit), ("2", ButtonKind::Digit), ("3", ButtonKind::Digit), ("−", ButtonKind::Operator), ("+", ButtonKind::Operator)],
            &[("±", ButtonKind::Function), ("0", ButtonKind::Digit), (".", ButtonKind::Digit), ("%", ButtonKind::Operator), ("", ButtonKind::Spacer)],
        ];
        self.render_button_grid(ui, theme, rows);
    }

    // ── Keypad ──

    fn render_keypad(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        match self.state.mode {
            CalculatorMode::Standard   => self.render_standard_keypad(ui, theme),
            CalculatorMode::Scientific => self.render_scientific_keypad(ui, theme),
            CalculatorMode::Programmer => self.render_programmer_keypad(ui, theme),
            CalculatorMode::Graph      => self.render_graph_keypad(ui, theme),
        }
    }

    fn render_standard_keypad(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        let rows: &[&[(&str, ButtonKind)]] = &[
            &[("C", ButtonKind::Clear), ("⌫", ButtonKind::Backspace), ("%", ButtonKind::Operator), ("÷", ButtonKind::Operator)],
            &[("7", ButtonKind::Digit), ("8", ButtonKind::Digit), ("9", ButtonKind::Digit), ("×", ButtonKind::Operator)],
            &[("4", ButtonKind::Digit), ("5", ButtonKind::Digit), ("6", ButtonKind::Digit), ("−", ButtonKind::Operator)],
            &[("1", ButtonKind::Digit), ("2", ButtonKind::Digit), ("3", ButtonKind::Digit), ("+", ButtonKind::Operator)],
            &[("±", ButtonKind::Function), ("0", ButtonKind::Digit), (".", ButtonKind::Digit), ("=", ButtonKind::Equals)],
        ];
        self.render_button_grid(ui, theme, rows);
    }

    fn render_scientific_keypad(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        let rows: &[&[(&str, ButtonKind)]] = &[
            &[("sin", ButtonKind::SciFunc), ("cos", ButtonKind::SciFunc), ("tan", ButtonKind::SciFunc), ("π", ButtonKind::Constant), ("e", ButtonKind::Constant)],
            &[("ln", ButtonKind::SciFunc), ("log", ButtonKind::SciFunc), ("√", ButtonKind::SciFunc), ("x²", ButtonKind::SciFunc), ("xⁿ", ButtonKind::SciFunc)],
            &[("(", ButtonKind::Paren), (")", ButtonKind::Paren), ("n!", ButtonKind::SciFunc), ("C", ButtonKind::Clear), ("⌫", ButtonKind::Backspace)],
            &[("7", ButtonKind::Digit), ("8", ButtonKind::Digit), ("9", ButtonKind::Digit), ("÷", ButtonKind::Operator), ("%", ButtonKind::Operator)],
            &[("4", ButtonKind::Digit), ("5", ButtonKind::Digit), ("6", ButtonKind::Digit), ("×", ButtonKind::Operator), ("^", ButtonKind::Operator)],
            &[("1", ButtonKind::Digit), ("2", ButtonKind::Digit), ("3", ButtonKind::Digit), ("−", ButtonKind::Operator), ("+", ButtonKind::Operator)],
            &[("±", ButtonKind::Function), ("0", ButtonKind::Digit), (".", ButtonKind::Digit), ("=", ButtonKind::Equals), ("", ButtonKind::Spacer)],
        ];
        self.render_button_grid(ui, theme, rows);
    }

    fn render_programmer_keypad(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        let base = self.state.base;
        let rows: &[&[(&str, ButtonKind)]] = &[
            &[("AND", ButtonKind::BitOp), ("OR", ButtonKind::BitOp), ("XOR", ButtonKind::BitOp), ("NOT", ButtonKind::BitOp)],
            &[("<<", ButtonKind::BitOp), (">>", ButtonKind::BitOp), ("C", ButtonKind::Clear), ("⌫", ButtonKind::Backspace)],
            &[("A", ButtonKind::HexDigit), ("B", ButtonKind::HexDigit), ("(", ButtonKind::Paren), (")", ButtonKind::Paren)],
            &[("C_hex", ButtonKind::HexDigit), ("D", ButtonKind::HexDigit), ("E", ButtonKind::HexDigit), ("F", ButtonKind::HexDigit)],
            &[("7", ButtonKind::Digit), ("8", ButtonKind::Digit), ("9", ButtonKind::Digit), ("÷", ButtonKind::Operator)],
            &[("4", ButtonKind::Digit), ("5", ButtonKind::Digit), ("6", ButtonKind::Digit), ("×", ButtonKind::Operator)],
            &[("1", ButtonKind::Digit), ("2", ButtonKind::Digit), ("3", ButtonKind::Digit), ("−", ButtonKind::Operator)],
            &[("±", ButtonKind::Function), ("0", ButtonKind::Digit), ("+", ButtonKind::Operator), ("=", ButtonKind::Equals)],
        ];

        // Filter disabled digits based on base
        self.render_button_grid_with_base(ui, theme, rows, base);
    }

    fn render_button_grid(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        rows: &[&[(&str, ButtonKind)]],
    ) {
        let spacing = 5.0;
        let available_width = ui.available_width();
        // Match the inter-row gap to the value used in the height math and reserve
        // the leading gap egui inserts before the first row, so the grid fills the
        // remaining panel height exactly instead of spilling past the bottom edge.
        ui.spacing_mut().item_spacing.y = spacing;
        let available_height = (ui.available_height() - spacing).max(0.0);
        let num_rows = rows.len() as f32;

        for row in rows {
            let num_cols = row.len() as f32;
            let btn_width = (available_width - (num_cols - 1.0) * spacing) / num_cols;
            let btn_height = ((available_height - (num_rows - 1.0) * spacing) / num_rows).max(28.0).min(56.0);

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = spacing;
                for &(label, ref kind) in *row {
                    if *kind == ButtonKind::Spacer {
                        ui.add_space(btn_width);
                        continue;
                    }
                    let (bg, hover_bg, text_color) = self.button_colors(kind, theme);
                    let display_label = if label == "C_hex" { "C" } else { label };

                    let btn = egui::Button::new(
                        egui::RichText::new(display_label)
                            .color(text_color)
                            .size(if matches!(kind, ButtonKind::SciFunc | ButtonKind::BitOp) { 13.0 } else { 18.0 })
                    )
                    .fill(bg)
                    .corner_radius(egui::CornerRadius::same(10))
                    .min_size(egui::vec2(btn_width, btn_height));

                    let response = ui.add(btn);

                    // Paint hover effect
                    if response.hovered() {
                        ui.painter().rect_filled(
                            response.rect,
                            egui::CornerRadius::same(10),
                            hover_bg,
                        );
                        // Re-draw the label on top of the hover fill
                        ui.painter().text(
                            response.rect.center(),
                            egui::Align2::CENTER_CENTER,
                            display_label,
                            egui::FontId::proportional(if matches!(kind, ButtonKind::SciFunc | ButtonKind::BitOp) { 13.0 } else { 18.0 }),
                            text_color,
                        );
                    }

                    if response.clicked() {
                        self.handle_button_click(label, kind);
                    }
                }
            });
        }
    }

    fn render_button_grid_with_base(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        rows: &[&[(&str, ButtonKind)]],
        base: NumBase,
    ) {
        let spacing = 5.0;
        let available_width = ui.available_width();
        // See render_button_grid: pin the row gap and reserve the leading gap so
        // the (taller) 8-row programmer keypad never overflows the panel bottom.
        ui.spacing_mut().item_spacing.y = spacing;
        let available_height = (ui.available_height() - spacing).max(0.0);
        let num_rows = rows.len() as f32;

        for row in rows {
            let num_cols = row.len() as f32;
            let btn_width = (available_width - (num_cols - 1.0) * spacing) / num_cols;
            let btn_height = ((available_height - (num_rows - 1.0) * spacing) / num_rows).max(24.0).min(48.0);

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = spacing;
                for &(label, ref kind) in *row {
                    let display_label = if label == "C_hex" { "C" } else { label };

                    // Check if this digit is valid for the current base
                    let is_disabled = match kind {
                        ButtonKind::Digit => {
                            if let Some(ch) = label.chars().next() {
                                !base.is_valid_digit(ch)
                            } else {
                                false
                            }
                        }
                        ButtonKind::HexDigit => {
                            if let Some(ch) = display_label.chars().next() {
                                !base.is_valid_digit(ch)
                            } else {
                                false
                            }
                        }
                        _ => false,
                    };

                    if is_disabled {
                        let btn = egui::Button::new(
                            egui::RichText::new(display_label)
                                .color(theme.text_dim.gamma_multiply(0.3))
                                .size(15.0)
                        )
                        .fill(theme.btn.gamma_multiply(0.5))
                        .corner_radius(egui::CornerRadius::same(10))
                        .min_size(egui::vec2(btn_width, btn_height));
                        ui.add_enabled(false, btn);
                    } else {
                        let (bg, hover_bg, text_color) = self.button_colors(kind, theme);

                        let btn = egui::Button::new(
                            egui::RichText::new(display_label)
                                .color(text_color)
                                .size(if matches!(kind, ButtonKind::BitOp) { 13.0 } else { 15.0 })
                        )
                        .fill(bg)
                        .corner_radius(egui::CornerRadius::same(10))
                        .min_size(egui::vec2(btn_width, btn_height));

                        let response = ui.add(btn);

                        if response.hovered() {
                            ui.painter().rect_filled(
                                response.rect,
                                egui::CornerRadius::same(10),
                                hover_bg,
                            );
                            ui.painter().text(
                                response.rect.center(),
                                egui::Align2::CENTER_CENTER,
                                display_label,
                                egui::FontId::proportional(if matches!(kind, ButtonKind::BitOp) { 13.0 } else { 15.0 }),
                                text_color,
                            );
                        }

                        if response.clicked() {
                            self.handle_button_click(label, kind);
                        }
                    }
                }
            });
        }
    }

    fn button_colors(&self, kind: &ButtonKind, theme: &Theme) -> (egui::Color32, egui::Color32, egui::Color32) {
        match kind {
            ButtonKind::Equals => (Palette::ACCENT, Palette::ACCENT_HOVER, Palette::ACCENT_TEXT),
            ButtonKind::Operator => (theme.btn_op, theme.btn_op_hover, Palette::ACCENT),
            ButtonKind::Clear | ButtonKind::Backspace => (theme.btn_op, theme.btn_op_hover, Palette::ERROR),
            ButtonKind::SciFunc | ButtonKind::Paren => (theme.btn_op, theme.btn_op_hover, Palette::ACCENT),
            ButtonKind::BitOp => (theme.btn_op, theme.btn_op_hover, Palette::ACCENT),
            ButtonKind::Constant => (theme.btn_op, theme.btn_op_hover, Palette::ACCENT),
            ButtonKind::Variable => (theme.btn_op, theme.btn_op_hover, Palette::ACCENT),
            ButtonKind::Function => (theme.btn_op, theme.btn_op_hover, theme.text),
            ButtonKind::HexDigit => (theme.btn, theme.btn_hover, theme.text),
            _ => (theme.btn, theme.btn_hover, theme.text),
        }
    }

    fn handle_button_click(&mut self, label: &str, kind: &ButtonKind) {
        match kind {
            ButtonKind::Digit => {
                self.state.push_input(label);
            }
            ButtonKind::HexDigit => {
                let ch = if label == "C_hex" { "C" } else { label };
                self.state.push_input(&ch.to_uppercase());
            }
            ButtonKind::Operator => {
                let op = match label {
                    "÷" => "/",
                    "×" => "*",
                    "−" => "-",
                    "+" => "+",
                    "%" => "%",
                    "^" => "^",
                    _ => label,
                };
                self.state.push_input(&format!(" {} ", op));
            }
            ButtonKind::Equals => {
                self.state.evaluate();
            }
            ButtonKind::Clear => {
                self.state.clear_all();
            }
            ButtonKind::Backspace => {
                self.state.backspace();
            }
            ButtonKind::Function => {
                match label {
                    "±" => self.state.negate(),
                    _ => {}
                }
            }
            ButtonKind::SciFunc => {
                match label {
                    "sin" | "cos" | "tan" | "ln" | "log" => {
                        self.state.push_input(&format!("{}(", label));
                    }
                    "√" => self.state.push_input("sqrt("),
                    "x²" => self.state.push_input("^2"),
                    "xⁿ" => self.state.push_input("^"),
                    "n!" => self.state.push_input("fact("),
                    _ => {}
                }
            }
            ButtonKind::Paren => {
                self.state.push_input(label);
            }
            ButtonKind::Constant => {
                match label {
                    "π" => self.state.push_input("pi"),
                    "e" => self.state.push_input("e"),
                    _ => {}
                }
            }
            ButtonKind::BitOp => {
                match label {
                    "AND" => self.state.push_input(" AND "),
                    "OR"  => self.state.push_input(" OR "),
                    "XOR" => self.state.push_input(" XOR "),
                    "NOT" => self.state.push_input("NOT "),
                    "<<"  => self.state.push_input(" << "),
                    ">>"  => self.state.push_input(" >> "),
                    _ => {}
                }
            }
            ButtonKind::Variable => {
                self.state.push_input("x");
            }
            ButtonKind::Spacer => {}
        }
    }

    // ── History Sidebar ──

    fn render_history_sidebar(&mut self, ctx: &egui::Context, theme: &Theme) {
        egui::SidePanel::left("history_panel")
            .default_width(220.0)
            .min_width(180.0)
            .max_width(320.0)
            .frame(
                egui::Frame::new()
                    .fill(theme.sidebar)
                    .inner_margin(egui::Margin::same(12))
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("History")
                            .color(theme.text)
                            .size(16.0)
                            .strong(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Close button inside sidebar
                        let close_btn = egui::Button::new(
                            egui::RichText::new("✕").color(theme.text_dim).size(12.0)
                        )
                        .fill(egui::Color32::TRANSPARENT);
                        if ui.add(close_btn).clicked() {
                            self.state.toggle_history();
                        }

                        ui.add_space(8.0);

                        // Clear button
                        let clear_btn = egui::Button::new(
                            egui::RichText::new("Clear").color(Palette::ERROR).size(11.0),
                        )
                        .fill(egui::Color32::TRANSPARENT);
                        if ui.add(clear_btn).clicked() {
                            self.state.clear_history();
                        }
                    });
                });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);

                if self.state.history.is_empty() {
                    ui.label(
                        egui::RichText::new("No calculations yet")
                            .color(theme.text_dim)
                            .size(12.0)
                            .italics(),
                    );
                } else {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let mut restore_idx = None;
                        for (i, entry) in self.state.history.iter().enumerate().rev() {
                            let response = ui.add(
                                egui::Button::new({
                                    let mut job = egui::text::LayoutJob::default();
                                    job.append(
                                        &entry.expression,
                                        0.0,
                                        egui::TextFormat {
                                            font_id: egui::FontId::proportional(11.0),
                                            color: theme.text_dim,
                                            ..Default::default()
                                        },
                                    );
                                    job.append(
                                        &format!("\n= {}", entry.result),
                                        0.0,
                                        egui::TextFormat {
                                            font_id: egui::FontId::proportional(13.0),
                                            color: theme.text,
                                            ..Default::default()
                                        },
                                    );
                                    job
                                })
                                .fill(egui::Color32::TRANSPARENT)
                                .min_size(egui::vec2(ui.available_width(), 0.0)),
                            );
                            if response.clicked() {
                                restore_idx = Some(i);
                            }
                            ui.add_space(2.0);
                        }
                        if let Some(idx) = restore_idx {
                            self.state.restore_history(idx);
                        }
                    });
                }
            });
    }

    // ── Keyboard Input ──

    fn handle_keyboard(&mut self, ctx: &egui::Context) {
        // Text to place on the clipboard, deferred until after the input closure
        // releases its borrow of `ctx` (calling `ctx` methods from inside
        // `ctx.input(..)` can deadlock).
        let mut to_copy: Option<String> = None;

        ctx.input(|i| {
            for event in &i.events {
                match event {
                    egui::Event::Text(text) => {
                        for ch in text.chars() {
                            match ch {
                                '0'..='9' => self.state.push_input(&ch.to_string()),
                                '.' => self.state.push_input("."),
                                '+' => self.state.push_input(" + "),
                                '-' => self.state.push_input(" - "),
                                '*' => self.state.push_input(" * "),
                                '/' => self.state.push_input(" / "),
                                '%' => self.state.push_input(" % "),
                                '^' => self.state.push_input("^"),
                                '(' => self.state.push_input("("),
                                ')' => self.state.push_input(")"),
                                // Letters: hex digits in programmer mode; function
                                // names, constants and the graph variable `x` in
                                // scientific and graph modes (e.g. typing "sin(x").
                                c if c.is_ascii_alphabetic() => match self.state.mode {
                                    CalculatorMode::Programmer => {
                                        if c.is_ascii_hexdigit() {
                                            self.state.push_input(&c.to_uppercase().to_string());
                                        }
                                    }
                                    CalculatorMode::Scientific | CalculatorMode::Graph => {
                                        self.state.push_input(&c.to_lowercase().to_string());
                                    }
                                    CalculatorMode::Standard => {}
                                },
                                _ => {}
                            }
                        }
                    }
                    egui::Event::Key { key, pressed: true, .. } => {
                        match key {
                            // In graph mode the plot updates live, so Enter is a no-op.
                            egui::Key::Enter if self.state.mode != CalculatorMode::Graph => {
                                self.state.evaluate()
                            }
                            egui::Key::Backspace => self.state.backspace(),
                            egui::Key::Escape => self.state.clear_all(),
                            egui::Key::Delete => self.state.clear_all(),
                            _ => {}
                        }
                    }
                    // Ctrl+C / Cmd+C: copy the current result (or the expression
                    // if the result is an error message).
                    egui::Event::Copy => {
                        to_copy = Some(if self.state.has_error {
                            self.state.input.clone()
                        } else {
                            self.state.result_display.clone()
                        });
                    }
                    // Ctrl+V / Cmd+V: paste an expression into the input buffer.
                    egui::Event::Paste(text) => {
                        self.state.paste(text);
                    }
                    _ => {}
                }
            }
        });

        if let Some(text) = to_copy {
            ctx.copy_text(text);
        }
    }
}

// ── Button classification ────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum ButtonKind {
    Digit,
    HexDigit,
    Operator,
    Equals,
    Clear,
    Backspace,
    Function,
    SciFunc,
    Paren,
    Constant,
    Variable,
    BitOp,
    Spacer,
}
