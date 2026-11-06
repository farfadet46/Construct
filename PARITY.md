# Parité avec la version HTML

✅ porté et vérifié · 🟡 porté en version simplifiée · ⬜ pas encore porté (prochaine étape)

## Moteur de jeu — complet

Vérification automatique (`tools/parity.sh`) sur les 14 modèles, 300 images de jeu avec le
même scénario de touches et le même hasard : **positions, variables globales, niveau et nombre
d'objets identiques** à la version HTML.

| Fonction | État |
|---|---|
| Langage d'expressions (priorités, `&`, `et/ou/non`, conversions texte ↔ nombre de JavaScript, messages d'erreur) | ✅ |
| Fonctions : random, floor…, clamp, lerp, choose, int, float, str, len, distance, angle, zeropad, upper/lower, call, storage, loopindex, highscore… | ✅ |
| Expressions système et d'objets (X, Y, Speed, IsOnFloor, AnimationName, Lap, Rank, Horizon…) | ✅ |
| Conditions : système, clavier, souris, audio, sauvegarde, objets, comportements, tableaux / dictionnaires | ✅ (toutes celles du catalogue) |
| Actions : toutes celles du catalogue | ✅ |
| Sélection d'objets, sous-événements, Sinon, OU, une seule fois, toutes les X secondes | ✅ |
| Boucles (répéter, pour, pour chaque objet / élément, tant que), attendre, fonctions avec paramètres et retour | ✅ |
| Groupes (activer / désactiver), variables locales (statiques), inclusion de feuilles | ✅ |
| Comportements : plateforme (double saut, saut mural, glissade, échelles), solide, traversable, échelle, patrouille (poursuite, saut), physique (boîtes, cercles, cordes, poussée en chaîne), trajectoire, oscillation, projectile, rotation, minuterie, rester dans le niveau, détruit hors du niveau, 8 directions, voiture, interpolation, fondu, attacher, glisser-déposer, tourelle, vision, lumière, caméra | ✅ |
| Collisions : boîtes, polygones, pentes, cartes de tuiles (solides, traversables, pentes) | ✅ |
| Familles, variables d'instance, points d'image, calques, parallaxe, interface fixe | ✅ |
| Caméra : suivi, lissage, zone morte, anticipation, zoom animé, limites, secousses | ✅ |
| Effets : obscurité + lumières, flash d'écran, teinte, clignotement, ralenti, transitions (fondu, cercle, balayage) | ✅ |
| Particules (toutes les formes dont « douce », mélange additif) | ✅ |
| Dialogues, questions à choix, portraits, cinématiques (caméra, déplacements, titres, fondus, sons) | ✅ |
| Boutons (souris, clavier, manette de navigation), barres, classements avec saisie du nom | ✅ |
| Sauvegarde de partie, points de reprise, stockage persistant (fichier sur le disque) | ✅ |
| Menu des touches configurables par le joueur | ✅ |
| Route 3D (rallye) : piste, virages, bosses, panneaux automatiques, rivaux, trafic, décor, mini-carte, tours, chronos | ✅ |
| Mini-carte (radar / niveau entier, points clignotants) | ✅ |
| Sons intégrés (synthétisés), fichiers importés, musique avec fondus, étiquettes, volume, coupure (M) | ✅ |
| Son du moteur de voiture (rapports, crissement, hors-piste) | ✅ |
| Débogueur : pause, image par image, vitesse, collisions, variables modifiables, instances | 🟡 (points d'arrêt non réglables depuis l'éditeur) |
| Manette de jeu, boutons tactiles | ⬜ |

## Éditeur

| Fonction | État |
|---|---|
| Choix d'un modèle (14 jeux), ouvrir / enregistrer (même format JSON) | ✅ |
| Annuler / Rétablir (100 étapes) | ✅ |
| Arbre du projet : niveaux, feuilles, objets (filtre), sons, variables globales | ✅ |
| Vue du niveau : aperçu fidèle, zoom, défilement, grille, aimantation, calques (afficher / verrouiller), sélection, déplacement, redimensionnement, glisser-déposer depuis l'arbre, flèches, dupliquer, supprimer | ✅ |
| Propriétés : instance, objet (tous les champs), comportements (paramètres du catalogue), variables, niveau et calques, variables globales (renommage sûr dans les expressions), paramètres du jeu, commandes | ✅ |
| Feuille d'événements : affichage, ajout d'événements / sous-événements / commentaires / groupes / fonctions / variables locales / inclusions, sélecteur de conditions et d'actions, édition des paramètres (vérification des expressions), inverser, désactiver, déplacer, dupliquer, supprimer | ✅ |
| Tester le jeu (tout le jeu ou ce niveau), recommencer, pause | ✅ |
| Exporter le jeu : application autonome (copie de l'exécutable + jeu) | ✅ |
| Import d'images (sprites, mosaïques) et de sons | 🟡 (remplacement d'image, pas encore de découpage de planche) |
| Éditeur de sprites (dessin pixel, animations, polygone de collision, points d'image) | ⬜ |
| Peinture des cartes de tuiles dans le niveau | ⬜ |
| Éditeur de trajectoires | ⬜ |
| Éditeur visuel de piste (Route 3D) | ⬜ (la piste se modifie en texte) |
| Éditeur de cinématiques | ⬜ (elles sont jouées, pas encore modifiables) |
| Générateur d'effets sonores | ⬜ |
| Rechercher / remplacer, historique visuel | ⬜ |
| Export HTML / PWA (jeu web) | ⬜ |
| Thème clair, aide intégrée | ⬜ |
| Glisser-déposer des événements, copier / coller entre feuilles | ⬜ |
