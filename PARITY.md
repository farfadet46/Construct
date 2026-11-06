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
| Débogueur : pause, image par image, vitesse, collisions, variables modifiables, instances, points d'arrêt sur les événements | ✅ |
| Manette de jeu | ✅ |
| Boutons tactiles | ⬜ (sans objet sur ordinateur) |

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
| Import d'images (sprites, mosaïques, planches découpées) et de sons | ✅ |
| Éditeur de sprites : crayon, gomme, remplissage, pipette, ligne, rectangle, taille du pinceau, palette, annuler, animations (ajout, nom, vitesse, boucle), images (ajout, duplication, ordre, miroir), pelure d'oignon, aperçu animé, taille de l'image, origine, boîte de collision, import d'image / de planche | ✅ |
| Éditeur de sprites : polygone de collision (par image, depuis la boîte, copie sur toutes les images), points d'image (ajout, renommage, placement, recopie) | ✅ |
| Peinture des cartes de tuiles : palette, peindre / effacer, remplissage de zone, taille de la carte, type de collision de chaque tuile | ✅ |
| Éditeur de trajectoires : ajouter, déplacer (aimanté), supprimer les points, aperçu du parcours | ✅ |
| Éditeur visuel de piste (Route 3D) : modèles, tronçons (longueur, virage, relief), vue du pilote animée (« Rouler »), vue de dessus, profil du relief | ✅ |
| Éditeur de cinématiques : 13 types d'étapes, étapes simultanées, ordre, options (bandes, blocage, gel, Échap), test direct | ✅ |
| Générateur d'effets sonores (façon sfxr) : préréglages, variation, 13 réglages, forme d'onde, écoute, modification ultérieure | ✅ |
| Rechercher / remplacer dans tout le projet (Ctrl+F), avec accès direct au résultat | ✅ |
| Historique visuel des modifications (Édition → Historique, Ctrl+H) : description de chaque étape, heure, retour à n'importe quel état, rétablissement | ✅ |
| Export page web (.html) et jeu web installable (PWA .zip : manifeste, icônes, hors connexion), avec le moteur JavaScript d'origine | ✅ |
| Aide intégrée | ✅ |
| Thème clair / sombre (bouton ☀ / ☾ en haut à droite, choix mémorisé) | ✅ |
| Copier / coller des événements (Ctrl+C / Ctrl+V, entre feuilles) | ✅ |
| Glisser-déposer des événements (poignée ⠿ : avant, après, ou dedans un événement / groupe) | ✅ |

## Au-delà de la version HTML (outils de création)

| Ajout | État |
|---|---|
| Sprites : outil Sélection (déplacer, copie avec Alt, flèches, Suppr), copier / couper / coller entre sprites | ✅ |
| Sprites : ellipse, formes pleines, dessin symétrique (↔, ↕) | ✅ |
| Sprites : retourner ↔ ↕, rotation 90°, décalage bouclé, contour automatique, remplacer une couleur (image ou toute l'animation) | ✅ |
| Sprites : pelure d'oignon avant (rouge) et après (bleu) | ✅ |
| Sprites : découpe de planche par taille d'image, dans une nouvelle animation, cases vides ignorées ; export PNG (image ou planche) | ✅ |
| Tuiles : rectangle, tampon de plusieurs tuiles (glisser dans la palette), pipette (Alt+clic), tracé continu sans trous | ✅ |
| Tuiles : terrain automatique 3×3 (bords et coins posés tout seuls) | ✅ |
| Particules : éditeur dédié en français, couleurs, forme, aperçu en direct par le moteur du jeu (4 fonds) | ✅ |
| Sons : bibliothèque (forme d'onde, durée, format, taille, nombre d'utilisations, import multiple, export, suppression avec avertissement) | ✅ |
| Sons : rognage, volume, normalisation, fondus d'entrée / de sortie, écoute du résultat | ✅ |
| Interface en français ou en anglais (bouton FR / EN, choix mémorisé, langue du système au premier lancement) ; conditions, actions et comportements traduits aussi | ✅ |
| Correctif : les boutons à l'intérieur d'un événement (« + action », « + condition », cases à cocher…) recevaient mal les clics | ✅ |
