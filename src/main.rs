#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use anyhow::Result;
use ashlang::r1cs::witness;
use ashlang::Config;
use ashlang::{compiler::Compiler, r1cs::constraint};
use eframe::egui;
use scalarff::{Bn128FieldElement, Curve25519FieldElement, FoiFieldElement};

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
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
    source: String, // the source code being edited
}

impl IDE {
    fn compile(&mut self) {
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
        let mut compiler: Compiler<Curve25519FieldElement> = compiler.unwrap();
        let constraints = compiler.compile_str(&self.source);
        if let Err(e) = constraints {
            self.compile_result = format!("Failed to compile ar1cs: {:?}", e);
            return;
        }
        let constraints = constraints.unwrap();
        let witness = witness::build::<Curve25519FieldElement>(&constraints);
        if let Err(e) = witness {
            self.compile_result = format!("Failed to build witness: {:?}", e);
            return;
        }
        let witness = witness.unwrap();

        if let Err(e) = witness::verify::<Curve25519FieldElement>(&constraints, witness) {
            self.compile_result = format!("Failed to solve r1cs: {:?}", e);
        } else {
            self.compile_result = format!("R1CS: built and validated witness ✅");
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
        Self {
            target: "r1cs".to_string(),
            source: "let x = 0\nlet y = 1\nlet _ = x + y\n".to_string(),
            compile_result: "".to_string(),
            field: "curve25519".to_string(),
        }
    }
}

impl eframe::App for IDE {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.compile();
            }
            ui.horizontal(|ui| ui.text_edit_multiline(&mut self.source));
            ui.horizontal(|ui| ui.label(&self.compile_result));
            // ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            // if ui.button("Increment").clicked() {
            //     self.age += 1;
            // }
            // ui.label(format!("Hello '{}', age {}", self.name, self.age));

            // ui.image(egui::include_image!("../static/clouds.png"));
        });
    }
}
