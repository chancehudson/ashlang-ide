#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use ashlang::compiler::Compiler;
use ashlang::r1cs::witness;
use ashlang::Config;
use eframe::egui;
use scalarff::Bn128FieldElement;
use scalarff::Curve25519FieldElement;
use scalarff::FieldElement;
use scalarff::FoiFieldElement;

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        // set a very large max height to fill the screen
        // TODO: determine how this looks on Windows/Linux
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 2_f32.powf(50.0)]),
        ..Default::default()
    };
    eframe::run_native(
        "ashlang IDE",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::<IDE>::default())
        }),
    )
}

struct IDE {
    target: String, // "tasm" or "r1cs"
    field: String,  // "curve25519" or "foi" or "alt_bn128"
    compile_result: String,
    compile_output: String,
    source: String, // the source code being edited
}

impl IDE {
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
        let compiler = Compiler::new(&Config {
            include_paths: vec![],
            verbosity: 0,
            inputs: vec![],
            secret_inputs: vec![],
            target: self.target.clone(),
            extension_priorities: vec!["ash".to_string()],
            entry_fn: "entry".to_string(),
            field: self.field.clone(),
        });
        if let Err(e) = compiler {
            self.compile_result = format!("Failed to create compiler: {:?}", e);
            return;
        }
        let mut compiler: Compiler<T> = compiler.unwrap();
        let program = compiler.compile_str(&self.source);
        if let Err(e) = program {
            self.compile_result = format!("Failed to compile ar1cs: {:?}", e);
            self.compile_output = "".to_string();
            return;
        }
        let program = program.unwrap();
        match self.target.as_str() {
            "r1cs" => {
                // build a witness and validate it if we're compiling for r1cs
                let witness = witness::build::<T>(&program);
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

impl Default for IDE {
    fn default() -> Self {
        let mut s = Self {
            target: "r1cs".to_string(),
            source: "let x = 0
let y = 1
let _ = x + y
"
            .to_string(),
            compile_result: "".to_string(),
            compile_output: "".to_string(),
            field: "curve25519".to_string(),
        };
        s.compile_generic();
        s
    }
}

impl eframe::App for IDE {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        for i in 0..10 {
                            ui.label(&format!("ahfjksahfsakhf {i}"));
                        }
                    });
                    // TODO: use layouter to implement syntax highlighting
                    ui.vertical(|ui| {
                        let editor = egui::TextEdit::multiline(&mut self.source);
                        let size = egui::Vec2::new(
                            ui.available_width(),
                            ctx.screen_rect().height() - 200_f32,
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
            });
    });
}

fn render_build_info(ide: &mut IDE, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical()
        .id_source("compile_output_scroll")
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.heading("Compile status");
                ui.colored_label(egui::Color32::WHITE, &ide.compile_result);
            });
            ui.add(egui::Separator::default());
            ui.vertical(|ui| {
                ui.heading("Compile output");
                ui.colored_label(egui::Color32::WHITE, &ide.compile_output);
            });
        });
}
