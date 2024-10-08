use std::collections::HashMap;

use ashlang::compiler::Compiler;
use ashlang::r1cs::witness;
use ashlang::Config;
use eframe::egui;
use scalarff::Bn128FieldElement;
use scalarff::Curve25519FieldElement;
use scalarff::FieldElement;
use scalarff::FoiFieldElement;
use strip_ansi_escapes;

pub struct IDE {
    target: String, // "tasm" or "r1cs"
    field: String,  // "curve25519" or "foi" or "alt_bn128"
    compile_result: String,
    compile_output: String,
    source: String, // the source code being edited
    fs: HashMap<String, String>,
    active_file: String,
}

impl Default for IDE {
    fn default() -> Self {
        let fs = super::fs::init();
        if let Err(e) = fs {
            panic!("Failed to initialize filesystem: {:?}", e);
        }
        let active_file = "entry.ash".to_string();
        let fs = fs.unwrap();
        let source = fs.get(&active_file).unwrap().clone();
        let mut s = Self {
            active_file,
            fs,
            target: "r1cs".to_string(),
            source,
            compile_result: "".to_string(),
            compile_output: "".to_string(),
            field: "oxfoi".to_string(),
        };
        s.compile_generic();
        s
    }
}

impl IDE {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }

        Default::default()
    }

    fn compile_generic(&mut self) {
        if self.target == "tasm" && self.field != "oxfoi" {
            self.compile_result = "tasm target must be compiled to the oxfoi field".to_string();
            self.compile_output = "".to_string();
            return;
        }
        // otherwise we're compiling for r1cs
        match self.field.as_str() {
            "oxfoi" => self.compile::<FoiFieldElement>(),
            "curve25519" => self.compile::<Curve25519FieldElement>(),
            "alt_bn128" => self.compile::<Bn128FieldElement>(),
            _ => unreachable!(),
        }
    }

    fn compile<T: FieldElement>(&mut self) {
        self.fs
            .insert(self.active_file.clone(), self.source.clone());
        let extension_priorities = if self.target == "tasm" {
            vec!["ash".to_string(), "tasm".to_string()]
        } else if self.target == "r1cs" {
            vec!["ash".to_string(), "ar1cs".to_string()]
        } else {
            vec!["ash".to_string()]
        };
        let compiler = Compiler::new(&Config {
            include_paths: vec![],
            verbosity: 0,
            inputs: vec![],
            secret_inputs: vec![],
            target: self.target.clone(),
            extension_priorities,
            entry_fn: "entry".to_string(),
            field: self.field.clone(),
        });
        if let Err(e) = compiler {
            self.compile_result = format!("Failed to create compiler: {:?}", e);
            return;
        }
        let mut compiler: Compiler<T> = compiler.unwrap();
        compiler.include_vfs(&self.fs).unwrap();
        let program = compiler.compile("entry");
        if let Err(e) = program {
            self.compile_result = format!(
                "Failed to compile to {}: {}",
                self.target,
                strip_ansi_escapes::strip_str(&e.to_string())
            );
            self.compile_output = "".to_string();
            return;
        }
        let program = program.unwrap();
        match self.target.as_str() {
            "r1cs" => {
                // build a witness and validate it if we're compiling for r1cs
                let witness = witness::build::<T>(&program, vec![]);
                if let Err(e) = witness {
                    self.compile_result = format!("Failed to build witness: {:?}", e);
                    self.compile_output = "".to_string();
                    return;
                }
                let witness = witness.unwrap();

                if let Err(e) = witness::verify::<T>(&program, witness) {
                    self.compile_result = format!("Failed to solve r1cs: {:?}", e);
                    self.compile_output = "".to_string();
                } else {
                    self.compile_result = format!(
                        "Compiling for field {}...\nR1CS: built and validated witness ✅",
                        self.field
                    );
                    self.compile_output = program.to_string();
                }
            }
            "tasm" => {
                self.compile_result = format!("Compiled program to tasm source");
                self.compile_output = program.to_string();
            }
            _ => unreachable!(),
        }
    }
}

impl eframe::App for IDE {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        for (filename, _) in &self.fs {
                            let label = if self.active_file == filename.to_string() {
                                let res = ui.label(&format!("> {filename}"));
                                res.on_hover_cursor(egui::CursorIcon::PointingHand)
                            } else {
                                let res = ui.label(filename);
                                res.on_hover_cursor(egui::CursorIcon::PointingHand)
                            };
                            if label.clicked() {
                                self.active_file = filename.clone();
                                self.source = self.fs.get(&self.active_file).unwrap().clone();
                            }
                        }
                    });
                    // TODO: use layouter to implement syntax highlighting
                    ui.vertical(|ui| {
                        let editor = egui::TextEdit::multiline(&mut self.source).lock_focus(true);
                        let size = egui::Vec2::new(
                            ui.available_width(),
                            ctx.screen_rect().height() - 400_f32,
                        );
                        let editor = ui.add_sized(size, editor);
                        if editor.changed() {
                            self.compile_generic();
                        }
                        render_build_options(self, ui);
                    });
                });
                // render_build_options(self, ui);
                render_build_info(self, ui);
            });
        });
        egui::TopBottomPanel::bottom("bottom").show(ctx, |ui| {
            ui.horizontal(|ui| {
                egui::widgets::global_dark_light_mode_buttons(ui);
                if ui.ui_contains_pointer() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
            });
        });
    }
}

fn render_build_options(ide: &mut IDE, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        egui::ComboBox::new("scalar_field_selector", "")
            .selected_text(format!("Scalar Field: {}", ide.field))
            .show_ui(ui, |ui| {
                if ui
                    .selectable_value(&mut ide.field, "curve25519".to_string(), "curve25519")
                    .changed()
                    || ui
                        .selectable_value(&mut ide.field, "alt_bn128".to_string(), "alt_bn128")
                        .changed()
                    || ui
                        .selectable_value(&mut ide.field, "oxfoi".to_string(), "oxfoi")
                        .changed()
                {
                    ide.compile_generic();
                }
                if ui.ui_contains_pointer() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
            });
        egui::ComboBox::new("target_selector", "")
            .selected_text(format!("Target: {}", ide.target))
            .show_ui(ui, |ui| {
                if ui
                    .selectable_value(&mut ide.target, "tasm".to_string(), "tasm")
                    .changed()
                    || ui
                        .selectable_value(&mut ide.target, "r1cs".to_string(), "r1cs")
                        .changed()
                {
                    ide.compile_generic();
                }
                if ui.ui_contains_pointer() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
            });
        if ui.ui_contains_pointer() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
    });
}

fn render_build_info(ide: &mut IDE, ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        ui.heading("Compile status");
        egui::ScrollArea::vertical()
            .id_source("compile_status_scroll")
            .auto_shrink([false, true])
            .show(ui, |ui| {
                ui.label(&ide.compile_result);
            });
    });
    ui.add(egui::Separator::default());
    ui.vertical(|ui| {
        ui.heading("Compile output");
        egui::ScrollArea::vertical()
            .id_source("compile_output_scroll")
            .auto_shrink([false, true])
            .show(ui, |ui| {
                ui.label(&ide.compile_output);
            });
    });
}
