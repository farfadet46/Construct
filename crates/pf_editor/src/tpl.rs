//! Modèles de jeux intégrés (projets complets).
use pf_core::Json;

pub const LIST: [(&str, &str, &str); 14] = [
    ("blank", "Projet vierge", "Un niveau vide pour partir de zéro."),
    ("sample", "Exemple complet", "« Les aventures de Pixel » : 2 niveaux, menus, ennemis, sauvegarde… pour apprendre."),
    ("platformer", "Plateforme", "Un héros qui court et saute, des pièces et un drapeau. La base la plus simple."),
    ("topdown", "Vue de dessus", "Aventure en 8 directions : clé, porte, gemmes et ennemis."),
    ("shmup", "Shoot’em up", "Vaisseau, tirs, vagues d’ennemis et fond étoilé qui défile."),
    ("physics", "Physique", "Bac à sable : caisses empilées, ballons, rocher et pendules à pousser."),
    ("breakout", "Casse-briques", "Raquette, balle qui rebondit et briques à détruire."),
    ("flappy", "Vol plané", "Un oiseau, des tuyaux, un record sauvegardé. Une seule touche pour jouer."),
    ("runner", "Course infinie", "Le décor défile de plus en plus vite : sautez les obstacles, ramassez les pièces."),
    ("td", "Défense du château", "Tower defense : vagues d’ennemis sur un chemin, tours à construire avec l’or gagné."),
    ("city", "Pixel City", "Ville vue de dessus façon premiers GTA : voitures, missions, passants et police."),
    ("rally", "Rallye", "Course vue de l’arrière (pseudo-3D) : virages, bosses, sapins, 5 rivaux et 3 tours."),
    ("story", "Histoire interactive", "Un récit à choix sur 4 niveaux : dialogues, embranchements, deux fins."),
    ("memory", "Memory", "16 cartes à retourner : tableaux, sélection d’objets et animation, à la souris."),
];

fn src(id: &str) -> &'static str {
    match id {
        "sample" => include_str!("../../../fixtures/sample.json"),
        "platformer" => include_str!("../../../fixtures/platformer.json"),
        "topdown" => include_str!("../../../fixtures/topdown.json"),
        "shmup" => include_str!("../../../fixtures/shmup.json"),
        "physics" => include_str!("../../../fixtures/physics.json"),
        "breakout" => include_str!("../../../fixtures/breakout.json"),
        "flappy" => include_str!("../../../fixtures/flappy.json"),
        "runner" => include_str!("../../../fixtures/runner.json"),
        "td" => include_str!("../../../fixtures/td.json"),
        "city" => include_str!("../../../fixtures/city.json"),
        "rally" => include_str!("../../../fixtures/rally.json"),
        "story" => include_str!("../../../fixtures/story.json"),
        "memory" => include_str!("../../../fixtures/memory.json"),
        _ => include_str!("../../../fixtures/blank.json"),
    }
}

pub fn make(id: &str) -> Json {
    serde_json::from_str(src(id)).unwrap_or(Json::Null)
}
