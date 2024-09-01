#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use std::sync::Arc;
use std::sync::RwLock;
use std::thread;

use anyhow::Result;
use ashlang::r1cs::witness;
use ashlang::Config;
use ashlang::{compiler::Compiler, r1cs::constraint};
use eframe::egui;
use scalarff::FieldElement;
use scalarff::{Bn128FieldElement, Curve25519FieldElement, FoiFieldElement};

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 800.0]),
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
        match self.field.as_str() {
            "oxfoi" => {
                self.compile::<FoiFieldElement>();
            }
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
        let constraints = compiler.compile_str(&self.source);
        if let Err(e) = constraints {
            self.compile_result = format!("Failed to compile ar1cs: {:?}", e);
            self.compile_output = "".to_string();
            return;
        }
        let constraints = constraints.unwrap();
        let witness = witness::build::<T>(&constraints);
        if let Err(e) = witness {
            self.compile_result = format!("Failed to build witness: {:?}", e);
            self.compile_output = "".to_string();
            return;
        }
        let witness = witness.unwrap();

        if let Err(e) = witness::verify::<T>(&constraints, witness) {
            self.compile_result = format!("Failed to solve r1cs: {:?}", e);
            self.compile_output = "".to_string();
        } else {
            self.compile_result = format!(
                "Compiling for field {}...\nR1CS: built and validated witness ✅",
                self.field
            );
            self.compile_output = constraints.to_string();
        }
        // // produce a tiny instance
        // let config = transform_r1cs(&out);
        // let spartan_proof = prove(config);

        // let valid = verify(spartan_proof);
        // assert!(valid);
        // println!("proof verification successful!");
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
            // ui.horizontal(|ui| {
            let editor = egui::TextEdit::multiline(&mut self.source);
            // let editor = ui.add_sized(ui.available_size(), editor);
            let editor = ui.add(editor);
            if editor.changed() {
                self.compile_generic();
            }
            // });
            egui::ComboBox::from_label("")
                .selected_text(format!("Scalar Field: {}", self.field))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.field, "curve25519".to_string(), "curve25519");
                    ui.selectable_value(&mut self.field, "alt_bn128".to_string(), "alt_bn128");
                    ui.selectable_value(&mut self.field, "oxfoi".to_string(), "oxfoi");
                });
            ui.horizontal(|ui| {
                ui.label(&self.compile_result);
                ui.add(egui::Separator::default());
                ui.label(&self.compile_output);
            });

            // ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            // if ui.button("Increment").clicked() {
            //     self.age += 1;
            // }
            // ui.label(format!("Hello '{}', age {}", self.name, self.age));

            // ui.image(egui::include_image!("../static/clouds.png"));
        });
    }
}
