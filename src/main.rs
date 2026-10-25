#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use serde::Deserialize;

// Structure pour lire le numéro de version renvoyé par GitHub
#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
}

// Enumération pour gérer nos onglets
#[derive(PartialEq)]
enum SelectedTab {
    Application,
    MiseAJour,
}

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
    current_tab: SelectedTab,
    
    // Champs pour l'auto-update
    current_version: String,
    latest_version: Option<String>,
    update_available: bool,
    checking: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "John".to_owned(),
            age: 30,
            counter: 0,
            current_tab: SelectedTab::Application,
            current_version: format!("v{}", env!("CARGO_PKG_VERSION")),
            latest_version: None,
            update_available: false,
            checking: false,
        }
    }
}

impl MyApp {
    // Fonction pour interroger l'API GitHub
    fn check_for_updates(&mut self) {
        self.checking = true;
        let url = "https://api.github.com/repos/JohnCoalman/hello_rust/releases/latest";
        
        let client = reqwest::blocking::Client::builder()
            .user_agent("hello_rust-app")
            .build();

        if let Ok(client) = client {
            if let Ok(response) = client.get(url).send() {
                if let Ok(release) = response.json::<GitHubRelease>() {
                    if release.tag_name != self.current_version {
                        self.update_available = true;
                    }
                    self.latest_version = Some(release.tag_name);
                }
            }
        }
        self.checking = false;
    }
}

// Fonction pour déclencher la mise à jour officielle via le script cargo-dist (PowerShell sous Windows)
fn trigger_cargo_dist_update() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")]
    {
        // Commande officielle recommandée par cargo-dist pour installer/mettre à jour via PowerShell
        let ps_command = "irm https://github.com/JohnCoalman/hello_rust/releases/latest/download/hello_rust-installer.ps1 | iex";
        std::process::Command::new("powershell")
            .args(&["-NoProfile", "-Command", ps_command])
            .spawn()?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Équivalent shell pour Linux/macOS si besoin
        let sh_command = "curl --proto '=https' --tlsv1.2 -sSf https://github.com/JohnCoalman/hello_rust/releases/latest/download/installer.sh | sh";
        std::process::Command::new("sh")
            .args(&["-c", sh_command])
            .spawn()?;
    }

    Ok(())
}

// Implémentation de l'interface graphique avec egui
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Barre d'onglets supérieure
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.current_tab, SelectedTab::Application, "🏠 Application");
                ui.selectable_value(&mut self.current_tab, SelectedTab::MiseAJour, "🔄 Mises à jour");
            });
        });

        // Contenu principal selon l'onglet choisi
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_tab {
                SelectedTab::Application => {
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
                    ui.label(format!("Version actuelle de l'application : {}", self.current_version));
                }
                SelectedTab::MiseAJour => {
                    ui.heading("🔄 Gestion des mises à jour (Cargo-Dist)");
                    ui.separator();

                    ui.label(format!("Version actuelle de l'application : {}", self.current_version));
                    
                    ui.add_space(10.0);

                    if ui.button("Vérifier les nouvelles versions").clicked() {
                        self.check_for_updates();
                    }

                    if self.checking {
                        ui.add_space(5.0);
                        ui.spinner();
                        ui.label("Vérification auprès de GitHub en cours...");
                    }

                    ui.add_space(10.0);

                    if self.update_available {
                        ui.colored_label(
                            egui::Color32::YELLOW,
                            format!("✨ Une nouvelle version est disponible : {} !", self.latest_version.as_deref().unwrap_or(""))
                        );

                        ui.add_space(5.0);
                        if ui.button("📥 Appliquer la mise à jour officielle").clicked() {
                            if let Err(e) = trigger_cargo_dist_update() {
                                eprintln!("Erreur lors du lancement de la mise à jour : {}", e);
                            } else {
                                // Quitter l'application pour laisser le script installer la nouvelle version propre
                                std::process::exit(0);
                            }
                        }
                    } else if self.latest_version.is_some() {
                        ui.colored_label(egui::Color32::LIGHT_GREEN, "✅ Votre application est déjà à jour !");
                    }

                    ui.add_space(20.0);
                    ui.separator();
                    if ui.button("🌐 Ouvrir la page des Releases GitHub").clicked() {
                        let _ = open::that("https://github.com/JohnCoalman/hello_rust/releases");
                    }
                }
            }
        });
    }
}