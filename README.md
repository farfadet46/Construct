# PlatForge Studio — version native (Rust)

Éditeur de jeux 2D façon Construct, réécrit en Rust : une vraie application de bureau
(Windows, macOS, Linux), sans navigateur. Les projets sont **les mêmes fichiers `.json`**
que la version HTML : on peut passer de l'une à l'autre.

## Lancer

```
platforge                 # ouvre l'éditeur (choix d'un modèle)
platforge mon-jeu.json    # ouvre un projet
platforge --play jeu.json # lance directement le jeu (mode joueur)
```

## Construire depuis les sources

1. Installer Rust : https://rustup.rs
2. Linux uniquement : `sudo apt install libasound2-dev libxkbcommon-dev libgtk-3-dev`
3. Dans ce dossier : `cargo build --release -p pf_editor`
4. L'application est dans `target/release/platforge` (`platforge.exe` sous Windows).

Le fichier `.github/workflows/build.yml` construit automatiquement les trois versions
(Windows, macOS, Linux) si le dossier est placé sur GitHub.

## Organisation

| Dossier | Rôle |
|---|---|
| `crates/pf_core` | valeurs (mêmes conversions que JavaScript), langage d'expressions, catalogue des conditions / actions / comportements |
| `crates/pf_runtime` | le moteur de jeu complet : événements, comportements, physique, tuiles, Route 3D, rendu (tiny-skia), sons synthétisés |
| `crates/pf_editor` | l'application (egui) : projet, vue du niveau, feuille d'événements, propriétés, test du jeu, sons (rodio) |
| `fixtures/` | les 14 modèles de jeux, utilisés aussi pour les tests de parité |
| `tools/` | comparaison automatique avec la version HTML (`parity.sh`) |

## Vérifier que le moteur se comporte comme la version HTML

```
cargo build --release -p pf_runtime --example play
tools/parity.sh fixtures/platformer.json 300 "10:+ArrowRight,30:+Space,33:-Space" "Joueur"
```
Le script joue la même partie (mêmes touches, même hasard) dans les deux moteurs et compare
positions, variables et nombre d'objets. Voir `PARITY.md`.
