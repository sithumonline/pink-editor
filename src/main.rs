use eframe::egui;
use egui::{Key, KeyboardShortcut, Modifiers};
use std::path::{Path, PathBuf};

fn language_from_path(path: Option<&Path>) -> &'static str {
    let Some(ext) = path.and_then(|p| p.extension()).and_then(|e| e.to_str()) else {
        return "";
    };
    match ext {
        "rs" => "rs",
        "go" => "go",
        "py" => "py",
        "js" | "mjs" | "cjs" => "js",
        "ts" | "tsx" => "ts",
        "md" | "markdown" => "md",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "json" => "json",
        "sh" | "bash" | "zsh" => "sh",
        "c" | "h" => "c",
        "cpp" | "cxx" | "cc" | "hpp" | "hxx" => "cpp",
        "html" | "htm" => "html",
        "css" => "css",
        "xml" => "xml",
        "sql" => "sql",
        _ => "",
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "pink-editor",
        options,
        Box::new(|_cc| Ok(Box::new(PinkEditor::default()))),
    )
}

#[derive(Copy, Clone)]
enum PendingAction {
    CloseTab(usize),
}

enum Choice {
    Save,
    Discard,
    Cancel,
}

#[derive(Default)]
struct Buffer {
    text: String,
    path: Option<PathBuf>,
    dirty: bool,
}

impl Buffer {
    fn read_from(path: PathBuf) -> std::io::Result<Self> {
        let text = std::fs::read_to_string(&path)?;
        Ok(Self {
            text,
            path: Some(path),
            dirty: false,
        })
    }

    fn write_to(&mut self, path: PathBuf) -> std::io::Result<()> {
        std::fs::write(&path, &self.text)?;
        self.path = Some(path);
        self.dirty = false;
        Ok(())
    }

    fn tab_label(&self) -> String {
        let name = self
            .path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_else(|| "untitled".to_string());
        if self.dirty {
            format!("{} *", name)
        } else {
            name
        }
    }
}

struct PinkEditor {
    buffers: Vec<Buffer>,
    active: usize,
    error: Option<String>,
    pending: Option<PendingAction>,
}

impl Default for PinkEditor {
    fn default() -> Self {
        Self {
            buffers: vec![Buffer::default()],
            active: 0,
            error: None,
            pending: None,
        }
    }
}

impl PinkEditor {
    fn new_tab(&mut self) {
        self.buffers.push(Buffer::default());
        self.active = self.buffers.len() - 1;
    }

    fn open_file(&mut self) {
        let Some(path) = rfd::FileDialog::new().pick_file() else {
            return;
        };
        match Buffer::read_from(path.clone()) {
            Ok(buffer) => {
                self.buffers.push(buffer);
                self.active = self.buffers.len() - 1;
                self.error = None;
            }
            Err(e) => {
                self.error = Some(format!("failed to open {}: {}", path.display(), e));
            }
        }
    }

    fn save(&mut self) {
        let index = self.active;
        let path = self.buffers.get(index).and_then(|b| b.path.clone());
        match path {
            Some(path) => self.write_to_index(index, path),
            None => self.save_as(),
        }
    }

    fn save_as(&mut self) {
        let Some(path) = rfd::FileDialog::new().save_file() else {
            return;
        };
        self.write_to_index(self.active, path);
    }

    fn write_to_index(&mut self, index: usize, path: PathBuf) {
        let Some(buffer) = self.buffers.get_mut(index) else {
            return;
        };
        match buffer.write_to(path.clone()) {
            Ok(()) => self.error = None,
            Err(e) => {
                self.error = Some(format!("failed to save {}: {}", path.display(), e));
            }
        }
    }

    fn request_close_tab(&mut self, index: usize) {
        let dirty = self.buffers.get(index).map(|b| b.dirty).unwrap_or(false);
        if dirty {
            self.pending = Some(PendingAction::CloseTab(index));
        } else {
            self.close_tab(index);
        }
    }

    fn close_tab(&mut self, index: usize) {
        if index >= self.buffers.len() {
            return;
        }
        self.buffers.remove(index);
        if self.buffers.is_empty() {
            self.buffers.push(Buffer::default());
            self.active = 0;
        } else if self.active >= self.buffers.len() {
            self.active = self.buffers.len() - 1;
        } else if self.active > index {
            self.active -= 1;
        }
    }

    fn show_unsave_modal(&mut self, ctx: &egui::Context) {
        let Some(PendingAction::CloseTab(index)) = self.pending else {
            return;
        };
        let mut choice: Option<Choice> = None;

        egui::Window::new("Unsaved changes")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                let name = self
                    .buffers
                    .get(index)
                    .map(|b| b.tab_label())
                    .unwrap_or_default();
                ui.label(format!("{} has unsaved changes.", name));
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        choice = Some(Choice::Save);
                    }
                    if ui.button("Discard").clicked() {
                        choice = Some(Choice::Discard);
                    }
                    if ui.button("Cancel").clicked() {
                        choice = Some(Choice::Cancel);
                    }
                });
            });

        let Some(choice) = choice else { return };
        self.pending = None;

        match choice {
            Choice::Save => {
                let path = self.buffers.get(index).and_then(|b| b.path.clone());
                match path {
                    Some(p) => self.write_to_index(index, p),
                    None => {
                        if let Some(p) = rfd::FileDialog::new().save_file() {
                            self.write_to_index(index, p);
                        }
                    }
                }
                let still_dirty = self.buffers.get(index).map(|b| b.dirty).unwrap_or(false);
                if !still_dirty {
                    self.close_tab(index);
                }
            }
            Choice::Discard => self.close_tab(index),
            Choice::Cancel => {}
        }
    }
}

impl eframe::App for PinkEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let new_sc = KeyboardShortcut::new(Modifiers::COMMAND, Key::N);
        let open_sc = KeyboardShortcut::new(Modifiers::COMMAND, Key::O);
        let save_sc = KeyboardShortcut::new(Modifiers::COMMAND, Key::S);
        let save_as_sc = KeyboardShortcut::new(Modifiers::COMMAND | Modifiers::SHIFT, Key::S);
        let close_sc = KeyboardShortcut::new(Modifiers::COMMAND, Key::W);

        if ctx.input_mut(|i| i.consume_shortcut(&new_sc)) {
            self.new_tab();
        }
        if ctx.input_mut(|i| i.consume_shortcut(&open_sc)) {
            self.open_file();
        }
        if ctx.input_mut(|i| i.consume_shortcut(&save_sc)) {
            self.save();
        }
        if ctx.input_mut(|i| i.consume_shortcut(&save_as_sc)) {
            self.save_as();
        }
        if ctx.input_mut(|i| i.consume_shortcut(&close_sc)) {
            self.request_close_tab(self.active);
        }

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui
                        .add(egui::Button::new("New").shortcut_text(ctx.format_shortcut(&new_sc)))
                        .clicked()
                    {
                        self.new_tab();
                        ui.close_menu();
                    }
                    if ui
                        .add(
                            egui::Button::new("Open...")
                                .shortcut_text(ctx.format_shortcut(&open_sc)),
                        )
                        .clicked()
                    {
                        self.open_file();
                        ui.close_menu();
                    }
                    if ui
                        .add(egui::Button::new("Save").shortcut_text(ctx.format_shortcut(&save_sc)))
                        .clicked()
                    {
                        self.save();
                        ui.close_menu();
                    }
                    if ui
                        .add(
                            egui::Button::new("Save As...")
                                .shortcut_text(ctx.format_shortcut(&save_as_sc)),
                        )
                        .clicked()
                    {
                        self.save_as();
                        ui.close_menu();
                    }
                    if ui
                        .add(
                            egui::Button::new("Close Tab")
                                .shortcut_text(ctx.format_shortcut(&close_sc)),
                        )
                        .clicked()
                    {
                        self.request_close_tab(self.active);
                        ui.close_menu();
                    }
                });
            });
        });

        egui::TopBottomPanel::top("tab_bar").show(ctx, |ui| {
            egui::ScrollArea::horizontal()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let mut to_close: Option<usize> = None;
                        let mut to_select: Option<usize> = None;
                        let mut close_others_of: Option<usize> = None;

                        for (i, buffer) in self.buffers.iter().enumerate() {
                            if i > 0 {
                                ui.separator();
                            }
                            let selected = i == self.active;
                            let label = buffer.tab_label();
                            let tab_count = self.buffers.len();

                            ui.spacing_mut().item_spacing.x = 2.0;

                            let tab_resp = ui.selectable_label(selected, &label);
                            if tab_resp.clicked() {
                                to_select = Some(i);
                            }
                            if tab_resp.middle_clicked() {
                                to_close = Some(i);
                            }
                            tab_resp.context_menu(|ui| {
                                if ui.button("Close").clicked() {
                                    to_close = Some(i);
                                    ui.close_menu();
                                }
                                if ui
                                    .add_enabled(
                                        tab_count > 1,
                                        egui::Button::new("Close Others"),
                                    )
                                    .clicked()
                                {
                                    close_others_of = Some(i);
                                    ui.close_menu();
                                }
                            });

                            let close_resp = ui.add(
                                egui::Button::new(egui::RichText::new("×").weak())
                                    .small()
                                    .frame(false),
                            );
                            if close_resp.clicked() {
                                to_close = Some(i);
                            }
                        }

                        if let Some(i) = to_select {
                            self.active = i;
                        }
                        if let Some(keep) = close_others_of {
                            for i in (0..self.buffers.len()).rev() {
                                if i != keep {
                                    self.request_close_tab(i);
                                }
                            }
                        } else if let Some(i) = to_close {
                            self.request_close_tab(i);
                        }
                    });
                });
        });

        if let Some(err) = &self.error {
            egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
                ui.colored_label(egui::Color32::RED, err);
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let Some(buffer) = self.buffers.get_mut(self.active) else {
                return;
            };
            const HIGHLIGHT_MAX_BYTES: usize = 500_000;
            let language = if buffer.text.len() > HIGHLIGHT_MAX_BYTES {
                ""
            } else {
                language_from_path(buffer.path.as_deref())
            };
            let theme = egui_extras::syntax_highlighting::CodeTheme::from_style(ui.style());

            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.horizontal_top(|ui| {
                        let line_count = buffer.text.matches('\n').count() + 1;
                        let width = line_count.to_string().len().max(3);
                        let gutter: String = (1..=line_count)
                            .map(|n| format!("{:>width$}", n, width = width))
                            .collect::<Vec<_>>()
                            .join("\n");
                        ui.label(
                            egui::RichText::new(gutter)
                                .monospace()
                                .color(egui::Color32::GRAY),
                        );

                        let mut layouter = |ui: &egui::Ui, text: &str, _wrap_width: f32| {
                            let job = egui_extras::syntax_highlighting::highlight(
                                ui.ctx(),
                                ui.style(),
                                &theme,
                                text,
                                language,
                            );
                            ui.fonts(|f| f.layout_job(job))
                        };

                        let response = ui.add_sized(
                            ui.available_size(),
                            egui::TextEdit::multiline(&mut buffer.text)
                                .font(egui::TextStyle::Monospace)
                                .code_editor()
                                .desired_width(f32::INFINITY)
                                .layouter(&mut layouter),
                        );
                        if response.changed() {
                            buffer.dirty = true;
                        }
                    });
                });
        });

        self.show_unsave_modal(ctx);
    }
}
