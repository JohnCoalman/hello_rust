fn main() {
    // Récupère la fenêtre du navigateur et le document HTML
    let window = web_sys::window().expect("Pas de fenêtre globale trouvée");
    let document = window.document().expect("Pas de document trouvé");

    // Crée un nouvel élément paragraphe <p>
    let val = document.create_element("p").unwrap();
    val.set_inner_html("🦀 Message injecté dynamiquement depuis mon fichier main.rs en Rust !");

    // Ajoute ce paragraphe dans le <body> de la page HTML
    let body = document.body().expect("Pas de body trouvé");
    body.append_child(&val).unwrap();
}