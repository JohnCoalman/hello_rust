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
    updating: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "John".to_owned(),
            age: 30,
            counter: 0,
            current_tab: SelectedTab::Application,
            current_version: format!("v{}", env!("CARGO_PKG_VERSION")), // 👈 Récupéré dynamiquement du Cargo.toml
            latest_version: None,
            update_available: false,
            checking: false,
            updating: false,
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

// Fonction globale pour télécharger le .exe et appliquer la mise à jour sous Windows
fn download_and_apply_update() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://github.com/JohnCoalman/hello_rust/releases/latest/download/hello_rust.exe";
    
    // 1. Télécharger le nouveau .exe dans un fichier temporaire
    let client = reqwest::blocking::Client::builder().user_agent("hello_rust-app").build()?;
    let mut response = client.get(url).send()?;
    
    let mut dest = std::fs::File::create("update.exe")?;
    std::io::copy(&mut response, &mut dest)?;

    // 2. Créer un script batch pour remplacer l'ancien exe et relancer l'app
    let batch_content = r#"
    @echo off
    timeout /t 2 /nobreak > nul
    move /y update.exe hello_rust.exe
    start hello_rust.exe
    del %0
    "#;

    std::fs::write("updater.bat", batch_content)?;

    // 3. Lancer le script et quitter l'application
    std::process::Command::new("cmd")
        .args(&["/C", "updater.bat"])
        .spawn()?;

    std::process::exit(0);
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
                }
                SelectedTab::MiseAJour => {
                    ui.heading("🔄 Gestion des mises à jour");
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
                        if self.updating {
                            ui.spinner();
                            ui.label("Téléchargement et installation en cours...");
                        } else if ui.button("📥 Télécharger et installer automatiquement").clicked() {
                            self.updating = true;
                            // Lancer le téléchargement dans un thread pour ne pas bloquer l'UI
                            std::thread::spawn(|| {
                                if let Err(e) = download_and_apply_update() {
                                    eprintln!("Erreur de mise à jour : {}", e);
                                }
                            });
                        }
                    } else if self.latest_version.is_some() {
                        ui.colored_label(egui::Color32::LIGHT_GREEN, "✅ Votre application est déjà à jour !");
                    }
                }
            }
        });
    }
}