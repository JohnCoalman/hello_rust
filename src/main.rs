use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    // --- 1. VERSION NATIVE (.exe) ---
    #[cfg(not(target_arch = "wasm32"))]
    {
        let options = eframe::NativeOptions::default();
        eframe::run_native(
            "Mon Appli Egui Rust",
            options,
            Box::new(|_cc| Ok(Box::<MyApp>::default())),
        )
    }

    // --- 2. VERSION WEBASSEMBLY (Navigateur / GitHub Pages) ---
    #[cfg(target_arch = "wasm32")]
    {
        use eframe::wasm_bindgen::JsCast as _;

        wasm_bindgen_futures::spawn_local(async {
            let document = web_sys::window()
                .unwrap()
                .document()
                .unwrap();
            let canvas = document
                .get_element_by_id("the_canvas_id")
                .unwrap()
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .unwrap();

            let web_options = eframe::WebOptions::default();

            eframe::WebRunner::new()
                .start(
                    canvas,
                    web_options,
                    Box::new(|_cc| Ok(Box::<MyApp>::default())),
                )
                .await
                .expect("échec du lancement de eframe sur le web");
        });
        Ok(())
    }
}

// Définition de l'état de notre application
struct MyApp {
    name: String,
    age: u32,
    counter: i32,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "John".to_owned(),
            age: 30,
            counter: 0,
        }
    }
}

// Implémentation de l'interface graphique avec egui
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🦀 Mon Application Egui en Rust");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Votre nom : ");
                ui.text_edit_singleline(&mut self.name);
            });

            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("Âge"));

            if ui.button("Cliquez-moi !").clicked() {
                self.counter += 1;
            }

            ui.add_space(10.0);
            ui.colored_label(
                egui::Color32::LIGHT_GREEN,
                format!("Bonjour {}, vous avez {} ans et vous avez cliqué {} fois.", self.name, self.age, self.counter)
            );
        });
    }
}