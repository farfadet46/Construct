(function(G){
"use strict";
/* ================= CATALOGUE ================= */
var CMP=[['=','='],['<>','≠'],['<','<'],['<=','≤'],['>','>'],['>=','≥']];
var CMPSYM={'=':'=','<>':'≠','<':'<','<=':'≤','>':'>','>=':'≥'};
function cmp(a,op,b){switch(op){case '=':return a==b;case '<>':return a!=b;case '<':return a<b;case '<=':return a<=b;case '>':return a>b;case '>=':return a>=b;}return false;}
var KEYS=[['ArrowLeft','←'],['ArrowRight','→'],['ArrowUp','↑'],['ArrowDown','↓'],['Space','Espace'],['Enter','Entrée'],['ShiftLeft','Maj'],['ControlLeft','Ctrl'],['Escape','Échap'],['Tab','Tab'],['Backspace','Retour']];
'ABCDEFGHIJKLMNOPQRSTUVWXYZ'.split('').forEach(function(l){KEYS.push(['Key'+l,l]);});
'0123456789'.split('').forEach(function(d){KEYS.push(['Digit'+d,d]);});
var KEYNAME={};KEYS.forEach(function(k){KEYNAME[k[0]]=k[1];});
var SOUNDS=[['jump','Saut'],['coin','Pièce'],['hit','Choc'],['hurt','Blessure'],['powerup','Bonus'],['spring','Ressort'],['laser','Laser'],['explosion','Explosion'],['blip','Bip'],['win','Victoire'],['lose','Défaite']];

var BEHAVIORS={
 platform:{n:'Plateforme',d:'Court et saute avec les flèches (ou ZQSD/WASD et Espace). Gère la gravité et les collisions avec Solide et Traversable.',p:{maxSpeed:{l:'Vitesse max',d:240,min:0,max:1000},accel:{l:'Accélération',d:1500,min:0,max:6000},decel:{l:'Décélération',d:1500,min:0,max:6000},jump:{l:'Force de saut',d:620,min:0,max:2000},gravity:{l:'Gravité',d:1600,min:0,max:5000},maxFall:{l:'Chute max',d:1000,min:0,max:3000},doubleJump:{l:'Double saut',t:'bool',d:false},wallJump:{l:'Saut mural (sauter contre un mur en l’air)',t:'bool',d:false},wallSlide:{l:'Glissade le long des murs : vitesse max (0 = non)',d:0,min:0,max:2000},wallKick:{l:'Saut mural : poussée horizontale',d:320,min:0,max:2000},climbSpeed:{l:'Vitesse sur les échelles (0 = ne grimpe pas)',d:140,min:0,max:1000},controls:{l:'Contrôles par défaut',t:'bool',d:true}}},
 ladder:{n:'Échelle',d:'Un objet Plateforme peut y grimper avec ↑ / ↓ et se tenir sur le haut. Espace pour sauter de l’échelle.',p:{}},
 solid:{n:'Solide',d:'Bloque les objets Plateforme et Patrouille (sol, murs, plafonds).',p:{}},
 jumpthru:{n:'Traversable',d:'Plateforme sur laquelle on peut sauter depuis le dessous.',p:{}},
 scrollto:{n:'Caméra',d:'La vue suit cet objet.',p:{smooth:{l:'Lissage (0 = instantané, 90 = très doux)',d:0,min:0,max:95},deadW:{l:'Zone morte : largeur (px)',d:0,min:0,max:2000},deadH:{l:'Zone morte : hauteur (px)',d:0,min:0,max:2000},lookahead:{l:'Anticipation dans la direction du mouvement (px)',d:0,min:0,max:600},offsetY:{l:'Décalage vertical (px, négatif = vue plus haute)',d:0,min:-1000,max:1000}}},
 patrol:{n:'Patrouille',d:'Marche, tombe et fait demi-tour contre les murs (et au bord du vide).',p:{speed:{l:'Vitesse',d:60,min:0,max:600},ledges:{l:'Demi-tour au bord du vide',t:'bool',d:true},startLeft:{l:'Part vers la gauche',t:'bool',d:true},flip:{l:'Miroir automatique',t:'bool',d:true},gravity:{l:'Gravité',d:1600,min:0,max:5000},chase:{l:'Poursuivre (objet visé, vide = jamais)',t:'obj',d:''},range:{l:'Distance de détection (px)',d:240,min:0,max:3000},chaseSpeed:{l:'Vitesse de poursuite',d:110,min:0,max:800},jump:{l:'Saut pour franchir murs et trous (0 = ne saute pas)',d:0,min:0,max:2000}}},
 physics:{n:'Physique',d:'Objet physique réaliste : tombe, rebondit, roule et se pousse (caisses, ballons, rochers). Le héros peut le pousser et monter dessus.',p:{shape:{l:'Forme',t:'select',o:[['box','Boîte'],['circle','Cercle (roule)']],d:'box'},density:{l:'Densité (poids)',d:1,min:.05,max:50,step:.05},friction:{l:'Frottement',d:.5,min:0,max:1,step:.05},bounce:{l:'Rebond',d:.2,min:0,max:1,step:.05},gravity:{l:'Gravité',d:1600,min:-5000,max:5000},immovable:{l:'Immobile (fixe, mais les autres rebondissent dessus)',t:'bool',d:false},pushable:{l:'Peut être poussé par le héros',t:'bool',d:true},rope:{l:'Suspendu à une corde : longueur (0 = non)',d:0,min:0,max:2000}}},
 path:{n:'Trajectoire',d:'Suit un chemin de points dessiné dans le niveau (sélectionnez l’instance puis « Modifier la trajectoire »). Les instances créées pendant le jeu reprennent le chemin de la première instance placée (vagues d’ennemis). Plateformes mobiles, ennemis volants, rondes…',p:{speed:{l:'Vitesse (px/s)',d:100,min:0,max:2000},mode:{l:'Parcours',t:'select',o:[['pingpong','Aller-retour'],['loop','Boucle'],['once','Une fois']],d:'pingpong'},pause:{l:'Pause à chaque point (s)',d:0,min:0,max:30,step:.1},ease:{l:'Ralentir aux points',t:'bool',d:false},rotate:{l:'S’orienter dans la direction',t:'bool',d:false},auto:{l:'Démarre tout seul',t:'bool',d:true}}},
 sine:{n:'Oscillation',d:'Va-et-vient régulier, idéal pour les plateformes mobiles.',p:{axis:{l:'Mouvement',t:'select',o:[['h','Horizontal'],['v','Vertical']],d:'h'},period:{l:'Période (s)',d:3,min:.1,max:30,step:.1},magnitude:{l:'Amplitude (px)',d:64,min:0,max:2000}}},
 bullet:{n:'Projectile',d:'Avance en ligne droite selon un angle.',p:{speed:{l:'Vitesse',d:300,min:0,max:3000},angle:{l:'Angle (°)',d:0,min:-360,max:360},gravity:{l:'Gravité',d:0,min:0,max:5000}}},
 rotate:{n:'Rotation',d:'Tourne en continu.',p:{speed:{l:'Vitesse (°/s)',d:90,min:-2000,max:2000}}},
 timer:{n:'Minuterie',d:'Déclenche des événements après un délai, une seule fois ou en boucle (tirs, apparitions, pièges…). Chaque minuterie a un nom.',p:{}},
 bound:{n:'Rester dans le niveau',d:'Empêche l’objet de sortir du niveau par la gauche, la droite et le haut. Le bas reste ouvert pour pouvoir tomber dans le vide.',p:{bottom:{l:'Bloquer aussi le bas',t:'bool',d:false}}},
 eightdir:{n:'8 directions',d:'Déplacement vu de dessus avec les flèches (ou ZQSD/WASD). Bloqué par les objets Solides.',p:{maxSpeed:{l:'Vitesse max',d:200,min:0,max:2000},accel:{l:'Accélération',d:1200,min:0,max:10000},decel:{l:'Décélération',d:1600,min:0,max:10000},dirs:{l:'Directions',t:'select',o:[['8','8 directions'],['4','4 directions'],['h','Gauche / droite'],['v','Haut / bas']],d:'8'},rotate:{l:'Tourner dans la direction',t:'bool',d:false},controls:{l:'Contrôles par défaut',t:'bool',d:true}}},
 car:{n:'Voiture',d:'Conduite vue de dessus : ↑ accélère, ↓ freine puis recule, ← → tournent. La voiture dérape dans les virages et rebondit sur les objets Solides.',p:{maxSpeed:{l:'Vitesse max (px/s)',d:420,min:0,max:3000},accel:{l:'Accélération',d:360,min:0,max:10000},brake:{l:'Freinage',d:760,min:0,max:10000},friction:{l:'Ralentissement sans gaz',d:200,min:0,max:10000},reverse:{l:'Vitesse max en marche arrière',d:150,min:0,max:3000},steer:{l:'Braquage (°/s)',d:190,min:0,max:1000},drift:{l:'Dérapage (0 = rails, 1 = glace)',d:.25,min:0,max:1,step:.05},controls:{l:'Contrôles par défaut (flèches / ZQSD)',t:'bool',d:true},engine:{l:'Son de moteur quand elle est pilotée',t:'bool',d:true}}},
 tween:{n:'Interpolation',d:'Anime en douceur la position, la taille, l’angle ou l’opacité (portes, menus, bonus qui flottent…). Se pilote avec l’action « Démarrer une interpolation ».',p:{}},
 fade:{n:'Fondu',d:'Apparaît et/ou disparaît en fondu, puis peut détruire l’objet (textes de points, effets, messages).',p:{fadeIn:{l:'Apparition (s)',d:0,min:0,max:60,step:.1},wait:{l:'Attente (s)',d:1,min:0,max:60,step:.1},fadeOut:{l:'Disparition (s)',d:1,min:0,max:60,step:.1},destroy:{l:'Détruire à la fin',t:'bool',d:true},auto:{l:'Démarrer tout seul',t:'bool',d:true}}},
 pin:{n:'Attacher',d:'Suit un autre objet : arme tenue, chapeau, barre de vie… Utilisez l’action « Attacher à un objet ».',p:{mirror:{l:'Suivre le miroir',t:'bool',d:true},angle:{l:'Suivre l’angle',t:'bool',d:false},destroy:{l:'Détruire avec l’objet suivi',t:'bool',d:true}}},
 drag:{n:'Glisser-déposer',d:'Peut être déplacé à la souris ou au doigt.',p:{axes:{l:'Axes',t:'select',o:[['both','Libre'],['h','Horizontal'],['v','Vertical']],d:'both'}}},
 turret:{n:'Tourelle',d:'Vise la cible la plus proche à portée et déclenche « Au tir » à intervalle régulier. Ajoutez les cibles avec l’action « Ajouter une cible ».',p:{range:{l:'Portée (px)',d:300,min:0,max:5000},rate:{l:'Cadence (s entre deux tirs)',d:1,min:.05,max:60,step:.05},rotSpeed:{l:'Vitesse de rotation (°/s, 0 = instantané)',d:270,min:0,max:5000},rotate:{l:'Tourner vers la cible',t:'bool',d:true}}},
 los:{n:'Vision',d:'Permet de savoir si un objet est visible : à portée, dans le cône de vision et sans Solide entre les deux.',p:{range:{l:'Portée (px)',d:400,min:0,max:5000},cone:{l:'Cône de vision (°)',d:360,min:1,max:360}}},
 light:{n:'Lumière',d:'Éclaire autour de l’objet quand le niveau est plongé dans l’obscurité (action « Définir l’obscurité »).',p:{radius:{l:'Rayon (px)',d:150,min:0,max:4000},intensity:{l:'Intensité (%)',d:100,min:0,max:100},color:{l:'Couleur de la lueur',t:'color',d:'#ffd9a0'},glow:{l:'Force de la lueur colorée (%)',d:25,min:0,max:100},flicker:{l:'Vacillement (flamme)',t:'bool',d:false}}},
 destroy:{n:'Détruit hors du niveau',d:'Détruit l’objet quand il sort du niveau.',p:{}}
};
function bdefaults(t){var o={},d=BEHAVIORS[t];if(d)for(var k in d.p)o[k]=d.p[k].d;return o;}

var X_='expr';
function P(n,t,d,x){var o={n:n,t:t};if(d!==undefined)o.d=d;if(x)for(var k in x)o[k]=x[k];return o;}
var pCmp=P('Comparaison','cmp','='),pObj=P('Objet','obj'),pKey=P('Touche','key','Space');
var CONDS={
 system:{
  start:{n:'Au démarrage du niveau',t:'Au démarrage du niveau',trig:1,cat:'Général'},
  tick:{n:'À chaque image',t:'À chaque image',cat:'Général'},
  'else':{n:'Sinon',t:'Sinon',cat:'Général',hint:'Vrai si l’événement précédent (au même niveau) n’a pas été exécuté.'},
  once:{n:'Une seule fois tant que vrai',t:'Une seule fois tant que vrai',cat:'Général'},
  every:{n:'Toutes les X secondes',t:'Toutes les {0} secondes',p:[P('Secondes',X_,'1')],cat:'Temps'},
  compare:{n:'Comparer deux valeurs',t:'{0} {1} {2}',p:[P('Valeur 1',X_,'0'),pCmp,P('Valeur 2',X_,'0')],cat:'Valeurs'},
  ctlopen:{n:'Le menu des touches est ouvert',t:'Le menu des touches est ouvert',cat:'Commandes'},
  foreachitem:{n:'Pour chaque élément',t:'Pour chaque élément de {0}',loop:1,p:[P('Tableau ou dictionnaire','obj')],cat:'Boucles',hint:'Pendant la boucle : CurValue (valeur), CurKey (clé du dictionnaire ou index du tableau).'},
  gamepaused:{n:'Le jeu est en pause',t:'Le jeu est en pause',cat:'Pause'},
  hsqual:{n:'Le score entre dans le classement',t:'{1} entre dans le classement « {0} » ({2} places)',p:[P('Classement','name','scores'),P('Score',X_,'Score'),P('Places',X_,'10'),P('Ordre','choice','desc',{o:[['desc','le plus grand gagne'],['asc','le plus petit gagne (temps)']]})],cat:'Meilleurs scores',hint:'Vrai si ce score prendrait une place dans le classement (pratique pour ne demander le nom qu’en cas de record).'},
  hsnamed:{n:'Quand le joueur a saisi son nom',t:'Quand le joueur a saisi son nom',trig:1,cat:'Meilleurs scores',hint:'Après « Demander le nom du joueur ». Le nom est dans PlayerName.'},
  hsopen:{n:'La saisie du nom est affichée',t:'La saisie du nom est affichée',cat:'Meilleurs scores'},
  cineplaying:{n:'Une cinématique est en cours',t:'Une cinématique est en cours',cat:'Cinématiques'},
  cineended:{n:'Quand une cinématique se termine',t:'Quand la cinématique {0} se termine',trig:1,p:[P('Cinématique','cine')],cat:'Cinématiques'},
  hassave:{n:'Une sauvegarde existe',t:'Une sauvegarde « {0} » existe',p:[P('Emplacement','name','partie')],cat:'Sauvegarde de partie'},
  hascp:{n:'Un point de reprise est enregistré',t:'Un point de reprise est enregistré',cat:'Points de reprise'},
  dlgopen:{n:'Un dialogue est affiché',t:'Un dialogue est affiché',cat:'Dialogues'},
  dlgend:{n:'À la fin du dialogue',t:'À la fin du dialogue',trig:1,cat:'Dialogues',hint:'Quand la dernière réplique est fermée.'},
  dlgchoice:{n:'Au choix d’une réponse',t:'Réponse choisie : n°{0}',trig:1,p:[P('Numéro de la réponse (0 = n’importe laquelle)',X_,'1')],cat:'Dialogues',hint:'Les réponses sont numérotées à partir de 1. L’expression DialogueChoiceText donne le texte choisi.'},
  gcmp:{n:'Comparer une variable',t:'{0} {1} {2}',p:[P('Variable','gvar'),pCmp,P('Valeur',X_,'0')],cat:'Variables',hint:'Variable globale, variable locale ou paramètre de fonction.'},
  groupactive:{n:'Un groupe est actif',t:'Le groupe {0} est actif',p:[P('Groupe','gname')],cat:'Groupes'},
  chance:{n:'Probabilité',t:'Avec {0} % de chances',p:[P('Pourcentage',X_,'50')],cat:'Valeurs'},
  islayout:{n:'Le niveau actuel est',t:'Le niveau est {0}',p:[P('Niveau','layout')],cat:'Niveaux'},
  repeat:{n:'Répéter',t:'Répéter {0} fois',p:[P('Nombre de fois',X_,'10')],cat:'Boucles',loop:1,hint:'Le reste de l’événement (conditions suivantes, actions, sous-événements) est exécuté plusieurs fois. loopindex donne le numéro du tour (0, 1, 2…).'},
  forl:{n:'Pour (boucle numérotée)',t:'Pour « {0} » de {1} à {2}',p:[P('Nom de la boucle','name','i'),P('Début',X_,'1'),P('Fin',X_,'10')],cat:'Boucles',loop:1,hint:'Utilisez loopindex("nom") pour lire la valeur courante.'},
  foreach:{n:'Pour chaque objet',t:'Pour chaque {0}',p:[P('Objet','obj')],cat:'Boucles',loop:1,hint:'Exécute le reste de l’événement une fois par objet, chaque objet étant sélectionné seul à son tour.'},
  'while':{n:'Tant que',t:'Tant que',cat:'Boucles',loop:1,hint:'Répète tant que les conditions suivantes sont vraies (10 000 tours maximum).'},
  pickall:{n:'Choisir tous les objets',t:'Choisir tous les {0}',p:[P('Objet','obj')],cat:'Sélection',hint:'Annule les sélections précédentes de cet objet dans l’événement.'},
  count:{n:'Comparer le nombre d’objets',t:'Nombre de {0} {1} {2}',p:[P('Objet','obj'),pCmp,P('Valeur',X_,'0')],cat:'Sélection'}
 },
 mouse:{
  click:{n:'Clic (ou toucher)',t:'Clic {0}',trig:1,p:[P('Bouton','choice','left',{o:[['left','gauche'],['right','droit'],['middle','du milieu']]})],cat:'Souris et tactile'},
  down:{n:'Bouton maintenu',t:'Bouton {0} maintenu',p:[P('Bouton','choice','left',{o:[['left','gauche'],['right','droit'],['middle','du milieu']]})],cat:'Souris et tactile'},
  released:{n:'Bouton relâché',t:'Bouton {0} relâché',trig:1,p:[P('Bouton','choice','left',{o:[['left','gauche'],['right','droit'],['middle','du milieu']]})],cat:'Souris et tactile'},
  clickobj:{n:'Clic sur un objet',t:'Clic {0} sur {1}',trig:1,p:[P('Bouton','choice','left',{o:[['left','gauche'],['right','droit'],['middle','du milieu']]}),P('Objet','obj')],cat:'Souris et tactile',hint:'Fonctionne aussi au doigt sur mobile. Sélectionne les objets cliqués.'},
  overobj:{n:'Le curseur est sur un objet',t:'Le curseur est sur {0}',p:[P('Objet','obj')],cat:'Souris et tactile'}
 },
 audio:{
  musicplaying:{n:'Une musique est en cours',t:'Une musique est en cours',cat:'Audio'},
  tagplaying:{n:'Un son étiqueté est en cours',t:'Le son « {0} » est en cours',p:[P('Étiquette','name','son')],cat:'Audio'}
 },
 storage:{
  exists:{n:'Une sauvegarde existe',t:'La sauvegarde « {0} » existe',p:[P('Clé','name','record')],cat:'Sauvegarde'}
 },
 keyboard:{
  pressed:{n:'Touche pressée',t:'Touche {0} pressée',trig:1,p:[pKey],cat:'Clavier'},
  down:{n:'Touche maintenue',t:'Touche {0} maintenue',p:[pKey],cat:'Clavier'},
  released:{n:'Touche relâchée',t:'Touche {0} relâchée',trig:1,p:[pKey],cat:'Clavier'},
  any:{n:'N’importe quelle touche pressée',t:'N’importe quelle touche pressée',trig:1,cat:'Clavier'}
 },
 common:{
  collide:{n:'Entre en collision avec',t:'Entre en collision avec {0}',trig:1,p:[pObj],cat:'Collisions'},
  overlap:{n:'Chevauche',t:'Chevauche {0}',p:[pObj],cat:'Collisions'},
  cmpx:{n:'Comparer X',t:'X {0} {1}',p:[pCmp,P('Valeur',X_,'0')],cat:'Position'},
  cmpy:{n:'Comparer Y',t:'Y {0} {1}',p:[pCmp,P('Valeur',X_,'0')],cat:'Position'},
  onscreen:{n:'Est à l’écran',t:'Est à l’écran',cat:'Position'},
  outside:{n:'Est hors du niveau',t:'Est hors du niveau',cat:'Position'},
  ivar:{n:'Comparer une variable d’instance',t:'{0} {1} {2}',p:[P('Variable','ivar'),pCmp,P('Valeur',X_,'0')],cat:'Variables'},
  visible:{n:'Est visible',t:'Est visible',cat:'Apparence'},
  pickby:{n:'Choisir par comparaison',t:'Où {0} {1} {2}',p:[P('Expression (Self = l’objet testé)',X_,'Self.X'),pCmp,P('Valeur',X_,'0')],cat:'Sélection'},
  pnear:{n:'Choisir le plus proche',t:'Le plus proche de ({0}, {1})',p:[P('X',X_,'0'),P('Y',X_,'0')],cat:'Sélection'},
  pfar:{n:'Choisir le plus éloigné',t:'Le plus éloigné de ({0}, {1})',p:[P('X',X_,'0'),P('Y',X_,'0')],cat:'Sélection'},
  prand:{n:'Choisir au hasard',t:'Un seul, choisi au hasard',cat:'Sélection'},
  onlayer:{n:'Est sur le calque',t:'Est sur le calque {0}',p:[P('Calque','lname')],cat:'Calques'}
 },
 sprite:{
  animis:{n:'L’animation en cours est',t:'L’animation est {0}',p:[P('Animation','anim')],cat:'Animation'},
  animend:{n:'À la fin d’une animation',t:'À la fin de l’animation {0}',trig:1,p:[P('Animation','anim')],cat:'Animation'},
  mirrored:{n:'Est en miroir',t:'Est en miroir',cat:'Apparence'}
 },
 text:{
  cmptext:{n:'Comparer le texte',t:'Texte = {0}',p:[P('Texte',X_,'""')],cat:'Texte'}
 },
 'b:platform':{
  floor:{n:'Est au sol',t:'Est au sol',cat:'Plateforme'},
  onobj:{n:'Est au sol sur un objet',t:'Est au sol sur {0}',p:[P('Objet','obj')],cat:'Plateforme'},
  moving:{n:'Est en mouvement',t:'Est en mouvement',cat:'Plateforme'},
  jumping:{n:'Est en train de sauter',t:'Est en train de sauter',cat:'Plateforme'},
  falling:{n:'Est en train de tomber',t:'Est en train de tomber',cat:'Plateforme'},
  landed:{n:'À l’atterrissage',t:'À l’atterrissage',trig:1,cat:'Plateforme'},
  jumped:{n:'Au saut',t:'Au saut',trig:1,cat:'Plateforme'},
  onwall:{n:'Est contre un mur',t:'Est contre un mur (en l’air)',cat:'Plateforme'},
  walljumped:{n:'Au saut mural',t:'Au saut mural',trig:1,cat:'Plateforme'},
  climbing:{n:'Grimpe à une échelle',t:'Grimpe à une échelle',cat:'Plateforme'}
 },
 'b:patrol':{
  goingleft:{n:'Va vers la gauche',t:'Va vers la gauche',cat:'Patrouille'},
  chasing:{n:'Poursuit sa cible',t:'Poursuit sa cible',cat:'Patrouille',hint:'Vrai quand l’objet visé (réglage « Poursuivre ») est assez proche.'},
  spotted:{n:'Quand il repère sa cible',t:'Quand il repère sa cible',trig:1,cat:'Patrouille'}
 },
 'b:timer':{
  ontimer:{n:'Quand une minuterie se déclenche',t:'Minuterie « {0} » déclenchée',trig:1,p:[P('Nom','name','minuterie')],cat:'Minuterie'},
  running:{n:'Une minuterie est en cours',t:'Minuterie « {0} » en cours',p:[P('Nom','name','minuterie')],cat:'Minuterie'}
 },
 'b:eightdir':{moving:{n:'Est en mouvement',t:'Est en mouvement',cat:'8 directions'}},
 'b:car':{moving:{n:'Roule',t:'Roule',cat:'Voiture'},reversing:{n:'Recule',t:'Recule',cat:'Voiture'},driven:{n:'Est pilotée au clavier',t:'Est pilotée au clavier',cat:'Voiture'},hit:{n:'Quand elle heurte un obstacle',t:'Quand elle heurte un obstacle (vitesse > {0})',trig:1,p:[P('Vitesse minimale',X_,'60')],cat:'Voiture',hint:'Solides, murs, autres voitures… L’expression Self.Speed donne la vitesse actuelle.'},cmpspeed:{n:'Comparer la vitesse',t:'Vitesse {0} {1}',p:[pCmp,P('Vitesse',X_,'100')],cat:'Voiture'}},
 tilemap:{cmptile:{n:'Comparer une tuile',t:'Tuile ({0}, {1}) {2} {3}',p:[P('Colonne',X_,'0'),P('Ligne',X_,'0'),pCmp,P('Numéro de tuile (-1 = vide)',X_,'-1')],cat:'Tuiles'},
  cmptileat:{n:'Comparer la tuile à une position',t:'Tuile en ({0}, {1}) {2} {3}',p:[P('X',X_,'Joueur.X'),P('Y',X_,'Joueur.Y'),pCmp,P('Numéro de tuile (-1 = vide)',X_,'-1')],cat:'Tuiles'}},
 particles:{empty:{n:'N’a plus de particules',t:'N’a plus de particules',cat:'Particules'}},
 road3d:{lapdone:{n:'À chaque tour bouclé',t:'À chaque tour bouclé',trig:1,cat:'Route 3D',hint:'Self.Lap donne le nombre de tours, Self.LastLap le temps du dernier tour.'},hit:{n:'Au choc',t:'Au choc (décor ou voiture)',trig:1,cat:'Route 3D'},offroad:{n:'Est hors de la route',t:'Est hors de la route',cat:'Route 3D'},cmpspeed:{n:'Comparer la vitesse',t:'Vitesse {0} {1} km/h',p:[pCmp,P('Vitesse (km/h)',X_,'100')],cat:'Route 3D'},driven:{n:'Contrôles actifs',t:'Contrôles clavier actifs',cat:'Route 3D'}},
 bar:{cmpvalue:{n:'Comparer la valeur',t:'Valeur {0} {1}',p:[pCmp,P('Valeur',X_,'0')],cat:'Barre'},empty:{n:'Est vide',t:'Est vide (valeur à 0)',cat:'Barre'},full:{n:'Est pleine',t:'Est pleine',cat:'Barre'}},
 button:{clicked:{n:'Au clic sur le bouton',t:'Au clic sur le bouton',trig:1,cat:'Bouton',hint:'Souris, doigt, ou Entrée / Espace / manette quand le bouton a le focus.'},
  hover:{n:'Est survolé',t:'Est survolé',cat:'Bouton'},focused:{n:'A le focus (clavier / manette)',t:'A le focus',cat:'Bouton'}},
 array:{contains:{n:'Contient la valeur',t:'Contient {0}',p:[P('Valeur',X_,'0')],cat:'Tableau'},empty:{n:'Est vide',t:'Est vide',cat:'Tableau'},
  cmpat:{n:'Comparer un élément',t:'Élément n°{0} {1} {2}',p:[P('Index (à partir de 0)',X_,'0'),pCmp,P('Valeur',X_,'0')],cat:'Tableau'}},
 dict:{haskey:{n:'A la clé',t:'A la clé {0}',p:[P('Clé',X_,'"pièces"')],cat:'Dictionnaire'},empty:{n:'Est vide',t:'Est vide',cat:'Dictionnaire'},
  cmpkey:{n:'Comparer une valeur',t:'Valeur de {0} {1} {2}',p:[P('Clé',X_,'"pièces"'),pCmp,P('Valeur',X_,'0')],cat:'Dictionnaire'}},
 'b:tween':{
  done:{n:'À la fin d’une interpolation',t:'À la fin de l’interpolation « {0} »',trig:1,p:[P('Étiquette','name','tween')],cat:'Interpolation'},
  running:{n:'Une interpolation est en cours',t:'Interpolation « {0} » en cours',p:[P('Étiquette','name','tween')],cat:'Interpolation'}},
 'b:fade':{done:{n:'À la fin du fondu',t:'À la fin du fondu',trig:1,cat:'Fondu'}},
 'b:physics':{moving:{n:'Est en mouvement',t:'Est en mouvement',cat:'Physique'},onground:{n:'Est posé',t:'Est posé sur quelque chose',cat:'Physique'},hit:{n:'Quand il heurte quelque chose',t:'Quand il heurte quelque chose (vitesse > {0})',trig:1,p:[P('Vitesse minimale',X_,'200')],cat:'Physique',hint:'Pratique pour jouer un son de choc ou casser une caisse.'},roped:{n:'Est relié par une corde',t:'Est relié par une corde',cat:'Physique'}},
 'b:path':{arrived:{n:'Quand il atteint un point',t:'Quand il atteint un point',trig:1,cat:'Trajectoire',hint:'PathPoint (expression) donne le numéro du point atteint (0 = départ).'},finished:{n:'Quand le parcours est terminé',t:'Quand le parcours est terminé',trig:1,cat:'Trajectoire',hint:'Seulement en mode « Une fois ».'},moving:{n:'Est en mouvement',t:'Est en mouvement',cat:'Trajectoire'}},
 'b:pin':{pinned:{n:'Est attaché',t:'Est attaché',cat:'Attacher'}},
 'b:drag':{
  dragging:{n:'Est en train d’être glissé',t:'Est glissé',cat:'Glisser-déposer'},
  start:{n:'Au début du glissement',t:'Au début du glissement',trig:1,cat:'Glisser-déposer'},
  drop:{n:'Au dépôt',t:'Au dépôt',trig:1,cat:'Glisser-déposer'}},
 'b:turret':{
  shoot:{n:'Au tir',t:'Au tir',trig:1,cat:'Tourelle',hint:'Créez le projectile ici, par exemple avec « Créer un objet à un point d’image » (orientation comme cet objet).'},
  target:{n:'A une cible',t:'A une cible',cat:'Tourelle'}},
 'b:los':{sees:{n:'Voit un objet',t:'Voit {0}',p:[P('Objet','obj')],cat:'Vision',hint:'Sélectionne aussi les instances vues. La direction du regard suit l’angle et le miroir.'}}
};
var ACTS={
 system:{
  call:{n:'Appeler une fonction',t:'Appeler {0}{1}',p:[P('Fonction','fname'),P('Paramètres (séparés par des virgules)','args','')],cat:'Fonctions'},
  retval:{n:'Définir la valeur de retour',t:'Valeur de retour = {0}',p:[P('Valeur',X_,'0')],cat:'Fonctions',hint:'Dans une fonction : la valeur renvoyée par l’expression call("nom", …).'},
  setgroup:{n:'Activer / désactiver un groupe',t:'Groupe {0} : {1}',p:[P('Groupe','gname'),P('État','choice','on',{o:[['on','activé'],['off','désactivé'],['toggle','basculé']]})],cat:'Groupes'},
  gset:{n:'Définir une variable',t:'Mettre {0} à {1}',p:[P('Variable','gvar'),P('Valeur',X_,'0')],cat:'Variables'},
  gadd:{n:'Ajouter à une variable',t:'Ajouter {1} à {0}',p:[P('Variable','gvar'),P('Valeur',X_,'1')],cat:'Variables'},
  gsub:{n:'Soustraire d’une variable',t:'Soustraire {1} de {0}',p:[P('Variable','gvar'),P('Valeur',X_,'1')],cat:'Variables'},
  resetg:{n:'Réinitialiser les variables globales',t:'Réinitialiser les variables globales',cat:'Variables'},
  gotolayout:{n:'Aller au niveau',t:'Aller au niveau {0}',p:[P('Niveau','layout')],cat:'Niveaux'},
  controlsmenu:{n:'Ouvrir le menu des touches',t:'Ouvrir le menu des touches',cat:'Commandes',hint:'Le joueur peut y changer ses touches (liste réglée dans les paramètres du jeu, rubrique « Commandes du jeu »). Ses choix sont mémorisés dans le navigateur.'},
  pause:{n:'Mettre en pause / reprendre',t:'Pause : {0}',p:[P('Action','choice','toggle',{o:[['toggle','basculer'],['pause','mettre en pause'],['resume','reprendre']]})],cat:'Pause',hint:'Fige les déplacements, les comportements et le temps ; les événements et les boutons continuent de fonctionner (menu de pause).'},
  savegame:{n:'Sauvegarder la partie',t:'Sauvegarder la partie dans « {0} »',p:[P('Emplacement','name','partie')],cat:'Sauvegarde de partie',hint:'Mémorise le niveau en cours, les variables globales, les tableaux / dictionnaires et le point de reprise.'},
  loadgame:{n:'Charger la partie',t:'Charger la partie « {0} »',p:[P('Emplacement','name','partie')],cat:'Sauvegarde de partie',hint:'Restaure les variables et va au niveau sauvegardé. Au démarrage de ce niveau, « Un point de reprise est enregistré » permet de replacer le héros.'},
  hsadd:{n:'Ajouter au classement',t:'Ajouter {1} de {2} au classement « {0} » ({3} places)',p:[P('Classement','name','scores'),P('Score',X_,'Score'),P('Nom du joueur',X_,'PlayerName'),P('Places',X_,'10'),P('Ordre','choice','desc',{o:[['desc','le plus grand gagne'],['asc','le plus petit gagne (temps)']]})],cat:'Meilleurs scores',hint:'Le score n’est gardé que s’il fait partie des meilleurs. Le classement est enregistré dans le navigateur. LastRank donne sa place (0 = pas classé).'},
  hsask:{n:'Demander le nom du joueur',t:'Demander le nom du joueur : {0}',p:[P('Titre',X_,'"Nouveau record !"'),P('Nom proposé',X_,'PlayerName'),P('Longueur max',X_,'10')],cat:'Meilleurs scores',hint:'Fenêtre de saisie : clavier, ou flèches pour choisir les lettres (manette). Le jeu est figé pendant la saisie. Ensuite : « Quand le joueur a saisi son nom » et PlayerName.'},
  hsclear:{n:'Effacer un classement',t:'Effacer le classement « {0} »',p:[P('Classement','name','scores')],cat:'Meilleurs scores'},
  cineplay:{n:'Jouer une cinématique',t:'Jouer la cinématique {0}',p:[P('Cinématique','cine')],cat:'Cinématiques',hint:'Les cinématiques se créent dans le panneau Projet (Cinématiques). Pendant la lecture, le joueur ne contrôle plus son personnage (selon les réglages).'},
  cinestop:{n:'Arrêter la cinématique',t:'Arrêter (passer) la cinématique en cours',cat:'Cinématiques'},
  deletesave:{n:'Effacer la sauvegarde',t:'Effacer la sauvegarde « {0} »',p:[P('Emplacement','name','partie')],cat:'Sauvegarde de partie'},
  camzoom:{n:'Zoom de la caméra',t:'Zoom {0} % en {1} s',p:[P('Zoom (%) — 100 = normal',X_,'150'),P('Durée (s)',X_,'0.5')],cat:'Caméra',hint:'Les calques fixes (interface) ne sont pas zoomés. Revient à 100 % à chaque changement de niveau.'},
  camlook:{n:'Regarder une position',t:'Caméra vers ({0}, {1})',p:[P('X',X_,'LayoutWidth / 2'),P('Y',X_,'LayoutHeight / 2')],cat:'Caméra',hint:'La caméra quitte le héros pour regarder ce point (cinématique). « Suivre à nouveau » la rend au héros.'},
  camfollow:{n:'Suivre à nouveau',t:'La caméra suit à nouveau',cat:'Caméra'},
  cambounds:{n:'Limiter la caméra à une zone',t:'Caméra limitée à ({0}, {1}) {2} × {3}',p:[P('X',X_,'0'),P('Y',X_,'0'),P('Largeur',X_,'LayoutWidth'),P('Hauteur',X_,'LayoutHeight')],cat:'Caméra',hint:'Utile pour une salle de boss ou une zone fermée.'},
  camnobounds:{n:'Retirer les limites de la caméra',t:'Caméra limitée au niveau entier',cat:'Caméra'},
  setcp:{n:'Enregistrer un point de reprise',t:'Point de reprise en ({0}, {1})',p:[P('X',X_,'Joueur.X'),P('Y',X_,'Joueur.Y')],cat:'Points de reprise',hint:'Mémorise une position dans ce niveau (checkpoint). Lisez-la avec CheckpointX / CheckpointY. Effacé quand on change de niveau.'},
  respawn:{n:'Replacer au point de reprise',t:'Replacer {0} au point de reprise',p:[P('Objet','obj')],cat:'Points de reprise',hint:'Place l’objet (le héros) au dernier point de reprise et remet sa vitesse à zéro. Sans point de reprise, ne fait rien.'},
  clearcp:{n:'Effacer le point de reprise',t:'Effacer le point de reprise',cat:'Points de reprise'},
  transition:{n:'Définir la transition entre niveaux',t:'Transition {0} ({1} s, {2})',p:[P('Type','choice','fade',{o:[['none','aucune'],['fade','fondu'],['iris','cercle'],['wipe','balayage']]}),P('Durée (s)',X_,'0.6'),P('Couleur','color','#000000')],cat:'Effets visuels',hint:'S’applique à tous les changements de niveau suivants. La valeur par défaut se règle dans les paramètres du jeu.'},
  darkness:{n:'Définir l’obscurité',t:'Obscurité {0} % ({1})',p:[P('Obscurité (%)',X_,'60'),P('Couleur de la nuit','color','#0b0b1e')],cat:'Effets visuels',hint:'Les objets qui ont le comportement « Lumière » éclairent autour d’eux. L’interface (calques fixes) reste visible. Revient à 0 à chaque changement de niveau.'},
  timescale:{n:'Vitesse du jeu (ralenti)',t:'Vitesse du jeu {0} %',p:[P('Vitesse (%)',X_,'30')],cat:'Effets visuels',hint:'100 = normal, 30 = ralenti. Pratique pour un effet « bullet time » de quelques instants. Revient à 100 % à chaque changement de niveau.'},
  screenflash:{n:'Flash de l’écran',t:'Flash de l’écran {0} pendant {1} s',p:[P('Couleur','color','#ffffff'),P('Durée (s)',X_,'0.25')],cat:'Effets visuels'},
  say:{n:'Afficher une réplique',t:'Dialogue : {0} dit {1}',p:[P('Nom (vide = aucun)',X_,'"Pixel"'),P('Texte',X_,'"Bonjour !"'),P('Portrait (facultatif)','objopt',''),P('Position','choice','bottom',{o:[['bottom','en bas'],['top','en haut']]})],cat:'Dialogues',hint:'Plusieurs répliques à la suite forment une conversation. Le joueur avance avec Espace, Entrée ou un clic ; le jeu est en pause pendant le dialogue.'},
  ask:{n:'Poser une question (choix)',t:'Question de {0} : {1} → {2}',p:[P('Nom (vide = aucun)',X_,'"Pixel"'),P('Question',X_,'"Tu veux un conseil ?"'),P('Réponses (séparées par |)',X_,'"Oui|Non"'),P('Portrait (facultatif)','objopt',''),P('Position','choice','bottom',{o:[['bottom','en bas'],['top','en haut']]})],cat:'Dialogues',hint:'Ensuite, utilisez la condition « Au choix d’une réponse ». Le joueur choisit avec ↑ ↓ puis Espace, ou en cliquant.'},
  dlgclose:{n:'Fermer le dialogue',t:'Fermer le dialogue',cat:'Dialogues'},
  dlgspeed:{n:'Vitesse d’écriture',t:'Texte à {0} lettres/s',p:[P('Lettres par seconde (0 = instantané)',X_,'45')],cat:'Dialogues'},
  next:{n:'Aller au niveau suivant',t:'Aller au niveau suivant',cat:'Niveaux'},
  restart:{n:'Recommencer le niveau',t:'Recommencer le niveau',cat:'Niveaux'},
  create:{n:'Créer un objet',t:'Créer {0} en ({1}, {2}) sur {3}',p:[pObj,P('X',X_,'0'),P('Y',X_,'0'),P('Calque','lname')],cat:'Objets'},
  layervis:{n:'Afficher / masquer un calque',t:'Calque {0} visible : {1}',p:[P('Calque','lname'),P('Visible','bool',true)],cat:'Calques'},
  layeropa:{n:'Opacité d’un calque',t:'Opacité du calque {0} = {1} %',p:[P('Calque','lname'),P('Opacité (0-100)',X_,'100')],cat:'Calques'},
  wait:{n:'Attendre',t:'Attendre {0} seconde(s)',p:[P('Secondes',X_,'1')],cat:'Temps',hint:'Les actions suivantes (et les sous-événements) s’exécutent après ce délai, sur les mêmes objets.'},
  shake:{n:'Secouer la caméra',t:'Secouer la caméra ({0} px pendant {1} s)',p:[P('Intensité (px)',X_,'6'),P('Durée (s)',X_,'0.3')],cat:'Caméra'}
 },
 mouse:{
  cursor:{n:'Définir le curseur',t:'Curseur : {0}',p:[P('Curseur','choice','default',{o:[['default','normal'],['pointer','main'],['crosshair','viseur'],['none','caché']]})],cat:'Souris et tactile'}
 },
 storage:{
  savevar:{n:'Sauvegarder une variable globale',t:'Sauvegarder {0}',p:[P('Variable','gvar')],cat:'Sauvegarde',hint:'Enregistre la valeur dans le navigateur du joueur, sous le nom de la variable. Elle est conservée même après fermeture du jeu.'},
  loadvar:{n:'Charger une variable globale',t:'Charger {0}',p:[P('Variable','gvar')],cat:'Sauvegarde',hint:'Si une sauvegarde existe pour cette variable, elle reprend la valeur sauvegardée. Sinon elle ne change pas.'},
  save:{n:'Sauvegarder une valeur',t:'Sauvegarder {1} sous « {0} »',p:[P('Clé','name','cle'),P('Valeur',X_,'0')],cat:'Sauvegarde',hint:'Relisez-la avec l’expression storage("cle", valeur par défaut).'},
  remove:{n:'Supprimer une sauvegarde',t:'Supprimer la sauvegarde « {0} »',p:[P('Clé','name','cle')],cat:'Sauvegarde'},
  clear:{n:'Effacer toutes les sauvegardes',t:'Effacer toutes les sauvegardes',cat:'Sauvegarde'}
 },
 audio:{
  playfile:{n:'Jouer un son importé',t:'Jouer {0} (volume {1} %, boucle : {2}{3})',p:[P('Son','sfile'),P('Volume (%)',X_,'100'),P('En boucle','bool',false),P('Étiquette (facultative)','tag','')],cat:'Sons importés'},
  music:{n:'Jouer une musique',t:'Musique {0} (volume {1} %, fondu {2} s)',p:[P('Son','sfile'),P('Volume (%)',X_,'70'),P('Fondu d’entrée (s)',X_,'0.5')],cat:'Musique',hint:'Une seule musique à la fois, jouée en boucle. Elle continue quand on change de niveau.'},
  stopmusic:{n:'Arrêter la musique',t:'Arrêter la musique (fondu {0} s)',p:[P('Fondu de sortie (s)',X_,'0.5')],cat:'Musique'},
  stoptag:{n:'Arrêter un son étiqueté',t:'Arrêter les sons « {0} »',p:[P('Étiquette','name','son')],cat:'Sons importés'},
  stopall:{n:'Arrêter tous les sons',t:'Arrêter tous les sons et la musique',cat:'Général'},
  volume:{n:'Volume général',t:'Volume général {0} %',p:[P('Volume (%)',X_,'100')],cat:'Général'},
  play:{n:'Jouer un effet intégré',t:'Jouer l’effet {0}',p:[P('Effet','sound','coin')],cat:'Effets intégrés'}
 },
 common:{
  destroy:{n:'Détruire',t:'Détruire',cat:'Général'},
  spawn:{n:'Créer un objet à sa position',t:'Créer {0} à sa position',p:[pObj],cat:'Général'},
  posto:{n:'Placer sur un autre objet',t:'Placer sur {0} (point {1})',p:[pObj,P('Point d’image de cet objet-là (vide = origine)','iptx','')],cat:'Position',hint:'Utilise l’instance choisie par les conditions, sinon la première.'},
  setpos:{n:'Définir la position',t:'Position ({0}, {1})',p:[P('X',X_,'Self.X'),P('Y',X_,'Self.Y')],cat:'Position'},
  setx:{n:'Définir X',t:'X = {0}',p:[P('X',X_,'0')],cat:'Position'},
  sety:{n:'Définir Y',t:'Y = {0}',p:[P('Y',X_,'0')],cat:'Position'},
  move:{n:'Déplacer de',t:'Déplacer de ({0}, {1})',p:[P('Δ X',X_,'0'),P('Δ Y',X_,'0')],cat:'Position',hint:'Dans un événement répété à chaque image, multipliez par dt pour une vitesse en pixels par seconde, indépendante de l’écran : ex. 120 * dt.'},
  setsize:{n:'Définir la taille',t:'Taille {0} × {1}',p:[P('Largeur',X_,'Self.Width'),P('Hauteur',X_,'Self.Height')],cat:'Taille et angle'},
  setangle:{n:'Définir l’angle',t:'Angle = {0}°',p:[P('Angle (°)',X_,'0')],cat:'Taille et angle'},
  setvis:{n:'Définir la visibilité',t:'Visible : {0}',p:[P('Visible','bool',true)],cat:'Apparence'},
  setstyle:{n:'Changer une couleur / un style',t:'{0} = {1}',p:[P('Propriété','choice','bg',{o:[['bg','Fond (bouton, barre, classement)'],['fg','Texte (bouton) / barre / texte du classement'],['color','Couleur (objet Texte)'],['outlineColor','Contour (objet Texte)'],['hoverBg','Fond au survol (bouton)'],['accent','Accent (classement)']]}),P('Couleur','color','#3d6df2')],cat:'Apparence',hint:'Ne change que les instances concernées (celles choisies par les conditions).'},
  setopa:{n:'Définir l’opacité',t:'Opacité = {0} %',p:[P('Opacité (0-100)',X_,'100')],cat:'Apparence'},
  flash:{n:'Clignoter',t:'Clignoter pendant {0} s',p:[P('Durée (s)',X_,'1')],cat:'Apparence'},
  front:{n:'Mettre au premier plan',t:'Mettre au premier plan',cat:'Apparence'},
  tolayer:{n:'Déplacer vers un calque',t:'Déplacer vers le calque {0}',p:[P('Calque','lname')],cat:'Calques'},
  tint:{n:'Définir la teinte',t:'Teinte {0} à {1} %',p:[P('Couleur','color','#ff4d6a'),P('Intensité (%) — 0 = aucune',X_,'50')],cat:'Effets visuels',hint:'Colore les sprites (dégâts, poison, gel…).'},
  tintflash:{n:'Flash de couleur',t:'Flash {0} pendant {1} s',p:[P('Couleur','color','#ffffff'),P('Durée (s)',X_,'0.15')],cat:'Effets visuels',hint:'Le classique flash blanc quand un personnage est touché (sprites).'},
  ivset:{n:'Définir une variable d’instance',t:'Mettre {0} à {1}',p:[P('Variable','ivar'),P('Valeur',X_,'0')],cat:'Variables'},
  ivadd:{n:'Ajouter à une variable d’instance',t:'Ajouter {1} à {0}',p:[P('Variable','ivar'),P('Valeur',X_,'1')],cat:'Variables'},
  ivsub:{n:'Soustraire d’une variable d’instance',t:'Soustraire {1} de {0}',p:[P('Variable','ivar'),P('Valeur',X_,'1')],cat:'Variables'}
 },
 sprite:{
  setanim:{n:'Définir l’animation',t:'Animation {0}',p:[P('Animation','anim')],cat:'Animation'},
  mirror:{n:'Définir le miroir',t:'Miroir : {0}',p:[P('En miroir','bool',true)],cat:'Apparence'},
  spawnpt:{n:'Créer un objet à un point d’image',t:'Créer {0} au point {1} (orientation {2})',p:[pObj,P('Point d’image','ipt',''),P('Orientation','choice','same',{o:[['same','comme cet objet'],['none','par défaut']]})],cat:'Points d’image',hint:'Idéal pour tirer : un projectile créé « comme cet objet » part dans la direction où regarde le personnage.'}
 },
 text:{
  settext:{n:'Définir le texte',t:'Texte = {0}',p:[P('Texte',X_,'""')],cat:'Texte'},
  append:{n:'Ajouter du texte',t:'Ajouter {0} au texte',p:[P('Texte',X_,'""')],cat:'Texte'}
 },
 'b:platform':{
  simjump:{n:'Simuler un saut',t:'Simuler un saut',cat:'Plateforme'},
  simleft:{n:'Simuler la touche gauche',t:'Simuler gauche',cat:'Plateforme'},
  simright:{n:'Simuler la touche droite',t:'Simuler droite',cat:'Plateforme'},
  setvy:{n:'Définir la vitesse verticale',t:'Vitesse verticale = {0}',p:[P('Vitesse (négatif = haut)',X_,'-500')],cat:'Plateforme'},
  setvx:{n:'Définir la vitesse horizontale',t:'Vitesse horizontale = {0}',p:[P('Vitesse',X_,'0')],cat:'Plateforme'},
  setmax:{n:'Définir la vitesse max',t:'Vitesse max = {0}',p:[P('Vitesse',X_,'240')],cat:'Plateforme'},
  setjump:{n:'Définir la force de saut',t:'Force de saut = {0}',p:[P('Force',X_,'620')],cat:'Plateforme'},
  enable:{n:'Activer / désactiver',t:'Plateforme active : {0}',p:[P('Active','bool',true)],cat:'Plateforme'},
  controls:{n:'Activer / désactiver les contrôles',t:'Contrôles clavier : {0}',p:[P('Actifs','bool',true)],cat:'Plateforme',hint:'Sans contrôles, l’objet reste soumis à la gravité mais ne répond plus au clavier (menus, cinématiques).'}
 },
 'b:patrol':{
  reverse:{n:'Faire demi-tour',t:'Faire demi-tour',cat:'Patrouille'},
  speed:{n:'Définir la vitesse',t:'Vitesse de patrouille = {0}',p:[P('Vitesse',X_,'60')],cat:'Patrouille'},
  setchase:{n:'Changer la cible poursuivie',t:'Poursuivre {0}',p:[P('Objet visé (vide = arrêter)','objopt','')],cat:'Patrouille'}
 },
 'b:bullet':{
  speed:{n:'Définir la vitesse',t:'Vitesse du projectile = {0}',p:[P('Vitesse',X_,'300')],cat:'Projectile'},
  angle:{n:'Définir l’angle de déplacement',t:'Angle de déplacement = {0}°',p:[P('Angle (°)',X_,'0')],cat:'Projectile'},
  bounceoff:{n:'Rebondir sur un objet',t:'Rebondir sur {0}',p:[P('Objet','obj')],cat:'Projectile',hint:'À utiliser avec « Entre en collision avec » : le projectile repart dans l’autre sens (balle de casse-briques, ricochet…).'}
 },
 'b:timer':{
  start:{n:'Démarrer une minuterie',t:'Démarrer la minuterie « {0} » ({1} s, {2})',p:[P('Nom','name','minuterie'),P('Durée (s)',X_,'1'),P('Type','choice','once',{o:[['once','une fois'],['regular','en boucle']]})],cat:'Minuterie'},
  stop:{n:'Arrêter une minuterie',t:'Arrêter la minuterie « {0} »',p:[P('Nom','name','minuterie')],cat:'Minuterie'},
  stopall:{n:'Arrêter toutes les minuteries',t:'Arrêter toutes les minuteries',cat:'Minuterie'}
 },
 'b:physics':{
  impulse:{n:'Donner une impulsion',t:'Impulsion de {1} vers {0}°',p:[P('Angle (°, -90 = vers le haut)',X_,'-90'),P('Force (px/s)',X_,'500')],cat:'Physique'},
  setvel:{n:'Définir la vitesse',t:'Vitesse ({0}, {1})',p:[P('Vitesse X',X_,'0'),P('Vitesse Y',X_,'0')],cat:'Physique'},
  immovable:{n:'Rendre immobile / mobile',t:'Immobile : {0}',p:[P('Immobile','bool',true)],cat:'Physique'},
  ropeto:{n:'Relier par une corde à un objet',t:'Corde vers {0} (longueur {1})',p:[P('Objet','obj'),P('Longueur (0 = distance actuelle)',X_,'0'),P('Rigide (barre)','bool',false)],cat:'Physique',hint:'Une corde se tend mais ne pousse pas ; une barre garde toujours la même longueur.'},
  ropepoint:{n:'Accrocher à un point',t:'Corde vers ({0}, {1}) longueur {2}',p:[P('X',X_,'Self.X'),P('Y',X_,'Self.Y - 120'),P('Longueur (0 = distance actuelle)',X_,'0')],cat:'Physique'},
  cutropes:{n:'Couper les cordes',t:'Couper ses cordes',cat:'Physique'}
 },
 'b:path':{
  enable:{n:'Démarrer / arrêter',t:'Trajectoire en marche : {0}',p:[P('En marche','bool',true)],cat:'Trajectoire'},
  speed:{n:'Définir la vitesse',t:'Vitesse de trajectoire = {0}',p:[P('Vitesse',X_,'100')],cat:'Trajectoire'},
  restart:{n:'Revenir au départ',t:'Revenir au départ de la trajectoire',cat:'Trajectoire'}
 },
 'b:sine':{
  active:{n:'Activer / désactiver',t:'Oscillation active : {0}',p:[P('Active','bool',true)],cat:'Oscillation'}
 },
 tilemap:{
  settile:{n:'Définir une tuile',t:'Tuile ({0}, {1}) = {2}',p:[P('Colonne',X_,'0'),P('Ligne',X_,'0'),P('Numéro de tuile (-1 = vide)',X_,'0')],cat:'Tuiles',hint:'Les colonnes et lignes commencent à 0 en haut à gauche de la carte. Les numéros de tuile se lisent dans la palette.'},
  settileat:{n:'Définir la tuile à une position',t:'Tuile en ({0}, {1}) = {2}',p:[P('X',X_,'Self.X'),P('Y',X_,'Self.Y'),P('Numéro de tuile (-1 = vide)',X_,'-1')],cat:'Tuiles',hint:'Pratique pour casser une brique : Tuile en (Joueur.X, Joueur.Y - 40) = -1.'},
  fillrect:{n:'Remplir un rectangle de tuiles',t:'Remplir ({0}, {1}) → ({2}, {3}) avec {4}',p:[P('Colonne de départ',X_,'0'),P('Ligne de départ',X_,'0'),P('Colonne de fin',X_,'3'),P('Ligne de fin',X_,'0'),P('Numéro de tuile (-1 = vide)',X_,'0')],cat:'Tuiles'}},
 minimap:{setradius:{n:'Définir la portée du radar',t:'Portée du radar = {0} px',p:[P('Rayon visible (px)',X_,'600')],cat:'Mini-carte'}},
 road3d:{sim:{n:'Simuler une commande',t:'Simuler {0}',p:[P('Commande','choice','gas',{o:[['gas','accélérer'],['brake','freiner'],['left','tourner à gauche'],['right','tourner à droite']]})],cat:'Route 3D'},controls:{n:'Contrôles clavier',t:'Contrôles clavier : {0}',p:[P('Actifs','bool',true)],cat:'Route 3D',hint:'Désactivez-les pendant le compte à rebours, réactivez-les au départ.'},setspeed:{n:'Définir la vitesse',t:'Vitesse = {0} km/h',p:[P('Vitesse (km/h)',X_,'0')],cat:'Route 3D'},setmax:{n:'Définir la vitesse max',t:'Vitesse max = {0} km/h',p:[P('Vitesse (km/h)',X_,'220')],cat:'Route 3D'},reset:{n:'Revenir au départ',t:'Revenir au départ (tour 0, voitures replacées)',cat:'Route 3D'},settrack:{n:'Changer de circuit',t:'Circuit = {0}',p:[P('Tronçons « longueur virage colline » séparés par ;',X_,'"50 0 0; 60 3 20; 60 -3 -20"')],cat:'Route 3D'}},
 bar:{setvalue:{n:'Définir la valeur',t:'Valeur = {0}',p:[P('Valeur',X_,'100')],cat:'Barre'},addvalue:{n:'Ajouter à la valeur',t:'Ajouter {0} à la valeur',p:[P('Valeur (négatif pour retirer)',X_,'-10')],cat:'Barre'},setmax:{n:'Définir le maximum',t:'Maximum = {0}',p:[P('Maximum',X_,'100')],cat:'Barre'},setcolor:{n:'Changer la couleur',t:'Couleur de la barre = {0}',p:[P('Couleur','color','#e8566a')],cat:'Barre'}},
 leaderboard:{settable:{n:'Changer de classement',t:'Afficher le classement {0}',p:[P('Classement',X_,'"scores"')],cat:'Classement'}},
 button:{settext:{n:'Définir le texte',t:'Texte du bouton = {0}',p:[P('Texte',X_,'"OK"')],cat:'Bouton'},enable:{n:'Activer / désactiver',t:'Bouton actif : {0}',p:[P('Actif','bool',true)],cat:'Bouton'},focus:{n:'Donner le focus',t:'Donner le focus',cat:'Bouton'}},
 array:{push:{n:'Ajouter une valeur',t:'Ajouter {0} ({1})',p:[P('Valeur',X_,'0'),P('Où','choice','end',{o:[['end','à la fin'],['start','au début']]})],cat:'Tableau'},
  set:{n:'Définir un élément',t:'Élément n°{0} = {1}',p:[P('Index (à partir de 0)',X_,'0'),P('Valeur',X_,'0')],cat:'Tableau'},
  insert:{n:'Insérer une valeur',t:'Insérer {1} à l’index {0}',p:[P('Index',X_,'0'),P('Valeur',X_,'0')],cat:'Tableau'},
  removeat:{n:'Supprimer un élément',t:'Supprimer l’élément n°{0}',p:[P('Index',X_,'0')],cat:'Tableau'},
  removeval:{n:'Supprimer une valeur',t:'Supprimer la valeur {0}',p:[P('Valeur',X_,'0')],cat:'Tableau'},
  clear:{n:'Vider',t:'Vider le tableau',cat:'Tableau'},
  sort:{n:'Trier',t:'Trier ({0})',p:[P('Ordre','choice','asc',{o:[['asc','croissant'],['desc','décroissant']]})],cat:'Tableau'},
  shuffle:{n:'Mélanger',t:'Mélanger',cat:'Tableau'},
  save:{n:'Mémoriser dans le navigateur',t:'Mémoriser sous « {0} »',p:[P('Clé de stockage','name','tableau')],cat:'Stockage'},load:{n:'Recharger depuis le navigateur',t:'Recharger « {0} »',p:[P('Clé de stockage','name','tableau')],cat:'Stockage'}},
 dict:{set:{n:'Définir une clé',t:'{0} = {1}',p:[P('Clé',X_,'"pièces"'),P('Valeur',X_,'0')],cat:'Dictionnaire'},
  add:{n:'Ajouter à une clé',t:'Ajouter {1} à {0}',p:[P('Clé',X_,'"pièces"'),P('Valeur',X_,'1')],cat:'Dictionnaire',hint:'Si la clé n’existe pas, elle part de 0.'},
  remove:{n:'Supprimer une clé',t:'Supprimer la clé {0}',p:[P('Clé',X_,'"pièces"')],cat:'Dictionnaire'},
  clear:{n:'Vider',t:'Vider le dictionnaire',cat:'Dictionnaire'},
  save:{n:'Mémoriser dans le navigateur',t:'Mémoriser sous « {0} »',p:[P('Clé de stockage','name','dico')],cat:'Stockage'},load:{n:'Recharger depuis le navigateur',t:'Recharger « {0} »',p:[P('Clé de stockage','name','dico')],cat:'Stockage'}},
 particles:{
  burst:{n:'Émettre une rafale',t:'Émettre {0} particules',p:[P('Nombre',X_,'20')],cat:'Particules'},
  active:{n:'Activer / désactiver l’émission',t:'Émission active : {0}',p:[P('Active','bool',true)],cat:'Particules',hint:'Pour un émetteur continu (fumée, pluie, traînée…).'},
  setrate:{n:'Définir le débit',t:'Débit = {0} particules/s',p:[P('Particules par seconde',X_,'30')],cat:'Particules'},
  clear:{n:'Effacer les particules',t:'Effacer les particules',cat:'Particules'}},
 'b:car':{
  sim:{n:'Simuler une commande',t:'Simuler {0}',p:[P('Commande','choice','gas',{o:[['gas','accélérer'],['brake','freiner / reculer'],['left','tourner à gauche'],['right','tourner à droite']]})],cat:'Voiture',hint:'Pour piloter une voiture par les événements (IA, police…). À répéter à chaque image.'},
  setspeed:{n:'Définir la vitesse',t:'Vitesse = {0}',p:[P('Vitesse',X_,'0')],cat:'Voiture'},
  setmax:{n:'Définir la vitesse max',t:'Vitesse max = {0}',p:[P('Vitesse',X_,'420')],cat:'Voiture'},
  stop:{n:'Arrêter',t:'Arrêter net',cat:'Voiture'},
  controls:{n:'Contrôles clavier',t:'Contrôles clavier : {0}',p:[P('Actifs','bool',true)],cat:'Voiture',hint:'Activez-les sur la voiture où monte le joueur, désactivez-les quand il en descend.'},
  enable:{n:'Activer / désactiver',t:'Voiture active : {0}',p:[P('Active','bool',true)],cat:'Voiture'}},
 'b:eightdir':{
  sim:{n:'Simuler une direction',t:'Simuler {0}',p:[P('Direction','choice','right',{o:[['left','gauche'],['right','droite'],['up','haut'],['down','bas']]})],cat:'8 directions'},
  setmax:{n:'Définir la vitesse max',t:'Vitesse max = {0}',p:[P('Vitesse',X_,'200')],cat:'8 directions'},
  stop:{n:'Arrêter',t:'Arrêter net',cat:'8 directions'},
  enable:{n:'Activer / désactiver',t:'8 directions actif : {0}',p:[P('Actif','bool',true)],cat:'8 directions'}},
 'b:tween':{
  start:{n:'Démarrer une interpolation',t:'Interpoler {0} vers {1} en {2} s ({3}, {4}) « {5} »',p:[P('Propriété','choice','y',{o:[['x','X'],['y','Y'],['w','Largeur'],['h','Hauteur'],['angle','Angle'],['opacity','Opacité (%)']]}),P('Valeur d’arrivée',X_,'Self.Y - 32'),P('Durée (s)',X_,'1'),P('Courbe','choice','inout',{o:[['linear','linéaire'],['in','accélère'],['out','ralentit'],['inout','douce'],['back','élan'],['elastic','élastique'],['bounce','rebond']]}),P('Répétition','choice','once',{o:[['once','une fois'],['loop','en boucle'],['pingpong','aller-retour']]}),P('Étiquette','name','tween')],cat:'Interpolation',hint:'La valeur d’arrivée est calculée au départ : « Self.Y - 32 » monte de 32 px. Une nouvelle interpolation de la même propriété remplace la précédente.'},
  stop:{n:'Arrêter une interpolation',t:'Arrêter l’interpolation « {0} »',p:[P('Étiquette','name','tween')],cat:'Interpolation'},
  stopall:{n:'Arrêter toutes les interpolations',t:'Arrêter toutes les interpolations',cat:'Interpolation'}},
 'b:fade':{
  start:{n:'Démarrer le fondu',t:'Démarrer le fondu',cat:'Fondu'},
  out:{n:'Disparaître maintenant',t:'Disparaître en fondu en {0} s',p:[P('Durée (s)',X_,'0.5')],cat:'Fondu'}},
 'b:pin':{
  pinto:{n:'Attacher à un objet',t:'Attacher à {0} ({1})',p:[pObj,P('Point d’image (vide = garder la position actuelle)','iptx','')],cat:'Attacher',hint:'Avec un point d’image (ex. « Main »), l’objet suit exactement ce point, image par image.'},
  unpin:{n:'Détacher',t:'Détacher',cat:'Attacher'}},
 'b:drag':{drop:{n:'Lâcher',t:'Lâcher',cat:'Glisser-déposer'},enable:{n:'Activer / désactiver',t:'Glisser-déposer actif : {0}',p:[P('Actif','bool',true)],cat:'Glisser-déposer'}},
 'b:turret':{
  addtarget:{n:'Ajouter une cible',t:'Viser les {0}',p:[pObj],cat:'Tourelle'},
  cleartargets:{n:'Retirer toutes les cibles',t:'Ne plus viser personne',cat:'Tourelle'},
  setrange:{n:'Définir la portée',t:'Portée = {0}',p:[P('Portée (px)',X_,'300')],cat:'Tourelle'}},
 'b:los':{setrange:{n:'Définir la portée',t:'Portée de vision = {0}',p:[P('Portée (px)',X_,'400')],cat:'Vision'}}
};
var PROVNAME={minimap:'Mini-carte',road3d:'Route 3D',common:'Commun',sprite:'Sprite',text:'Texte',tilemap:'Tuiles',particles:'Particules',button:'Bouton',array:'Tableau',dict:'Dictionnaire',leaderboard:'Classement',bar:'Barre'};
function providersOf(o,objs){if(o.kind==='array'||o.kind==='dict')return[o.kind];if(o.kind==='family'){var ms=(o.members||[]).map(function(id){return (objs||[]).find(function(x){return x.id===id;});}).filter(Boolean);if(!ms.length)return['common'];var r=providersOf(ms[0]);ms.slice(1).forEach(function(m){var q=providersOf(m);r=r.filter(function(p){return q.indexOf(p)>=0;});});return r;}var l=['common',['text','sprite','tilemap','particles','button','leaderboard','bar','road3d','minimap'].indexOf(o.kind)>=0?o.kind:null].filter(Boolean);(o.behaviors||[]).forEach(function(b){l.push('b:'+b.type);});return l;}

/* ================= EXPRESSIONS ================= */
var EXF={
 random:function(a,b){a=+a||0;return b===undefined?Math.random()*a:a+Math.random()*(b-a);},
 floor:Math.floor,ceil:Math.ceil,round:Math.round,abs:Math.abs,sqrt:Math.sqrt,
 min:function(){return Math.min.apply(null,arguments);},max:function(){return Math.max.apply(null,arguments);},
 sin:function(d){return Math.sin(d*Math.PI/180);},cos:function(d){return Math.cos(d*Math.PI/180);},
 clamp:function(v,a,b){return Math.max(a,Math.min(b,v));},lerp:function(a,b,t){return a+(b-a)*t;},
 choose:function(){return arguments[Math.random()*arguments.length|0];},
 int:function(v){return parseInt(v,10)||0;},float:function(v){return parseFloat(v)||0;},str:function(v){return String(v);},
 len:function(s){return String(s).length;},distance:function(a,b,c,d){return Math.hypot(c-a,d-b);},
 angle:function(a,b,c,d){return Math.atan2(d-b,c-a)*180/Math.PI;},
 zeropad:function(n,d){n=String(Math.floor(n));while(n.length<d)n='0'+n;return n;},
 upper:function(s){return String(s).toUpperCase();},highscore:function(){return 0;},highscorename:function(){return '';},highscorecount:function(){return 0;},call:function(){return 0;},__a:function(){return Array.prototype.slice.call(arguments);},storage:function(k,d){return d===undefined?0:d;},loopindex:function(){return 0;},lower:function(s){return String(s).toLowerCase();}
};
var SYSEXP=['newline','PlayerName','LastRank','CurValue','CurKey','CameraZoom','CheckpointX','CheckpointY','DialogueChoice','DialogueChoiceText','TimeScale','loopindex','dt','time','TickCount','LayoutWidth','LayoutHeight','LayoutName','ViewportWidth','ViewportHeight','ScrollX','ScrollY','pi'];
var MOUSEEXP=['X','Y','ScreenX','ScreenY'];
var OBJMETH=['ImagePointX','ImagePointY','TileAt','TileAtXY','ColumnAt','RowAt','At','IndexOf','Get','Has'];
var METHARGS={At:[1,1],IndexOf:[1,1],Get:[1,1],Has:[1,1],ImagePointX:[0,1],ImagePointY:[0,1],TileAt:[2,2],TileAtXY:[2,2],ColumnAt:[1,1],RowAt:[1,1]};
var TILETYPES=[['s','Solide'],['n','Décor (aucune)'],['j','Traversable'],['r45','Pente ↗'],['l45','Pente ↘'],['r1','Douce ↗ bas'],['r2','Douce ↗ haut'],['l2','Douce ↘ haut'],['l1','Douce ↘ bas']];
function tilePoly(t,C,H){switch(t){case 'r45':return[[0,H],[C,0],[C,H]];case 'l45':return[[0,0],[C,H],[0,H]];case 'r1':return[[0,H],[C,H/2],[C,H]];case 'r2':return[[0,H/2],[C,0],[C,H],[0,H]];case 'l2':return[[0,0],[C,H/2],[C,H],[0,H]];case 'l1':return[[0,H/2],[C,H],[0,H]];}return null;}
var PTPRESETS={
 dust:{n:'Poussière',mode:'burst',burst:8,rate:20,angle:-90,spray:150,speed:60,speedRand:30,gravity:140,drag:2,life:.35,lifeRand:.1,size0:4,size1:1,sizeRand:1,fade:true,colors:['#e8d9b0','#c9b48a'],shape:'square',blend:'normal',spin:0,destroyEnd:true},
 sparks:{n:'Étincelles',mode:'burst',burst:14,rate:30,angle:-90,spray:360,speed:110,speedRand:50,gravity:260,drag:1,life:.45,lifeRand:.15,size0:4,size1:0,sizeRand:1,fade:true,colors:['#ffe08a','#ffd84d','#ffffff'],shape:'square',blend:'add',spin:0,destroyEnd:true},
 explosion:{n:'Explosion',mode:'burst',burst:40,rate:30,angle:0,spray:360,speed:180,speedRand:90,gravity:80,drag:3,life:.6,lifeRand:.25,size0:9,size1:2,sizeRand:3,fade:true,colors:['#ffd84d','#ff8a3d','#e8566a','#5f574f'],shape:'circle',blend:'normal',spin:0,destroyEnd:true},
 smoke:{n:'Fumée',mode:'cont',burst:10,rate:14,angle:-90,spray:30,speed:40,speedRand:15,gravity:-20,drag:.5,life:1.6,lifeRand:.4,size0:6,size1:18,sizeRand:3,fade:true,colors:['#c2c3c7','#9aa2b2'],shape:'circle',blend:'normal',spin:0,destroyEnd:true},
 fire:{n:'Feu',mode:'cont',burst:10,rate:50,angle:-90,spray:30,speed:70,speedRand:30,gravity:-60,drag:1,life:.6,lifeRand:.2,size0:10,size1:1,sizeRand:3,fade:true,colors:['#ffd84d','#ff8a3d','#e8566a'],shape:'circle',blend:'add',spin:0,destroyEnd:true},
 rain:{n:'Pluie',mode:'cont',burst:10,rate:90,angle:100,spray:4,speed:520,speedRand:60,gravity:0,drag:0,life:1.2,lifeRand:.1,size0:2,size1:2,sizeRand:0,fade:false,colors:['#9fd3ff','#cfe9ff'],shape:'square',blend:'normal',spin:0,destroyEnd:true,width:900},
 confetti:{n:'Confettis',mode:'burst',burst:60,rate:30,angle:-90,spray:70,speed:340,speedRand:120,gravity:420,drag:1.2,life:1.8,lifeRand:.5,size0:5,size1:5,sizeRand:1,fade:false,colors:['#ff4d6a','#ffd84d','#4da3ff','#3ddc84','#9b7bff'],shape:'square',blend:'normal',spin:540,destroyEnd:true}};
function ptDefaults(){var o=Object.assign({},PTPRESETS.dust);delete o.n;return o;}
var OBJEXP=['X','Y','Width','Height','Angle','Opacity','Count','UID','AnimationName','AnimationFrame','Text','VectorX','VectorY','IsOnFloor','ParticleCount','Columns','Rows','Length','KeyCount','AsJSON','Value','MaxValue','PathPoint','Speed','Distance','Lap','LapTime','LastLap','BestLap','Rank','Rivals','PlayerX','Curve','SkyOffset','Progress','Steer','Horizon'];
function tokenize(s){
 var re=/\s*(?:(\d+(?:\.\d+)?|\.\d+)|"((?:[^"]|"")*)"|([A-Za-z_À-ɏ][\wÀ-ɏ]*)|(<>|<=|>=|!=|==|[-+*\/%^&|=<>(),.?:]))/y;
 var out=[],m,i=0;s=String(s);
 while(i<s.length){if(/^\s*$/.test(s.slice(i)))break;re.lastIndex=i;m=re.exec(s);if(!m){var ch=s.slice(i).trim()[0];throw new Error(ch==='"'?'Guillemet non fermé':'Caractère inattendu : « '+ch+' »');}i=re.lastIndex;
  if(m[1]!==undefined)out.push({t:'n',v:m[1]});else if(m[2]!==undefined)out.push({t:'s',v:m[2].replace(/""/g,'"')});else if(m[3]!==undefined)out.push({t:'i',v:m[3]});else out.push({t:'o',v:m[4]});}
 return out;
}
function parse(tk){
 var pos=0;function pk(){return tk[pos];}function nx(){return tk[pos++];}
 function isOp(v){var t=pk();return t&&t.t==='o'&&t.v===v;}
 function isW(w){var t=pk();return t&&t.t==='i'&&t.v.toLowerCase()===w;}
 function expect(v){if(!isOp(v))throw new Error('« '+v+' » attendu');pos++;}
 function ter(){var c=or();if(isOp('?')){pos++;var a=ter();expect(':');var b=ter();return '(('+c+')?('+a+'):('+b+'))';}return c;}
 function or(){var l=and();while(isW('ou')||isW('or')||isOp('|')){pos++;l='((('+l+')||('+and()+'))?1:0)';}return l;}
 function and(){var l=comp();while(isW('et')||isW('and')){pos++;l='((('+l+')&&('+comp()+'))?1:0)';}return l;}
 function comp(){var l=cat(),t=pk();if(t&&t.t==='o'&&/^(=|==|<>|!=|<|>|<=|>=)$/.test(t.v)){pos++;var r=cat(),op={'=':'==','==':'==','<>':'!=','!=':'!='}[t.v]||t.v;return '((('+l+')'+op+'('+r+'))?1:0)';}return l;}
 function cat(){var l=add();while(isOp('&')){pos++;l='(""+('+l+')+('+add()+'))';}return l;}
 function add(){var l=mul();while(isOp('+')||isOp('-')){var o=nx().v;l='('+l+o+mul()+')';}return l;}
 function mul(){var l=un();while(isOp('*')||isOp('/')||isOp('%')){var o=nx().v;l='('+l+o+un()+')';}return l;}
 function un(){if(isOp('-')){pos++;return '(-'+un()+')';}if(isOp('+')){pos++;return un();}if(isW('non')||isW('not')){pos++;return '(('+un()+')?0:1)';}return pw();}
 function pw(){var b=prim();if(isOp('^')){pos++;return 'Math.pow('+b+','+un()+')';}return b;}
 function prim(){var t=nx();if(!t)throw new Error('Expression incomplète');
  if(t.t==='n')return t.v;if(t.t==='s')return JSON.stringify(t.v);
  if(t.t==='o'&&t.v==='('){var e=ter();expect(')');return '('+e+')';}
  if(t.t==='i'){
   if(isOp('(')){pos++;var a=[];if(!isOp(')')){a.push(ter());while(isOp(',')){pos++;a.push(ter());}}expect(')');var fn=t.v.toLowerCase();if(!EXF[fn])throw new Error('Fonction inconnue : '+t.v+'()');return 'F.'+fn+'('+a.join(',')+')';}
   if(isOp('.')){pos++;var p=nx();if(!p||p.t!=='i')throw new Error('Propriété attendue après « '+t.v+'. »');if(isOp('(')){pos++;var ma=[];if(!isOp(')')){ma.push(ter());while(isOp(',')){pos++;ma.push(ter());}}expect(')');return 'E.m('+JSON.stringify(t.v)+','+JSON.stringify(p.v)+',['+ma.join(',')+'])';}return 'E.o('+JSON.stringify(t.v)+','+JSON.stringify(p.v)+')';}
   return 'E.g('+JSON.stringify(t.v)+')';}
  throw new Error('« '+t.v+' » inattendu');}
 var js=ter();if(pos<tk.length)throw new Error('« '+tk[pos].v+' » inattendu');return js;
}
var XC={};
function compile(src){var k=String(src==null?'':src),f=XC[k];if(f)return f;f=new Function('E','F','return '+parse(tokenize(k))+';');XC[k]=f;return f;}
function checkExpr(src,P,extra){
 var f=compile(src);var objs={},gl={};(P.objects||[]).forEach(function(o){objs[o.name]=o;});(P.globals||[]).forEach(function(g){gl[g.name]=1;});(extra||[]).forEach(function(n){gl[n]=1;});
 f({o:function(n,p){if(n==='Mouse'){if(MOUSEEXP.indexOf(p)<0)throw new Error('Mouse.'+p+' n’existe pas (X, Y, ScreenX, ScreenY)');return 0;}if(n!=='Self'&&!objs[n])throw new Error('Objet inconnu : '+n);var o=objs[n];var hasV=function(ob){return ob&&(ob.vars||[]).some(function(v){return v.name===p;});};if(OBJEXP.indexOf(p)<0&&!hasV(o)&&!(o&&(P.objects||[]).some(function(f){return f.kind==='family'&&(f.members||[]).indexOf(o.id)>=0&&hasV(f);}))&&n!=='Self')throw new Error(n+'.'+p+' n’existe pas');return 0;},m:function(n,p,a){if(n!=='Self'&&!objs[n])throw new Error('Objet inconnu : '+n);if(OBJMETH.indexOf(p)<0)throw new Error(n+'.'+p+'() n’existe pas ('+OBJMETH.join(', ')+')');var ma=METHARGS[p];if(a.length<ma[0]||a.length>ma[1])throw new Error(n+'.'+p+'() attend '+(ma[0]===ma[1]?ma[0]:'au plus '+ma[1])+' paramètre'+(ma[1]>1?'s':''));var o=objs[n];if(p.indexOf('ImagePoint')===0&&o&&a.length&&typeof a[0]==='string'&&a[0]!==''&&!(o.points||[]).some(function(q){return q.name===a[0];}))throw new Error('« '+n+' » n’a pas de point d’image « '+a[0]+' »');return 0;},g:function(n){if(!gl[n]&&SYSEXP.indexOf(n)<0)throw new Error('Variable inconnue : '+n);return 0;}},EXF);
 return true;
}

/* ================= ART HELPERS ================= */
function rr(c,x,y,w,h,r){r=Math.min(r,w/2,h/2);c.beginPath();c.moveTo(x+r,y);c.arcTo(x+w,y,x+w,y+h,r);c.arcTo(x+w,y+h,x,y+h,r);c.arcTo(x,y+h,x,y,r);c.arcTo(x,y,x+w,y,r);c.closePath();}
var FONTS={sans:'system-ui,-apple-system,"Segoe UI",Roboto,sans-serif',round:'ui-rounded,"SF Pro Rounded","Nunito","Varela Round",system-ui,sans-serif',mono:'ui-monospace,"SF Mono",Menlo,Consolas,monospace',serif:'Georgia,"Times New Roman",serif'};
function patternFor(c,img){if(!img._pats)img._pats=[];for(var i=0;i<img._pats.length;i++)if(img._pats[i][0]===c)return img._pats[i][1];var p=c.createPattern(img,'repeat');img._pats.push([c,p]);return p;}
var TMC=typeof WeakMap!=='undefined'?new WeakMap():null,TMV=TMC?new WeakMap():null;
function tmDirty(data){if(TMV&&data)TMV.set(data,(TMV.get(data)||0)+1);}
function tmCanvas(o,D,img){if(!TMC||!img||!img.complete||!img.naturalWidth||!D.data)return null;var tw=o.tw||16,th=o.th||16,ver=TMV.get(D.data)||0,e=TMC.get(D.data);
 if(e&&e.ver===ver&&e.img===img&&e.cols===D.cols&&e.rows===D.rows&&e.tw===tw&&e.th===th)return e.cv;
 var cv=e?e.cv:document.createElement('canvas');cv.width=Math.max(1,D.cols*tw);cv.height=Math.max(1,D.rows*th);var x=cv.getContext('2d');x.clearRect(0,0,cv.width,cv.height);var tc=Math.max(1,Math.floor(img.naturalWidth/tw));
 for(var r=0;r<D.rows;r++)for(var k=0;k<D.cols;k++){var v=D.data[r*D.cols+k];if(v==null||v<0)continue;x.drawImage(img,(v%tc)*tw,Math.floor(v/tc)*th,tw,th,k*tw,r*th,tw,th);}
 TMC.set(D.data,{cv:cv,ver:ver,img:img,cols:D.cols,rows:D.rows,tw:tw,th:th});return cv;}
function ptEmit(P,st,x,y,ang,mir,n){var cols=P.colors&&P.colors.length?P.colors:['#ffffff'],base=(P.angle||0)+(ang||0);if(mir)base=180-base;
 for(var k=0;k<n&&st.parts.length<3000;k++){var a=(base+(Math.random()-.5)*(P.spray||0))*Math.PI/180,sp=Math.max(0,(P.speed||0)+(Math.random()*2-1)*(P.speedRand||0)),lf=Math.max(.05,(P.life||1)+(Math.random()*2-1)*(P.lifeRand||0)),sr=(Math.random()*2-1)*(P.sizeRand||0),wx=P.width?(Math.random()-.5)*P.width:0;
  st.parts.push({x:x+wx,y:y,vx:Math.cos(a)*sp,vy:Math.sin(a)*sp,age:0,life:lf,s0:Math.max(0,(P.size0==null?4:P.size0)+sr),s1:Math.max(0,(P.size1==null?0:P.size1)+sr*.5),c:cols[Math.random()*cols.length|0],rot:P.spin?Math.random()*360:0,spin:(P.spin||0)*(Math.random()<.5?-1:1)});}}
function ptStep(P,st,dt,x,y,ang,mir){
 if(P.mode==='burst'){if(!st.started){st.started=true;ptEmit(P,st,x,y,ang,mir,P.burst||0);}}
 else if(st.active!==false){st.acc=(st.acc||0)+(st.rate!=null?st.rate:(P.rate||0))*dt;var n=Math.floor(st.acc);st.acc-=n;if(n)ptEmit(P,st,x,y,ang,mir,Math.min(n,300));}
 var g=P.gravity||0,dr=Math.max(0,1-(P.drag||0)*dt),L=st.parts;
 for(var k=L.length-1;k>=0;k--){var q=L[k];q.age+=dt;if(q.age>=q.life){L.splice(k,1);continue;}q.vx*=dr;q.vy=q.vy*dr+g*dt;q.x+=q.vx*dt;q.y+=q.vy*dt;if(q.spin)q.rot+=q.spin*dt;}}
function drawParts(c,o,i,getImg){ptDraw(c,o.pt||{},i.ps,getImg);}
var PTSOFT={};function ptSoft(col){var cv=PTSOFT[col];if(cv)return cv;cv=document.createElement('canvas');cv.width=cv.height=64;var x=cv.getContext('2d'),g=x.createRadialGradient(32,32,0,32,32,32);x.fillStyle=col;x.fillRect(0,0,1,1);var d=x.getImageData(0,0,1,1).data;x.clearRect(0,0,1,1);var a=d[3]/255,rgb=d[0]+','+d[1]+','+d[2];g.addColorStop(0,'rgba('+rgb+','+a+')');g.addColorStop(.45,'rgba('+rgb+','+(a*.6)+')');g.addColorStop(1,'rgba('+rgb+',0)');x.fillStyle=g;x.fillRect(0,0,64,64);PTSOFT[col]=cv;return cv;}
function ptDraw(c,P,st,getImg){var L=st.parts;if(!L.length)return;var img=null;if(P.shape==='image'&&st.img)img=getImg(st.img);
 if(P.blend==='add')c.globalCompositeOperation='lighter';var ba=c.globalAlpha;
 for(var k=0;k<L.length;k++){var q=L[k],f=q.age/q.life,sz=q.s0+(q.s1-q.s0)*f;if(sz<=0)continue;c.globalAlpha=ba*(P.fade!==false?Math.max(0,1-f):1);
  if(img&&img.complete&&img.naturalWidth){c.save();c.translate(q.x,q.y);if(q.rot)c.rotate(q.rot*Math.PI/180);c.drawImage(img,-sz/2,-sz/2,sz,sz*img.naturalHeight/img.naturalWidth);c.restore();continue;}
  if(P.shape==='soft'){c.drawImage(ptSoft(q.c),q.x-sz/2,q.y-sz/2,sz,sz);continue;}
  c.fillStyle=q.c;if(P.shape==='circle'){c.beginPath();c.arc(q.x,q.y,sz/2,0,6.2832);c.fill();}
  else if(q.rot){c.save();c.translate(q.x,q.y);c.rotate(q.rot*Math.PI/180);c.fillRect(-sz/2,-sz/2,sz,sz);c.restore();}else c.fillRect(q.x-sz/2,q.y-sz/2,sz,sz);}
 c.globalAlpha=ba;c.globalCompositeOperation='source-over';}
var TCV=null;function tintCv(w,h){if(!TCV)TCV=document.createElement('canvas');if(TCV.width<w)TCV.width=w;if(TCV.height<h)TCV.height=h;return TCV;}
var ISTYLE={button:[['bg','Fond','color'],['fg','Texte','color'],['hoverBg','Fond (survol)','color'],['hoverFg','Texte (survol)','color'],['borderColor','Bordure','color'],['size','Taille du texte','num']],text:[['color','Couleur','color'],['outlineColor','Contour','color'],['size','Taille','num']],bar:[['fg','Barre','color'],['bg','Fond','color'],['lowColor','Si presque vide','color']],leaderboard:[['fg','Texte','color'],['accent','Titre / nouveau score','color'],['bg','Fond','color']]};
function lighten(h,k){var m=/^#?([0-9a-f]{6})$/i.exec(String(h||''));if(!m)return h;var n=parseInt(m[1],16),f=function(v){return Math.round(v+(255-v)*k);};return '#'+[f(n>>16&255),f(n>>8&255),f(n&255)].map(function(v){return ('0'+v.toString(16)).slice(-2);}).join('');}

/* ================= ROUTE PSEUDO-3D (vue arrière façon OutRun / rallye) ================= */
var RDLK=null;
var RD_TRACK='40 0 0\n60 3 20\n40 0 -20\n80 -4 0\n50 0 40\n60 5 -40\n40 -2 0\n70 0 0';
function rdNum(v,d){v=+v;return isFinite(v)?v:d;}
function rdEase(a,b,t){return a+(b-a)*(-Math.cos(t*Math.PI)/2+.5);}
function rdEaseIn(a,b,t){return a+(b-a)*t*t;}
function rdEaseOut(a,b,t){return a+(b-a)*(1-(1-t)*(1-t));}
function rdBuild(o){var key=[o.track,o.segLen,o.rumble,o.decor,o.decorEvery,o.decor2,o.decor2Every,o.seed,o.autoSigns].join('|');if(o._rd&&o._rdk===key)return o._rd;
 var secs=[],L=Math.max(20,rdNum(o.segLen,200)),rum=Math.max(1,Math.round(rdNum(o.rumble,3))),segs=[],lastY=function(){return segs.length?segs[segs.length-1].p2.y:0;};
 function add(curve,y){var n=segs.length;segs.push({i:n,p1:{y:lastY(),z:n*L},p2:{y:y,z:(n+1)*L},curve:curve,col:Math.floor(n/rum)%2,spr:[],cars:[]});}
 String(o.track||RD_TRACK).split(/\n|;/).forEach(function(line){var q=line.trim().split(/[\s,]+/).map(Number);if(!q.length||!isFinite(q[0])||q[0]<=0)return;var n=Math.min(2000,Math.round(q[0])),c=rdNum(q[1],0),h=rdNum(q[2],0),en=Math.floor(n/4),lv=en,hold=n-en-lv,y0=lastY(),y1=y0+h*L,tot=n,k;secs.push({s:segs.length,n:n,c:c});
  for(k=0;k<en;k++)add(rdEaseIn(0,c,k/en),rdEase(y0,y1,k/tot));for(k=0;k<hold;k++)add(c,rdEase(y0,y1,(en+k)/tot));for(k=0;k<lv;k++)add(rdEaseOut(c,0,k/lv),rdEase(y0,y1,(en+hold+k)/tot));});
 while(segs.length<40)add(0,lastY());
 /* back to the starting height smoothly */
 var yEnd=lastY();if(Math.abs(yEnd)>1){var m=Math.min(60,Math.floor(segs.length/3)),y0b=segs[segs.length-m].p1.y;for(var k2=0;k2<m;k2++){var sg=segs[segs.length-m+k2];sg.p1.y=rdEase(y0b,0,k2/m);sg.p2.y=rdEase(y0b,0,(k2+1)/m);}}
 var r=(function(s0){return function(){s0=(s0*16807)%2147483647;return (s0-1)/2147483646;};})(Math.max(1,Math.round(rdNum(o.seed,7))));
 /* automatic curve signs: arrows toward the bend, 1 to 3 chevrons by severity, on the outside of the curve */
 if(o.autoSigns!==false)secs.forEach(function(sc,k){var ac=Math.abs(sc.c);if(ac<1)return;var prev=secs[k-1];if(prev&&Math.sign(prev.c)===Math.sign(sc.c)&&Math.abs(prev.c)>=ac)return;
  var dir=sc.c>0?1:-1,lvl=ac<3?1:ac<5?2:3,side=-dir*1.32,add=function(at){if(at<4||at>=segs.length)return;segs[at].spr=segs[at].spr.filter(function(q){return Math.sign(q.off)!==Math.sign(side);});segs[at].spr.push({id:'#sign',dir:dir,lvl:lvl,off:side});};
  add(sc.s-26);add(sc.s-14);for(var j=4;j<Math.min(sc.n*.6,48);j+=11)add(sc.s+j);});
 [[o.decor,rdNum(o.decorEvery,6)],[o.decor2,rdNum(o.decor2Every,17)]].forEach(function(d){if(!d[0]||d[1]<=0)return;for(var n=8;n<segs.length;n+=Math.max(1,Math.round(d[1]))){var side=r()<.5?-1:1;segs[n].spr.push({id:d[0],off:side*(1.25+r()*1.6)});if(r()<.5)segs[n].spr.push({id:d[0],off:-side*(1.3+r()*2.2)});}});
 /* top-down shape for the minimap: integrate the curves, then spread the closing error so the loop closes */
 var tot=0;segs.forEach(function(sg){tot+=sg.curve*.55;});var dir0=tot<0?-1:1,extra=(dir0*360-tot)/segs.length;
 var mp=[],hx=0,hy=0,ang=-90;segs.forEach(function(sg){ang+=sg.curve*.55+extra;hx+=Math.cos(ang*Math.PI/180);hy+=Math.sin(ang*Math.PI/180);mp.push([hx,hy]);});
 var ex=hx,ey=hy,nn=mp.length;mp.forEach(function(q,k){q[0]-=ex*(k+1)/nn;q[1]-=ey*(k+1)/nn;});
 var mxs=mp.map(function(q){return q[0];}),mys=mp.map(function(q){return q[1];}),ml=Math.min.apply(null,mxs),mr=Math.max.apply(null,mxs),mt=Math.min.apply(null,mys),mb=Math.max.apply(null,mys);
 o._rd={segs:segs,L:L,len:segs.length*L,map:mp,mb:{l:ml,r:mr,t:mt,b:mb}};o._rdk=key;return o._rd;}
function rdProject(p,cx,cy,cz,depth,W,H,rw){var x=-cx,y=p.y-cy,z=p.z-cz;p.cz=z;p.s=depth/z;p.X=Math.round(W/2+p.s*(x+p.wx)*W/2);p.Y=Math.round(H/2-p.s*y*H/2);p.W=Math.round(p.s*rw*W/2);}
function rdQuad(c,x1,y1,w1,x2,y2,w2,col){c.fillStyle=col;c.beginPath();c.moveTo(x1-w1,y1);c.lineTo(x2-w2,y2);c.lineTo(x2+w2,y2);c.lineTo(x1+w1,y1);c.closePath();c.fill();}
var RDSIGN={};function rdSignImg(dir,lvl){var k=dir+'_'+lvl;if(RDSIGN[k])return RDSIGN[k];var cv=document.createElement('canvas');cv.width=26;cv.height=34;var c=cv.getContext('2d');c.fillStyle='#4a5263';c.fillRect(12,18,2,16);c.fillStyle='#1b1f2a';c.fillRect(0,0,26,19);c.fillStyle='#ffd84d';c.fillRect(1,1,24,17);c.fillStyle='#1b1f2a';var w=lvl===1?7:lvl===2?13:19,x0=13-w/2;for(var n=0;n<lvl;n++){var bx=x0+n*6;for(var j=0;j<5;j++){var px=dir>0?bx+j:bx+4-j;c.fillRect(Math.round(px),4+j,2,1);c.fillRect(Math.round(px),13-j,2,1);}}RDSIGN[k]={im:cv,def:{defW:50}};return RDSIGN[k];}
function rdImg(id,lk,getImg){var d=lk&&lk(id);if(!d||d.kind!=='sprite'||!d.anims||!d.anims[0])return null;var im=getImg(d.anims[0].frames[0]);return im&&im.width?{im:im,def:d}:null;}
function rdDraw(c,o,i,getImg){var R=rdBuild(o),S=R.segs,N=S.length,W=i.w,H=i.h,st=i.rd||{pos:0,x:0,cars:[]},lk=i.lk||RDLK;
 var L=R.L,rw=rdNum(o.roadWidth,2000),camH=rdNum(o.camHeight,1000),depth=1/Math.tan(rdNum(o.fov,100)/2*Math.PI/180),pZ=camH*depth,dd=Math.min(N-1,Math.max(20,Math.round(rdNum(o.drawDist,160)))),lanes=Math.max(1,Math.round(rdNum(o.lanes,2)));
 var pos=((st.pos%R.len)+R.len)%R.len,base=S[Math.floor(pos/L)%N],bp=(pos%L)/L,ps=S[Math.floor((pos+pZ)/L)%N],pp=((pos+pZ)%L)/L,pY=ps.p1.y+(ps.p2.y-ps.p1.y)*pp;
 c.save();c.translate(i.x,i.y);c.beginPath();c.rect(0,0,W,H);c.clip();
 var vis=[],maxy=H,x=0,dx=-(base.curve*bp),fogC=o.fog||'#9fd3f2',fogD=rdNum(o.fogDensity,4),k,sg,cx=st.x*rw;
 for(k=0;k<dd;k++){sg=S[(base.i+k)%N];var looped=sg.i<base.i,cz=pos-(looped?R.len:0);sg.fog=1/Math.exp((k/dd)*(k/dd)*fogD);sg.clip=maxy;
  sg.p1.wx=x;sg.p2.wx=x+dx;rdProject(sg.p1,cx,pY+camH,cz,depth,W,H,rw);rdProject(sg.p2,cx,pY+camH,cz,depth,W,H,rw);x+=dx;dx+=sg.curve;
  if(sg.p1.cz<=depth||sg.p2.Y>=sg.p1.Y||sg.p2.Y>=maxy)continue;
  vis.push(sg);maxy=sg.p1.Y;}
 if(i.rd)i.rd.hz=maxy;
 /* painter's order: far to near, so near segments cover the seams */
 for(k=vis.length-1;k>=0;k--){sg=vis[k];
  var a=sg.p1,b=sg.p2,alt=sg.col,start=sg.i<2;
  c.fillStyle=alt?(o.grassB||'#4fa847'):(o.grassA||'#58b84f');c.fillRect(0,b.Y,W,a.Y-b.Y+1);
  var r1=a.W/Math.max(6,2*lanes),r2=b.W/Math.max(6,2*lanes);rdQuad(c,a.X-a.W-r1,a.Y,r1,b.X-b.W-r2,b.Y,r2,alt?(o.rumbleB||'#f4f4f4'):(o.rumbleA||'#e8566a'));rdQuad(c,a.X+a.W+r1,a.Y,r1,b.X+b.W+r2,b.Y,r2,alt?(o.rumbleB||'#f4f4f4'):(o.rumbleA||'#e8566a'));
  rdQuad(c,a.X,a.Y,a.W,b.X,b.Y,b.W,alt?(o.roadB||'#6b6b73'):(o.roadA||'#727280'));
  if(start){/* checkered start / finish line */for(var ck=0;ck<10;ck++){var u0=ck/10*2-1,u1=(ck+1)/10*2-1;c.fillStyle=(ck+sg.i)%2?'#f4f4f4':'#1b1f2a';c.beginPath();c.moveTo(a.X+a.W*u0,a.Y);c.lineTo(b.X+b.W*u0,b.Y);c.lineTo(b.X+b.W*u1,b.Y);c.lineTo(a.X+a.W*u1,a.Y);c.closePath();c.fill();}}
  if(!alt&&o.lane!==''&&lanes>1){var l1=a.W/Math.max(32,8*lanes),l2=b.W/Math.max(32,8*lanes),lw1=a.W*2/lanes,lw2=b.W*2/lanes;for(var ln=1;ln<lanes;ln++)rdQuad(c,a.X-a.W+lw1*ln,a.Y,l1,b.X-b.W+lw2*ln,b.Y,l2,o.lane||'#f4f4f4');}
  if(sg.fog<1){c.globalAlpha=1-sg.fog;c.fillStyle=fogC;c.fillRect(0,b.Y,W,a.Y-b.Y+1);c.globalAlpha=1;}}
 /* sprites, from far to near */
 /* sizes: a sprite is drawn at its placed size (defW) at the player's distance; the decor can be enlarged */
 var unit=2*camH/W,dsc=rdNum(o.spriteScale,2);
 function spr(id,seg,off,isCar,sp){var r=id==='#sign'?rdSignImg(sp.dir,sp.lvl):rdImg(id,lk,getImg);if(!r)return;var a=seg.p1;if(!a.s||a.cz<=depth)return;var ww=(r.def.defW||r.im.width)*unit*(isCar?1:dsc),dw=ww*a.s*W/2,dh=dw*r.im.height/r.im.width,dxp=a.X+a.s*off*rw*W/2-dw/2,dyp=a.Y-dh;
  var clipH=seg.clip?Math.max(0,dyp+dh-seg.clip):0;if(clipH>=dh||dw<1)return;c.globalAlpha=Math.min(1,.3+seg.fog);c.drawImage(r.im,0,0,r.im.width,r.im.height-r.im.height*clipH/dh,dxp,dyp,dw,dh-clipH);c.globalAlpha=1;}
 c.imageSmoothingEnabled=false;
 for(k=dd-1;k>0;k--){sg=S[(base.i+k)%N];for(var q=0;q<sg.spr.length;q++)spr(sg.spr[q].id,sg,sg.spr[q].off,false,sg.spr[q]);var cs=st.cars||[];for(var q2=0;q2<cs.length;q2++){var car=cs[q2];if(Math.floor(car.z/L)%N===sg.i)spr(car.id,sg,car.off,true);}}
 if(o.minimap)rdMinimap(c,o,R,st,W,H,pZ);
 c.restore();}
function rdMinimap(c,o,R,st,W,H,pZ){var sz=Math.max(60,rdNum(o.mmSize,130)),pad=12,cr=o.mmCorner||'tr',x0=cr.charAt(1)==='l'?pad:W-sz-pad,y0=cr.charAt(0)==='t'?pad:H-sz-pad,b=R.mb,sc=(sz-22)/Math.max(1e-6,Math.max(b.r-b.l,b.b-b.t)),ox=x0+sz/2-(b.l+b.r)/2*sc,oy=y0+sz/2-(b.t+b.b)/2*sc,mp=R.map,N=mp.length,P=function(k){var q=mp[((k%N)+N)%N];return [ox+q[0]*sc,oy+q[1]*sc];};
 c.save();c.globalAlpha=1;c.fillStyle='rgba(15,18,26,.72)';c.beginPath();if(c.roundRect)c.roundRect(x0,y0,sz,sz,10);else c.rect(x0,y0,sz,sz);c.fill();c.strokeStyle='rgba(255,255,255,.25)';c.lineWidth=1;c.stroke();
 c.lineJoin='round';c.lineCap='round';for(var pass=0;pass<2;pass++){c.beginPath();for(var k=0;k<=N;k+=2){var p=P(k);if(k)c.lineTo(p[0],p[1]);else c.moveTo(p[0],p[1]);}c.closePath();c.strokeStyle=pass?(o.roadA||'#b48a52'):'rgba(0,0,0,.6)';c.lineWidth=pass?3:6;c.stroke();}
 var s0=P(0);c.fillStyle='#ffffff';c.fillRect(s0[0]-3,s0[1]-3,6,6);
 (st.cars||[]).forEach(function(car){var p=P(Math.floor(car.z/R.L));c.fillStyle=car.rival?'#4da3ff':'#c2c3c7';c.beginPath();c.arc(p[0],p[1],3.2,0,6.283);c.fill();});
 var me=P(Math.floor(((st.pos+pZ)%R.len+R.len)%R.len/R.L));c.fillStyle='#ff3b4f';c.strokeStyle='#ffffff';c.lineWidth=1.5;c.beginPath();c.arc(me[0],me[1],4.6,0,6.283);c.fill();c.stroke();c.restore();}

/* ================= MINI-CARTE (radar ou niveau entier) ================= */
var MMSRC=null;
function mmDraw(c,o,i){var src=(i.mmSrc||MMSRC);var D=src?src(i):null,W=i.w,H=i.h,round=o.shape!=='rect';
 c.save();c.translate(i.x,i.y);c.beginPath();if(round)c.ellipse(W/2,H/2,W/2,H/2,0,0,6.2832);else if(c.roundRect)c.roundRect(0,0,W,H,Math.min(12,W/8));else c.rect(0,0,W,H);c.save();c.clip();
 var ba=(o.bgAlpha==null?75:o.bgAlpha)/100;c.globalAlpha*=ba;c.fillStyle=o.bg||'#0b0d12';c.fillRect(0,0,W,H);c.globalAlpha/=ba;
 if(D){var fo=null,dots=(o.dots||[]).filter(function(d){return d&&d.obj;});if(o.mode!=='level'&&o.follow)D.items.some(function(it){if(it.id===o.follow){fo=it;return true;}});
  var cx,cy,sc;if(fo){var Rr=Math.max(50,+(i.mmR||o.radius)||500);cx=fo.x;cy=fo.y;sc=Math.min(W,H)/2/Rr;}else{sc=Math.min(W/Math.max(1,D.w),H/Math.max(1,D.h));cx=D.w/2;cy=D.h/2;}
  var X=function(v){return W/2+(v-cx)*sc;},Y=function(v){return H/2+(v-cy)*sc;};
  if(o.solids!==false){c.fillStyle=o.solidColor||'#8b93a6';D.items.forEach(function(it){if(!it.solid)return;var x1=X(it.l),y1=Y(it.t),x2=X(it.r),y2=Y(it.b);if(x2<0||y2<0||x1>W||y1>H)return;c.fillRect(x1,y1,Math.max(1,x2-x1),Math.max(1,y2-y1));});}
  if(o.mode==='level'||!fo){c.strokeStyle='rgba(255,255,255,.18)';c.lineWidth=1;c.strokeRect(X(0)+.5,Y(0)+.5,D.w*sc-1,D.h*sc-1);}
  dots.forEach(function(d){var r=Math.max(1.5,+d.size||4),blink=d.blink&&Math.floor(Date.now()/300)%2;D.items.forEach(function(it){if(it.id!==d.obj||it===fo)return;var x=X(it.x),y=Y(it.y),m=r+2;if(o.mode!=='level'&&fo&&(x<-m||y<-m||x>W+m||y>H+m)){/* off radar: stick to the edge */var ang=Math.atan2(y-H/2,x-W/2),rr=Math.min(W,H)/2-r-3;x=W/2+Math.cos(ang)*rr;y=H/2+Math.sin(ang)*rr;}c.fillStyle=blink?'#ffffff':(d.color||'#ff3b4f');c.beginPath();c.arc(x,y,r,0,6.2832);c.fill();});});
  if(fo){var fx=X(fo.x),fy=Y(fo.y),a=(fo.angle||0)*Math.PI/180,ar=Math.max(5,Math.min(W,H)/18);c.save();c.translate(fx,fy);c.rotate(a);c.fillStyle=o.followColor||'#ffffff';c.strokeStyle='#1b1f2a';c.lineWidth=1.5;c.beginPath();c.moveTo(ar,0);c.lineTo(-ar*.7,ar*.65);c.lineTo(-ar*.35,0);c.lineTo(-ar*.7,-ar*.65);c.closePath();c.fill();c.stroke();c.restore();}}
 else{c.fillStyle='rgba(255,255,255,.5)';c.font='700 12px sans-serif';c.textAlign='center';c.textBaseline='middle';c.fillText('Mini-carte',W/2,H/2);}
 c.restore();c.lineWidth=Math.max(1,+o.borderW||2);c.strokeStyle=o.border||'#ffffff';c.globalAlpha*=.8;c.stroke();c.restore();}
function drawInst(c,o,i,getImg,editor){if(i.style){for(var sk in i.style){if(i.style[sk]!==''&&i.style[sk]!=null){o=Object.assign({},o,i.style);break;}}}
 c.save();c.globalAlpha*=(i.opacity==null?1:i.opacity);
 if(o.kind==='road3d'){rdDraw(c,o,i,getImg);c.restore();return;}
 if(o.kind==='minimap'){mmDraw(c,o,i);c.restore();return;}
 if(o.kind==='tilemap'){var D=i.tm||i,im=o.image&&getImg(o.image),ts=o.scale||1;c.translate(i.x,i.y);c.scale(ts,ts);var tcv=tmCanvas(o,D,im);if(tcv)c.drawImage(tcv,0,0);c.restore();return;}
 if(o.kind==='button'){var bt=i.btn||{},en=bt.en!==false,hv=en&&(bt.hover||bt.focus),pr=en&&bt.press,rad=Math.min(o.radius==null?12:o.radius,i.h/2,i.w/2);c.translate(i.x,i.y+(pr?1.5:0));if(!en)c.globalAlpha*=.5;
  if(bt.focus&&en){c.strokeStyle=o.focusColor||'#ffffff';c.lineWidth=3;c.globalAlpha*=.9;rr(c,-4,-4,i.w+8,i.h+8,rad+4);c.stroke();c.globalAlpha/=.9;}
  c.fillStyle=o.shadow===false?'transparent':'rgba(0,0,0,.28)';if(o.shadow!==false&&!pr){rr(c,0,3,i.w,i.h,rad);c.fill();}
  c.fillStyle=hv?(o.hoverBg||o.bg||'#ffa45e'):(o.bg||'#ff8a3d');rr(c,0,0,i.w,i.h,rad);c.fill();if(o.border){c.strokeStyle=o.borderColor||'#1b1f2a';c.lineWidth=o.border;rr(c,o.border/2,o.border/2,i.w-o.border,i.h-o.border,Math.max(0,rad-o.border/2));c.stroke();}
  var bfs=o.size||20,bff=FONTS[o.font]||FONTS.round,btx=String(i.text!=null?i.text:o.text||'');c.font='800 '+bfs+'px '+bff;var bmw=c.measureText(btx).width,bav=i.w-Math.max(16,rad);if(bmw>bav&&bmw>0){bfs=Math.max(8,bfs*bav/bmw);c.font='800 '+bfs+'px '+bff;}c.textAlign='center';c.textBaseline='middle';c.fillStyle=hv?(o.hoverFg||o.fg||'#1a0f07'):(o.fg||'#1a0f07');c.fillText(String(i.text!=null?i.text:(o.text||'')),i.w/2,i.h/2+1);c.restore();return;}
 if(o.kind==='bar'){var bv0=i.value!=null?+i.value:o.value!=null?+o.value:(o.max||100)*.7,B=i.bar||{v:bv0,max:o.max||100,shown:bv0,ghost:0,col:''},mxv=B.max||1,fr=Math.max(0,Math.min(1,B.shown/mxv)),gfr=Math.max(fr,Math.min(1,(B.ghost||0)/mxv)),br=Math.min(o.radius==null?6:o.radius,i.h/2,i.w/2),bd=o.border||0,dir=o.dir||'ltr',seg=Math.max(0,Math.floor(o.segments||0));
  c.translate(i.x,i.y);c.fillStyle=o.bg||'#1b1f2a';rr(c,0,0,i.w,i.h,br);c.fill();
  c.save();rr(c,bd,bd,i.w-bd*2,i.h-bd*2,Math.max(0,br-bd));c.clip();var iw=i.w-bd*2,ih=i.h-bd*2;
  var part=function(f,col){c.fillStyle=col;if(dir==='btt')c.fillRect(bd,bd+ih*(1-f),iw,ih*f);else if(dir==='rtl')c.fillRect(bd+iw*(1-f),bd,iw*f,ih);else c.fillRect(bd,bd,iw*f,ih);};
  if(gfr>fr)part(gfr,o.ghostColor||'#ffffff');var lo=o.lowColor&&fr<=(o.lowAt==null?.25:o.lowAt/100);part(fr,B.col||(lo?o.lowColor:(o.fg||'#e8566a')));
  c.fillStyle='rgba(255,255,255,.18)';if(dir==='btt')c.fillRect(bd,bd,Math.max(1,iw*.3),ih);else c.fillRect(bd,bd,iw,Math.max(1,ih*.32));
  if(seg>1){c.fillStyle=o.bg||'#1b1f2a';for(var sg=1;sg<seg;sg++){if(dir==='btt')c.fillRect(bd,bd+ih*sg/seg-1,iw,2);else c.fillRect(bd+iw*sg/seg-1,bd,2,ih);}}
  c.restore();if(bd){c.strokeStyle=o.borderColor||'#ffffff';c.lineWidth=bd;rr(c,bd/2,bd/2,i.w-bd,i.h-bd,Math.max(0,br-bd/2));c.stroke();}
  if(o.showText){var bfz=Math.max(8,Math.min(o.size||14,i.h-4)),vtx=String(o.format||'{valeur} / {max}').replace(/\{valeur\}|\{value\}/g,Math.round(B.v)).replace(/\{max\}/g,Math.round(mxv)).replace(/\{%\}/g,Math.round(B.v/mxv*100));c.font='800 '+bfz+'px '+(FONTS[o.font]||FONTS.round);c.textAlign='center';c.textBaseline='middle';c.lineWidth=3;c.strokeStyle='rgba(0,0,0,.55)';c.lineJoin='round';c.strokeText(vtx,i.w/2,i.h/2+1);c.fillStyle=o.textColor||'#ffffff';c.fillText(vtx,i.w/2,i.h/2+1);}
  c.restore();return;}
 if(o.kind==='leaderboard'){var rows=Math.max(1,o.rows||5),L=i.lb||(editor?[{n:'PIXEL',s:42},{n:'NOVA',s:35},{n:'ZED',s:28}]:[]),fsz=o.size||18,lf=FONTS[o.font]||FONTS.round,pad=Math.round(fsz*.7),hasT=o.showTitle!==false&&o.title,th=hasT?fsz*1.9:0,rh=(i.h-pad*2-th)/rows;
  c.translate(i.x,i.y);if(o.bgAlpha>0){c.globalAlpha*=o.bgAlpha/100;c.fillStyle=o.bg||'#1b1f2a';rr(c,0,0,i.w,i.h,Math.min(14,i.h/4));c.fill();c.globalAlpha/=o.bgAlpha/100;}
  c.textBaseline='middle';if(hasT){c.textAlign='center';c.font='800 '+Math.round(fsz*1.15)+'px '+lf;c.fillStyle=o.accent||'#ffd84d';c.fillText(o.title,i.w/2,pad+th/2-fsz*.15);}
  for(var r=0;r<rows;r++){var e=L[r],yy=pad+th+rh*r+rh/2,hi=e&&i.lbHi&&e.d===i.lbHi;c.font=(hi?'800 ':'700 ')+fsz+'px '+lf;c.fillStyle=hi?(o.accent||'#ffd84d'):(o.fg||'#ffffff');if(!e)c.globalAlpha*=.4;
   c.textAlign='left';c.fillText((r+1)+'.',pad,yy);c.fillText(e?String(e.n):'---',pad+fsz*2.2,yy);c.textAlign='right';c.fillText(e?String(e.s)+(o.suffix||''):'',i.w-pad,yy);if(!e)c.globalAlpha/=.4;}
  c.restore();return;}
 if(o.kind==='particles'){if(i.ps)drawParts(c,o,i,getImg);else if(editor){var P=o.pt||{},a0=((P.angle||0)+(i.angle||0))*Math.PI/180,sp=(P.spray||0)*Math.PI/360;c.translate(i.x,i.y);c.strokeStyle='#ffb84d';c.fillStyle='rgba(255,184,77,.18)';c.lineWidth=1.5;
   c.beginPath();c.arc(0,0,9,0,6.2832);c.fill();c.stroke();c.beginPath();if(sp<3.1){c.moveTo(0,0);c.arc(0,0,26,a0-sp,a0+sp);c.closePath();}else c.arc(0,0,26,0,6.2832);c.setLineDash([3,3]);c.stroke();c.setLineDash([]);
   c.beginPath();c.moveTo(0,0);c.lineTo(Math.cos(a0)*20,Math.sin(a0)*20);c.stroke();var cl=(P.colors||['#ffd84d']);cl.slice(0,4).forEach(function(col,k){c.fillStyle=col;c.fillRect(-7+k*4,-2,3,3);});}
  c.restore();return;}
 if(o.kind==='sprite'){
  var a=o.anims[i.anim||0]||o.anims[0],fr=a&&(a.frames[i.frame||0]||a.frames[0]),img=fr&&getImg(fr);
  c.translate(i.x,i.y);if(i.angle)c.rotate(i.angle*Math.PI/180);var sx=i.w/o.w,sy=i.h/o.h;c.scale(i.mirrored?-sx:sx,sy);
  var ta=(i.tint&&i.tint.a>0?i.tint.a:0),tf=(i.tf&&i.tf.t>0?i.tf.t/i.tf.d:0);
  if(img&&img.complete&&img.naturalWidth&&(ta||tf)){var tc=tintCv(o.w,o.h),tx=tc.getContext('2d');tx.globalCompositeOperation='source-over';tx.clearRect(0,0,o.w,o.h);tx.drawImage(img,0,0,o.w,o.h);tx.globalCompositeOperation='source-atop';if(ta){tx.globalAlpha=ta;tx.fillStyle=i.tint.c;tx.fillRect(0,0,o.w,o.h);}if(tf){tx.globalAlpha=tf;tx.fillStyle=i.tf.c;tx.fillRect(0,0,o.w,o.h);}tx.globalAlpha=1;tx.globalCompositeOperation='source-over';c.drawImage(tc,0,0,o.w,o.h,-o.ox,-o.oy,o.w,o.h);}
  else if(img&&img.complete&&img.naturalWidth)c.drawImage(img,-o.ox,-o.oy,o.w,o.h);else{c.fillStyle='rgba(255,138,61,.35)';c.fillRect(-o.ox,-o.oy,o.w,o.h);}
 }else if(o.kind==='tiled'){
  var im=o.image&&getImg(o.image);c.translate(i.x,i.y);if(i.angle)c.rotate(i.angle*Math.PI/180);var s=o.scale||1;c.scale(s,s);
  if(im&&im.complete&&im.naturalWidth){c.fillStyle=patternFor(c,im);c.fillRect(0,0,i.w/s,i.h/s);}else{c.fillStyle='rgba(120,140,170,.4)';c.fillRect(0,0,i.w/s,i.h/s);}
 }else if(o.kind==='text'){
  c.translate(i.x,i.y);if(i.angle)c.rotate(i.angle*Math.PI/180);
  var size=o.size||20;c.font=(o.bold?'800 ':'500 ')+size+'px '+(FONTS[o.font]||FONTS.sans);c.textBaseline='top';var al=o.align||'left';c.textAlign=al;
  var tx=al==='center'?i.w/2:al==='right'?i.w:0,lines=String(i.text!=null?i.text:(o.text||'')).split('\n');
  lines.forEach(function(L,n){var y=n*size*1.2;if(o.outline){c.lineJoin='round';c.lineWidth=Math.max(2,size*.16);c.strokeStyle=o.outlineColor||'#1b1f2a';c.strokeText(L,tx,y);}c.fillStyle=o.color||'#ffffff';c.fillText(L,tx,y);});
 }
 c.restore();
}
function drawBox(o,i){
 if(o.kind==='particles')return{l:i.x-16,t:i.y-16,r:i.x+16,b:i.y+16};
 if(o.kind!=='sprite')return{l:i.x,t:i.y,r:i.x+i.w,b:i.y+i.h};
 var sx=i.w/o.w,sy=i.h/o.h,l=-o.ox*sx,r=(o.w-o.ox)*sx;if(i.mirrored){var q=l;l=-r;r=-q;}
 return{l:i.x+l,r:i.x+r,t:i.y-o.oy*sy,b:i.y+(o.h-o.oy)*sy};
}
function xformPt(o,i,px,py){var lx=(px-o.ox)*i.w/o.w,ly=(py-o.oy)*i.h/o.h;if(i.mirrored)lx=-lx;
 if(i.angle){var r=i.angle*Math.PI/180,c=Math.cos(r),s=Math.sin(r),x2=lx*c-ly*s;ly=lx*s+ly*c;lx=x2;}return[i.x+lx,i.y+ly];}
function framePoly(o,i){if(o.kind!=='sprite'||o.collision!=='poly')return null;var a=o.anims[i.anim||0]||o.anims[0],fi=i.frame||0,a0=o.anims[0];
 var p=(a&&a.poly&&(a.poly[fi]||a.poly[0]))||(a0&&a0.poly&&a0.poly[0]);return p&&p.length>=3?p:null;}
function colPoly(o,i){var p=framePoly(o,i);if(!p)return null;return p.map(function(q){return xformPt(o,i,q[0],q[1]);});}
function polyBB(P){var l=Infinity,t=Infinity,r=-Infinity,b=-Infinity;for(var k=0;k<P.length;k++){var q=P[k];if(q[0]<l)l=q[0];if(q[0]>r)r=q[0];if(q[1]<t)t=q[1];if(q[1]>b)b=q[1];}return{l:l,t:t,r:r,b:b};}
function boxPoly(b){return[[b.l,b.t],[b.r,b.t],[b.r,b.b],[b.l,b.b]];}
function ptInPoly(x,y,P){var c=false;for(var k=0,j=P.length-1;k<P.length;j=k++){var a=P[k],b=P[j];if((a[1]>y)!==(b[1]>y)&&x<(b[0]-a[0])*(y-a[1])/(b[1]-a[1])+a[0])c=!c;}return c;}
function segX(a,b,c,d){var d1=(d[0]-c[0])*(a[1]-c[1])-(d[1]-c[1])*(a[0]-c[0]),d2=(d[0]-c[0])*(b[1]-c[1])-(d[1]-c[1])*(b[0]-c[0]),d3=(b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0]),d4=(b[0]-a[0])*(d[1]-a[1])-(b[1]-a[1])*(d[0]-a[0]);return d1*d2<0&&d3*d4<0;}
function polyOvl(P,Q){for(var k=0;k<P.length;k++){var a=P[k],b=P[(k+1)%P.length];for(var m=0;m<Q.length;m++)if(segX(a,b,Q[m],Q[(m+1)%Q.length]))return true;}
 var cp=function(A){var x=0,y=0;A.forEach(function(q){x+=q[0];y+=q[1];});return[x/A.length,y/A.length];};
 var pc=cp(P),qc=cp(Q);return ptInPoly(pc[0],pc[1],Q)||ptInPoly(qc[0],qc[1],P)||P.some(function(q){return ptInPoly(q[0],q[1],Q);})||Q.some(function(q){return ptInPoly(q[0],q[1],P);});}
function imgPoint(o,i,nm){
 var lx=0,ly=0;
 if(o.kind==='sprite'&&nm!=null&&nm!==''){
  var pts=o.points||[],key=null;
  if(typeof nm==='number'||/^\d+$/.test(String(nm))){var q=pts[(+nm)-1];if(q)key=q.name;}else if(pts.some(function(q){return q.name===nm;}))key=String(nm);
  if(key!=null){var a=o.anims[i.anim||0]||o.anims[0],fi=i.frame||0,pp=a&&a.pts;
   var loc=(pp&&pp[fi]&&pp[fi][key])||(pp&&pp[0]&&pp[0][key])||[o.ox,o.oy];
   lx=(loc[0]-o.ox)*i.w/o.w;ly=(loc[1]-o.oy)*i.h/o.h;}
 }
 if(i.mirrored)lx=-lx;
 if(i.angle){var r=i.angle*Math.PI/180,c=Math.cos(r),s=Math.sin(r),x2=lx*c-ly*s;ly=lx*s+ly*c;lx=x2;}
 return{x:i.x+lx,y:i.y+ly};
}
function colBox(o,i){
 if(o.kind==='particles')return{l:i.x-16,t:i.y-16,r:i.x+16,b:i.y+16};
 if(o.kind!=='sprite')return{l:i.x,t:i.y,r:i.x+i.w,b:i.y+i.h};
 var c=o.cbox||{x:0,y:0,w:o.w,h:o.h},sx=i.w/o.w,sy=i.h/o.h,l=(c.x-o.ox)*sx,r=(c.x+c.w-o.ox)*sx;if(i.mirrored){var q=l;l=-r;r=-q;}
 if(i.b&&i.b.car&&i.angle){/* cars turn: axis-aligned box of the rotated body, slightly trimmed at diagonals */var t0=(c.y-o.oy)*sy,b0=(c.y+c.h-o.oy)*sy,cx=(l+r)/2,cy=(t0+b0)/2,hw=(r-l)/2,hh=(b0-t0)/2,ra=i.angle*Math.PI/180,cs=Math.abs(Math.cos(ra)),sn=Math.abs(Math.sin(ra)),k=1-.22*Math.abs(Math.sin(2*ra)),ex=(cs*hw+sn*hh)*k,ey=(sn*hw+cs*hh)*k,rx=cx*Math.cos(ra)-cy*Math.sin(ra),ry=cx*Math.sin(ra)+cy*Math.cos(ra);return{l:i.x+rx-ex,r:i.x+rx+ex,t:i.y+ry-ey,b:i.y+ry+ey};}
 return{l:i.x+l,r:i.x+r,t:i.y+(c.y-o.oy)*sy,b:i.y+(c.y+c.h-o.oy)*sy};
}

/* ================= AUDIO ================= */
var AC=null,muted=false,noiseBuf=null;
function audio(){if(!AC){try{AC=new (window.AudioContext||window.webkitAudioContext)();}catch(e){AC=null;}}if(AC&&AC.state==='suspended')AC.resume();return AC;}
var SFX={
 jump:[['square',220,560,.12,.05]],coin:[['square',988,988,.06,.05],['square',1319,1319,.18,.05,.06]],
 hit:[['triangle',420,80,.16,.14]],hurt:[['sawtooth',460,55,.4,.07],['square',220,40,.35,.04,.05]],
 powerup:[['square',392,392,.08,.05],['square',523,523,.08,.05,.08],['square',659,659,.08,.05,.16],['square',784,784,.2,.05,.24]],
 spring:[['sine',160,980,.26,.13]],laser:[['square',1400,200,.18,.05]],explosion:[['noise',0,0,.5,.25]],
 blip:[['square',660,660,.06,.05]],win:[['square',523,523,.1,.045],['square',659,659,.1,.045,.1],['square',784,784,.1,.045,.2],['square',1047,1047,.35,.045,.3]],
 lose:[['triangle',392,392,.18,.1],['triangle',330,330,.18,.1,.18],['triangle',262,190,.5,.1,.36]]
};
function sfx(n,dest){if(muted&&!dest)return;var a=audio();if(!a)return;var now=a.currentTime;(SFX[n]||[]).forEach(function(d){try{var t0=now+(d[5]||0),g=a.createGain(),o;
 if(d[0]==='noise'){if(!noiseBuf){noiseBuf=a.createBuffer(1,a.sampleRate*.6,a.sampleRate);var ch=noiseBuf.getChannelData(0);for(var k=0;k<ch.length;k++)ch[k]=(Math.random()*2-1)*(1-k/ch.length);}o=a.createBufferSource();o.buffer=noiseBuf;}
 else{o=a.createOscillator();o.type=d[0];o.frequency.setValueAtTime(d[1],t0);o.frequency.exponentialRampToValueAtTime(Math.max(1,d[2]),t0+d[3]);}
 g.gain.setValueAtTime(.0001,t0);g.gain.exponentialRampToValueAtTime(d[4],t0+.01);g.gain.exponentialRampToValueAtTime(.0001,t0+d[3]);o.connect(g);g.connect(dest||a.destination);o.start(t0);o.stop(t0+d[3]+.03);}catch(e){}});}

/* ================= RUNTIME ================= */
var GAMEKEYS={ArrowLeft:1,ArrowRight:1,ArrowUp:1,ArrowDown:1,Space:1};
function Runtime(canvas,P,opts){
 opts=opts||{};
 var ctx=canvas.getContext('2d');
 var S=Object.assign({width:854,height:480,pixelArt:true,title:'Mon jeu'},P.settings||{});
 var W=S.width,H=S.height,DT=1/60;
 var types={},byName={},all=[],uidc=1,solids=[],jts=[],ladders=[],polyCount=0,joints=[],pendingRopes=[],lastBlk=null;
 (P.objects||[]).forEach(function(o){var t={def:o,id:o.id,name:o.name,insts:[],bh:{}};(o.behaviors||[]).forEach(function(b){t.bh[b.type]=Object.assign(bdefaults(b.type),b.props||{});});types[o.id]=t;byName[o.name]=t;});
 var famsOf={};(P.objects||[]).forEach(function(o){if(o.kind!=='family')return;var t=types[o.id];t.fam=true;t.mem={};t.memList=[];(o.members||[]).forEach(function(id){if(types[id]&&types[id].def.kind!=='family'){t.mem[id]=1;t.memList.push(types[id]);(famsOf[id]=famsOf[id]||[]).push(t);}});});
 function inType(i,t){return i.t===t||(t.fam&&!!t.mem[i.t.id]);}
 var imgs={};function img(src){var m=imgs[src];if(!m){m=new Image();m.src=src;imgs[src]=m;}return m;}
 var srcs=[];(P.objects||[]).forEach(function(o){if(o.kind==='sprite')(o.anims||[]).forEach(function(a){a.frames.forEach(function(f){srcs.push(f);});});if((o.kind==='tiled'||o.kind==='tilemap')&&o.image)srcs.push(o.image);});
 var globals={};function resetG(){(P.globals||[]).forEach(function(g){globals[g.name]=g.type==='text'?String(g.value==null?'':g.value):(+g.value||0);});}resetG();
 var waits=[],layout=null,sheet=null,cstate={},tick=0,time=0,firstTick=true,pending=null,cam={x:0,y:0},shakeT=0,shakeA=0,shakeD=1;
 var layers=[],loopStack=[],F=Object.create(EXF);
 var scopes=[],groupState={},funcs={},groupsByName={},callDepth=0,includeStack=[];
 F.__a=EXF.__a;F.call=function(nm){return runFunc(String(nm),Array.prototype.slice.call(arguments,1));};
 function findVarScope(n){for(var k=scopes.length-1;k>=0;k--)if(n in scopes[k].vars)return scopes[k];return null;}
 function getVar(n){var sc=findVarScope(n);return sc?sc.vars[n]:globals[n];}
 function setVar(n,v){var sc=findVarScope(n);if(sc)sc.vars[n]=v;else globals[n]=v;}
 (function walk(list){(list||[]).forEach(function(e){if(e.kind==='function'&&e.name)funcs[e.name]=e;if(e.kind==='group'){(groupsByName[e.name]=groupsByName[e.name]||[]).push(e);walk(e.subs);}});})([].concat.apply([],(P.sheets||[]).map(function(s){return s.events||[];})));
 F.loopindex=function(n){for(var k=loopStack.length-1;k>=0;k--)if(n==null||loopStack[k].name===String(n))return loopStack[k].i;return 0;};
 var mouse={sx:-9999,sy:-9999,down:{},pr:{},rl:{}},BTN={0:'left',1:'middle',2:'right'};
 var spref='pf:'+(S.title||'jeu')+':',smem={};
 function sget(k){try{var v=localStorage.getItem(spref+k);if(v!=null)return JSON.parse(v);}catch(e){}return smem[k];}
 function sset(k,v){smem[k]=v;try{localStorage.setItem(spref+k,JSON.stringify(v));}catch(e){}}
 function sdel(k){delete smem[k];try{localStorage.removeItem(spref+k);}catch(e){}}
 function sclear(){smem={};try{var ks=[];for(var n=0;n<localStorage.length;n++){var k=localStorage.key(n);if(k&&k.indexOf(spref)===0)ks.push(k);}ks.forEach(function(k){localStorage.removeItem(k);});}catch(e){}}
 F.highscore=function(t,r){var e=hsList(t)[Math.floor(num(r))-1];return e?e.s:0;};F.highscorename=function(t,r){var e=hsList(t)[Math.floor(num(r))-1];return e?e.n:'';};F.highscorecount=function(t){return hsList(t).length;};
 F.storage=function(k,d){var v=sget(String(k));return v===undefined?(d===undefined?0:d):v;};
 /* audio */
 var master=null,bufs={},voices=[],music=null,mvol=1;
 function mst(){var a=audio();if(!a)return null;if(!master){master=a.createGain();master.gain.value=muted?0:mvol;master.connect(a.destination);}return master;}
 function sndData(id){var s=(P.sounds||[]).find(function(x){return x.id===id;});return s&&s.data;}
 function loadSnd(id){if(bufs[id])return bufs[id];var d=sndData(id),a=audio();if(!d||!a)return Promise.reject();
  bufs[id]=new Promise(function(res,rej){try{var b64=d.slice(d.indexOf(',')+1),bin=atob(b64),u=new Uint8Array(bin.length);for(var k=0;k<bin.length;k++)u[k]=bin.charCodeAt(k);a.decodeAudioData(u.buffer,res,rej);}catch(e){rej(e);}});bufs[id].catch(function(){});return bufs[id];}
 function playFile(id,vol,loop,tag){var m=mst();if(!m)return;loadSnd(id).then(function(buf){if(!running)return;var a=audio(),src=a.createBufferSource(),g=a.createGain();src.buffer=buf;src.loop=!!loop;g.gain.value=Math.max(0,vol);src.connect(g);g.connect(m);src.start();var v={src:src,g:g,tag:tag||''};voices.push(v);src.onended=function(){var k=voices.indexOf(v);if(k>=0)voices.splice(k,1);};},function(){});}
 function fadeStop(g,src,f){var a=audio();try{var t=a.currentTime;g.gain.cancelScheduledValues(t);g.gain.setValueAtTime(g.gain.value,t);if(f>0){g.gain.linearRampToValueAtTime(0,t+f);src.stop(t+f+.02);}else src.stop();}catch(e){}}
 function stopMusic(f){if(music){fadeStop(music.g,music.src,f);music=null;}}
 function playMusic(id,vol,f){var m=mst();if(!m)return;if(music&&music.id===id){music.g.gain.setValueAtTime(Math.max(0,vol),audio().currentTime);return;}stopMusic(f);var tok={id:id,pending:true};music=tok;
  loadSnd(id).then(function(buf){if(!running||music!==tok)return;var a=audio(),src=a.createBufferSource(),g=a.createGain();src.buffer=buf;src.loop=true;src.connect(g);g.connect(m);var t=a.currentTime;g.gain.setValueAtTime(f>0?0:vol,t);if(f>0)g.gain.linearRampToValueAtTime(Math.max(0,vol),t+f);src.start();tok.src=src;tok.g=g;tok.pending=false;},function(){if(music===tok)music=null;});}
 /* ---- synthesized engine: pitch follows speed through 5 virtual gears, tyre screech and off-road rumble are filtered noise ---- */
 var engines=[];
 function engNoise(a){if(!noiseBuf){noiseBuf=a.createBuffer(1,a.sampleRate*2,a.sampleRate);var d=noiseBuf.getChannelData(0);for(var k=0;k<d.length;k++)d[k]=Math.random()*2-1;}return noiseBuf;}
 function engWave(a){if(engWave.w)return engWave.w;var n=24,re=new Float32Array(n),im=new Float32Array(n);for(var k=1;k<n;k++){var amp=1/Math.pow(k,1.35)*(k%2?1:.6);re[k]=amp*Math.cos(k*1.7);im[k]=amp*Math.sin(k*1.7);}engWave.w=a.createPeriodicWave(re,im);return engWave.w;}
 function engGet(i){if(i._eng)return i._eng;var a=audio(),m=mst();if(!a||!m||a.state!=='running')return null;try{
  /* a soft pulse wave at the firing frequency + a sub sine, low-passed, with a slow "chug" tremolo */
  var o1=a.createOscillator(),o2=a.createOscillator(),lfo=a.createOscillator(),lfoG=a.createGain(),mix=a.createGain(),g2=a.createGain(),f=a.createBiquadFilter(),f2=a.createBiquadFilter(),g=a.createGain();
  o1.setPeriodicWave(engWave(a));o2.type='sine';g2.gain.value=.55;lfo.type='sine';lfoG.gain.value=.22;mix.gain.value=.78;
  f.type='lowpass';f.frequency.value=320;f.Q.value=.6;f2.type='lowpass';f2.frequency.value=1600;f2.Q.value=.3;g.gain.value=0;
  o1.connect(mix);o2.connect(g2);g2.connect(mix);lfo.connect(lfoG);lfoG.connect(mix.gain);mix.connect(f);f.connect(f2);f2.connect(g);g.connect(m);
  var nz=a.createBufferSource(),bp=a.createBiquadFilter(),bp2=a.createBiquadFilter(),sg=a.createGain(),lp=a.createBiquadFilter(),rg=a.createGain();
  nz.buffer=engNoise(a);nz.loop=true;bp.type='bandpass';bp.frequency.value=2300;bp.Q.value=9;bp2.type='bandpass';bp2.frequency.value=3100;bp2.Q.value=11;sg.gain.value=0;nz.connect(bp);nz.connect(bp2);bp.connect(sg);bp2.connect(sg);sg.connect(m);
  lp.type='lowpass';lp.frequency.value=180;lp.Q.value=.5;rg.gain.value=0;nz.connect(lp);lp.connect(rg);rg.connect(m);
  o1.frequency.value=32;o2.frequency.value=16;lfo.frequency.value=8;o1.start();o2.start();lfo.start();nz.start();
  var e={i:i,o1:o1,o2:o2,lfo:lfo,f:f,g:g,nz:nz,bp:bp,sg:sg,rg:rg,last:a.currentTime,rpm:.2,vol:0};engines.push(e);i._eng=e;return e;}catch(err){return null;}}
 function engSet(i,v,load,screech,rumble){var e=engGet(i);if(!e)return;var a=AC,t=a.currentTime,G=5,gi=Math.min(G-1,Math.floor(v*G)),w=v*G-gi,rpm=v<.02?.12:(gi?.38+.62*w:.15+.85*w);
  e.rpm+=(rpm-e.rpm)*(rpm<e.rpm?.06:.12);var hz=26+e.rpm*92;
  e.o1.frequency.setTargetAtTime(hz,t,.06);e.o2.frequency.setTargetAtTime(hz*.5,t,.06);e.lfo.frequency.setTargetAtTime(hz*.25,t,.08);
  e.f.frequency.setTargetAtTime(200+e.rpm*520+load*380,t,.08);
  var vol=.05+.035*e.rpm+.03*load;e.g.gain.setTargetAtTime(vol,t,.08);
  e.bp.frequency.setTargetAtTime(2000+Math.min(1,screech)*700,t,.1);e.sg.gain.setTargetAtTime(Math.max(0,Math.min(1,screech))*.028,t,.08);e.rg.gain.setTargetAtTime(Math.max(0,Math.min(1,rumble))*.12,t,.08);e.last=t;}
 function engIdle(){if(!AC)return;var t=AC.currentTime;engines.forEach(function(e){if(t-e.last>.12||e.i.dead){e.g.gain.setTargetAtTime(0,t,.05);e.sg.gain.setTargetAtTime(0,t,.05);e.rg.gain.setTargetAtTime(0,t,.05);}});}
 function engStopAll(){engines.forEach(function(e){try{e.o1.stop();e.o2.stop();e.lfo.stop();e.nz.stop();e.g.disconnect();e.sg.disconnect();e.rg.disconnect();}catch(err){}if(e.i)e.i._eng=null;});engines=[];}
 function stopAllSnd(){voices.slice().forEach(function(v){try{v.src.stop();}catch(e){}});voices=[];if(music&&music.src)fadeStop(music.g,music.src,0);music=null;}
 var keys={},kp={},pkp={},padNav={},kr={},anyPress=false,cur=null,curSol={},running=true,raf=0,last=0,acc=0,ready=false,cw=0,ch=0;
 var errors=[];

 /* ---- instances ---- */
 function mainLayer(){for(var k=0;k<layers.length;k++)if(layers[k].px===1&&layers[k].py===1)return layers[k];return layers[0]||{px:1,py:1};}
 function mouseOver(i){var l=layers[i.layer]||{px:1,py:1},z=zf(l,i),x=mouse.sx/z+(i.hud?0:cam.x*l.px),y=mouse.sy/z+(i.hud?0:cam.y*l.py),b=drawBox(i.t.def,i);if(!(i.visible&&(!layers[i.layer]||layers[i.layer].visible)&&x>=b.l&&x<=b.r&&y>=b.t&&y<=b.b))return false;var cp=colPoly(i.t.def,i);return cp?ptInPoly(x,y,cp):true;}
 function layerIdx(nm){for(var k=0;k<layers.length;k++)if(layers[k].name===nm)return k;return -1;}
 function createInst(t,d,x,y,li){
  if(t&&t.fam){if(!t.memList.length)return null;t=t.memList[Math.floor(Math.random()*t.memList.length)];}
  if(!t)return null;var o=t.def;
  var i={uid:uidc++,t:t,x:d?d.x:x,y:d?d.y:y,w:(d&&d.w)||o.defW||o.w||32,h:(d&&d.h)||o.defH||o.h||32,angle:(d&&d.angle)||0,mirrored:!!(d&&d.mirrored),
   opacity:d&&d.opacity!=null?d.opacity:1,visible:d?d.visible!==false:true,hud:!!(d&&d.hud),text:d&&d.text!=null?d.text:(o.text||''),layer:0,vars:{},anim:0,frame:0,ft:0,animEnd:-1,dead:false,flash:0,b:{}};
  if(d){var lk=layers.findIndex(function(l){return l.id===d.layer;});i.layer=lk>=0?lk:0;}else if(li!=null&&li>=0)i.layer=li;
  if(o.kind==='tilemap'){var cols=(d&&d.cols)||o.defCols||8,rows=(d&&d.rows)||o.defRows||4,dt=(d&&d.data)||[],arr=new Array(cols*rows);for(var q=0;q<arr.length;q++)arr[q]=dt[q]==null?-1:dt[q];i.tm={cols:cols,rows:rows,data:arr};i.w=cols*(o.tw||16)*(o.scale||1);i.h=rows*(o.th||16)*(o.scale||1);}
  if(o.kind==='road3d'){i.lk=function(id){return types[id]&&types[id].def;};rdInit(i,false);}
  if(o.kind==='minimap')i.mmSrc=mmSource;
  if(o.kind==='bar'){var bmx=Math.max(.0001,+o.max||100),bv=d&&d.value!=null?+d.value:(o.value!=null?+o.value:bmx);bv=Math.max(0,Math.min(bmx,bv));i.bar={v:bv,max:bmx,shown:bv,ghost:bv,gt:0,col:''};}
  if(d&&d.style)i.style=Object.assign({},d.style);
  if(o.kind==='leaderboard'){i.lbt=(d&&d.table)||o.table||'scores';i.lb=hsList(i.lbt);i.lbHi=hsLast&&hsLast.t===i.lbt?hsLast.d:null;}
  if(o.kind==='button'){i.btn={hover:false,press:false,focus:false,en:true,clicked:-1};if(d&&d.text==null)i.text=o.text||'';}
  if(o.kind==='particles'){var pso=o.pt&&o.pt.image&&types[o.pt.image];i.ps={parts:[],acc:0,active:true,started:false,img:pso&&pso.def.kind==='sprite'&&pso.def.anims[0]?pso.def.anims[0].frames[0]:null};}
  var ivd=(o.vars||[]).slice();(famsOf[t.id]||[]).forEach(function(ft){(ft.def.vars||[]).forEach(function(v){if(!ivd.some(function(x){return x.name===v.name;}))ivd.push(v);});});
  ivd.forEach(function(v){var val=d&&d.vars&&d.vars[v.name]!=null?d.vars[v.name]:v.value;i.vars[v.name]=v.type==='text'?String(val==null?'':val):(+val||0);});
  for(var k in t.bh){var p=Object.assign({},t.bh[k]),st={p:p};
   if(k==='platform'){st.vx=0;st.vy=0;st.floor=null;st.coyote=0;st.buf=0;st.jumps=0;st.landed=false;st.jumped=false;st.sim={};st.en=true;st.hold=false;}
   if(k==='patrol'){st.dir=p.startLeft?-1:1;st.vy=0;st.floor=null;}
   if(k==='sine'){st.bx=i.x;st.by=i.y;st.t=0;st.en=true;}
   if(k==='path'){if(d&&d.path&&d.path.length&&!t.tplPath)t.tplPath=d.path;st.bx=i.x;st.by=i.y;st.pts=[[0,0]].concat(((d&&d.path)||(!d&&t.tplPath)||[]).map(function(q){return [+q[0]||0,+q[1]||0];}));st.k=0;st.nx=1;st.dir=1;st.u=0;st.wt=0;st.en=!!p.auto;st.arr=-1;st.fin=-1;st.last=0;}
   if(k==='timer'){st.timers={};st.fired={};}
   if(k==='bullet'){var a=p.angle*Math.PI/180;st.vx=Math.cos(a)*p.speed;st.vy=Math.sin(a)*p.speed;}
   if(k==='eightdir'){st.vx=0;st.vy=0;st.sim={};st.en=true;}
   if(k==='car'){st.sp=0;st.vx=0;st.vy=0;st.sim={};st.en=true;st.ctl=!!p.controls;st.hit=-1;st.hitV=0;}
   if(k==='tween'){st.list=[];st.done={};}
   if(k==='physics'){st.vx=0;st.vy=0;st.fixed=!!p.immovable;st.ground=false;st.hitT=-1;st.hitV=0;if(p.rope>0)pendingRopes.push({i:i,len:+p.rope});}
   if(k==='fade'){st.t=0;st.run=!!p.auto;st.base=i.opacity;st.done=-1;if(st.run&&p.fadeIn>0)i.opacity=0;}
   if(k==='pin'){st.to=null;}
   if(k==='drag'){st.on=false;st.en=true;st.st=-1;st.dr=-1;}
   if(k==='turret'){st.types=[];st.target=null;st.cd=0;st.fired=-1;}
   i.b[k]=st;}
  t.insts.push(i);(famsOf[t.id]||[]).forEach(function(ft){ft.insts.push(i);});all.push(i);return i;
 }
 function alive(i){return !i.dead;}
 function picked(sol,t){return (sol[t.id]||t.insts).filter(alive);}
 function box(i){return i._v?i.vb:colBox(i.t.def,i);}
 function tmCellSize(i){var o=i.t.def;return[(o.tw||16)*(o.scale||1),(o.th||16)*(o.scale||1)];}
 function tmTiles(i){var o=i.t.def,D=i.tm,cs=tmCellSize(i),C=cs[0],H=cs[1],ver=TMV?(TMV.get(D.data)||0):0,key=i.x+','+i.y+','+ver+','+D.cols+','+D.rows+','+C+','+H;
  if(D._vk===key)return D._vt;var out={rect:[],jt:[],poly:[],all:[]},tc=o.tcol||{},r,c;
  for(r=0;r<D.rows;r++){var run=null;for(c=0;c<=D.cols;c++){var v=c<D.cols?D.data[r*D.cols+c]:-1,ty=(v==null||v<0)?null:(tc[v]||'s'),rk=ty==='s'||ty==='j'?ty:null;
    if(run&&run.ty!==rk){var V={_v:1,vb:{l:i.x+run.c0*C,r:i.x+c*C,t:i.y+r*H,b:i.y+(r+1)*H},sh:null,t:i.t,dead:false,owner:i};(run.ty==='s'?out.rect:out.jt).push(V);out.all.push(V);run=null;}
    if(rk&&!run)run={ty:rk,c0:c};
    if(ty&&!rk&&ty!=='n'){var pp=typeof ty==='object'?(ty.p&&ty.p.length>=3?ty.p.map(function(q){return[q[0]*C/(o.tw||16),q[1]*H/(o.th||16)];}):null):tilePoly(ty,C,H);if(pp){var PP=pp.map(function(q){return[i.x+c*C+q[0],i.y+r*H+q[1]];}),bb=polyBB(PP),V2={_v:1,vb:bb,sh:{p:PP,b:bb},t:i.t,dead:false,owner:i};out.poly.push(V2);out.all.push(V2);}}}}
  D._vk=key;D._vt=out;return out;}
 function tmGet(i,c,r){var D=i.tm;if(!D)return -1;c=Math.floor(c);r=Math.floor(r);if(c<0||r<0||c>=D.cols||r>=D.rows)return -1;var v=D.data[r*D.cols+c];return v==null?-1:v;}
 function tmSet(i,c,r,v){var D=i.tm;if(!D)return;c=Math.floor(c);r=Math.floor(r);if(c<0||r<0||c>=D.cols||r>=D.rows)return;v=Math.floor(+v);D.data[r*D.cols+c]=isNaN(v)||v<0?-1:v;tmDirty(D.data);}
 function tmCol(i,x){return Math.floor((x-i.x)/tmCellSize(i)[0]);}function tmRow(i,y){return Math.floor((y-i.y)/tmCellSize(i)[1]);}
 function ovl(a,b){return a.l<b.r&&a.r>b.l&&a.t<b.b&&a.b>b.t;}
 function shape(i){if(i.tm)return{b:box(i),tiles:tmTiles(i).all};var p=colPoly(i.t.def,i);return p?{p:p,b:polyBB(p)}:{b:box(i)};}
 function hits(sa,sb){if(!ovl(sa.b,sb.b))return false;if(sa.tiles||sb.tiles){var T=sa.tiles?sa:sb,O=sa.tiles?sb:sa;return T.tiles.some(function(v){return hits(O,v.sh?{p:v.sh.p,b:v.sh.b}:{b:v.vb});});}if(!sa.p&&!sb.p)return true;return polyOvl(sa.p||boxPoly(sa.b),sb.p||boxPoly(sb.b));}

 /* ---- layout ---- */
 var cpLoad=null;
 function loadLayoutPre(){}
 function loadLayout(id){engStopAll();joints=[];pendingRopes=[];cine=null;cineBars=0;cineTitle=null;cineFade.a=0;if(cpLoad){cp=cpLoad.layout===id?cpLoad:null;cpLoad=null;}if(cp&&cp.layout!==id)cp=null;gpaused=false;btnFocus=null;zoom=1;zoomA=null;camLook=null;camB=null;camLA=0;dark={a:0,c:dark.c};tscale=1;dlg.queue=[];dlg.cur=null;
  var L=P.layouts.find(function(l){return l.id===id;})||P.layouts[0];layout=L;
  sheet=(P.sheets||[]).find(function(s){return s.id===L.sheet;})||null;
  all=[];for(var k in types){types[k].insts=[];types[k].tplPath=null;}cstate={};waits=[];loopStack=[];scopes=[];groupState={};firstTick=true;shakeT=0;
  layers=(L.layers&&L.layers.length?L.layers:[{id:'_',name:'Principal',parallaxX:100,parallaxY:100}]).map(function(l){return{id:l.id,name:l.name,px:(l.parallaxX==null?100:l.parallaxX)/100,py:(l.parallaxY==null?100:l.parallaxY)/100,opacity:(l.opacity==null?100:l.opacity)/100,visible:l.visible!==false};});
  (L.instances||[]).forEach(function(d){if(!d.off)createInst(types[d.obj],d);});
  updateCam();
 }

 /* ---- behaviors ---- */
 /* solids: boxes (fast path) or polygons (slopes) */
 function sshape(s){if(s._v)return s.sh;var k=s.x+','+s.y+','+s.w+','+s.h+','+s.angle+','+(s.mirrored?1:0)+','+s.anim+','+s.frame;if(s._sk!==k){s._sk=k;var p=colPoly(s.t.def,s);s._sh=p?{p:p,b:polyBB(p)}:null;}return s._sh;}
 function shrink(b,e){return{l:b.l+e,r:b.r-e,t:b.t+e,b:b.b-e};}
 function surfY(P,x,flat){var y=null;for(var k=0;k<P.length;k++){var a=P[k],c=P[(k+1)%P.length];if(a[0]===c[0])continue;if(flat&&Math.abs((c[1]-a[1])/(c[0]-a[0]))>.25)continue;var x0=Math.min(a[0],c[0]),x1=Math.max(a[0],c[0]);if(x<x0||x>x1)continue;var t=(x-a[0])/(c[0]-a[0]),yy=a[1]+(c[1]-a[1])*t;if(y===null||yy<y)y=yy;}return y;}
 function groundY(P,b){var cx=(b.l+b.r)/2,g=surfY(P,cx);if(g!==null)return g;var y1=surfY(P,b.l+1,1),y2=surfY(P,b.r-1,1);if(y1===null)return y2;if(y2===null)return y1;return Math.min(y1,y2);}
 function polyHit(b,sh){return ovl(b,sh.b)&&polyOvl(boxPoly(shrink(b,.02)),sh.p);}
 function climbOf(b,dx){return (b.r-b.l)/2*1.25+Math.abs(dx)*1.5+2;}
 function polyBlock(i,lift){var b=box(i);if(lift)b.b-=lift;if(b.b<=b.t)return null;for(var n=0;n<solids.length;n++){var s=solids[n];if(s===i||s.dead)continue;var sh=sshape(s);if(sh&&polyHit(b,sh))return s;}return null;}
 function ovl0(a,s){if(!s.b||!s.b.physics)return false;var q=box(s);return a.r>q.l+2&&a.l<q.r-2&&a.b>q.t+2&&a.t<q.b-2;}
 function moveX(i,dx,grounded){lastBlk=null;if(!dx)return 0;var x0=i.x,pre=polyCount&&!grounded&&polyBlock(i,0),b0=box(i);i.x+=dx;var b=box(i),hit=0,n,s,sb;
  var stepUp=grounded?(polyCount?climbOf(b,dx):4):0;
    for(n=0;n<solids.length;n++){s=solids[n];if(s===i||s.dead||sshape(s)||ovl0(b0,s))continue;sb=box(s);var phy=i.b.physics||s.b&&s.b.physics,tol=phy?1.5:.01;if(phy&&(dx>0?sb.l+sb.r<b0.l+b0.r:sb.l+sb.r>b0.l+b0.r))continue;if(b.r>sb.l&&b.l<sb.r&&b.b>sb.t+tol&&b.t<sb.b-tol){
    if(stepUp&&b.b-sb.t<=(s.b&&s.b.physics?4:stepUp)){var oy=i.y;i.y-=b.b-sb.t;if(!blockedAny(i)){b=box(i);i._sf=s;continue;}i.y=oy;}
    if(dx>0)i.x-=b.r-sb.l;else i.x+=sb.r-b.l;if((i.x-x0)*dx<0)i.x=x0;b=box(i);hit=dx>0?1:-1;lastBlk=s;}}
  if(!polyCount)return hit;
  var climb=grounded||pre?climbOf(b,dx):0;
  if(polyBlock(i,climb)){var lo=0,hi=i.x-x0,base=x0;for(var k=0;k<10;k++){var mid=(lo+hi)/2;i.x=base+mid;if(polyBlock(i,climb))hi=mid;else lo=mid;}i.x=base+lo;hit=dx>0?1:-1;}
  if(grounded){b=box(i);var best=null,bs=null;for(n=0;n<solids.length;n++){s=solids[n];if(s===i||s.dead)continue;var sh=sshape(s);if(!sh||sh.b.r<b.l||sh.b.l>b.r)continue;var g=groundY(sh.p,b);if(g!==null&&g<b.b&&b.b-g<=climb&&(best===null||g<best)){best=g;bs=s;}}
   if(bs){i.y-=b.b-best;i._sf=bs;}}
  return hit;}
 function moveY(i,dy){if(!dy)return null;var b0=box(i),pb=b0.b,y0=i.y;i.y+=dy;var b=box(i),n,s,sb,res=null,sh;
  if(dy>0){for(n=0;n<solids.length;n++){s=solids[n];if(s===i||s.dead||(s.b&&s.b.physics&&pb>box(s).t+Math.max(3,dy+1)))continue;sh=sshape(s);
     if(sh){if(!ovl({l:b.l,r:b.r,t:b.t,b:b.b+.01},sh.b))continue;var g=groundY(sh.p,b);if(g!==null&&b.b>=g-.001&&pb<=g+2){i.y-=b.b-g;b=box(i);res=s;}continue;}
     sb=box(s);if(b.r>sb.l+.01&&b.l<sb.r-.01&&b.b>sb.t&&b.t<sb.b){i.y-=b.b-sb.t;b=box(i);res=s;}}
   for(n=0;n<jts.length;n++){s=jts[n];if(s===i||s.dead||(i._nojt&&s.t.bh.ladder))continue;sb=box(s);if(b.r>sb.l+.01&&b.l<sb.r-.01&&pb<=sb.t+.5&&b.b>=sb.t){i.y-=b.b-sb.t;b=box(i);res=s;}}}
  else{for(n=0;n<solids.length;n++){s=solids[n];if(s===i||s.dead||sshape(s)||(s.b&&s.b.physics&&b0.t<box(s).b-Math.max(3,1-dy)))continue;sb=box(s);if(b.r>sb.l+.01&&b.l<sb.r-.01&&b.t<sb.b&&b.b>sb.t){i.y+=sb.b-b.t;b=box(i);res=s;}}
   if(polyCount){var hs=polyBlock(i,climbOf(b,0));if(hs){var lo=0,hi=i.y-y0;for(var k=0;k<10;k++){var mid=(lo+hi)/2;i.y=y0+mid;if(polyBlock(i,climbOf(b,0)))hi=mid;else lo=mid;}i.y=y0+lo;res=hs;}}}
  return res;}
 function groundSnap(i,maxd){if(!polyCount)return null;var b=box(i),best=null,bs=null;for(var n=0;n<solids.length;n++){var s=solids[n];if(s===i||s.dead)continue;var sh=sshape(s);if(!sh||sh.b.r<b.l||sh.b.l>b.r)continue;var g=groundY(sh.p,b);if(g!==null&&g>=b.b-.001&&g-b.b<=maxd&&(best===null||g<best)){best=g;bs=s;}}
  if(bs){i.y+=best-b.b;}return bs;}
 function blockedAny(i){var b=shrink(box(i),.02);for(var n=0;n<solids.length;n++){var s=solids[n];if(s===i||s.dead)continue;var sh=sshape(s);if(sh?polyHit(b,sh):ovl(b,box(s)))return true;}return false;}
 function slide(i,dx,dy){var hx=0,hy=0;[['x',dx],['y',dy]].forEach(function(a){if(!a[1])return;var o=i[a[0]];i[a[0]]=o+a[1];if(blockedAny(i)){var lo=0,hi=a[1];for(var k=0;k<10;k++){var m=(lo+hi)/2;i[a[0]]=o+m;if(blockedAny(i))hi=m;else lo=m;}i[a[0]]=o+lo;if(a[0]==='x')hx=1;else hy=1;}});return[hx,hy];}
 function pointSolid(x,y){var n,b,sh;for(n=0;n<solids.length;n++){if(solids[n].dead)continue;sh=sshape(solids[n]);if(sh){if(x>=sh.b.l&&x<=sh.b.r&&y>=sh.b.t&&y<=sh.b.b&&ptInPoly(x,y,sh.p))return true;continue;}b=box(solids[n]);if(x>=b.l&&x<b.r&&y>=b.t&&y<b.b)return true;}for(n=0;n<jts.length;n++){if(jts[n].dead)continue;b=box(jts[n]);if(x>=b.l&&x<b.r&&y>=b.t&&y<b.b)return true;}return false;}
 function tprop(i,pr){return pr==='x'?i.x:pr==='y'?i.y:pr==='w'?i.w:pr==='h'?i.h:pr==='angle'?i.angle:pr==='opacity'?i.opacity*100:0;}
 function tset(i,pr,v){if(pr==='x')i.x=v;else if(pr==='y')i.y=v;else if(pr==='w')i.w=v;else if(pr==='h')i.h=v;else if(pr==='angle')i.angle=v;else if(pr==='opacity')i.opacity=Math.max(0,Math.min(1,v/100));
  if(i.b.sine&&(pr==='x'||pr==='y')){i.b.sine.bx=i.x;i.b.sine.by=i.y;}}
 function ease(k,t){switch(k){case 'linear':return t;case 'in':return t*t*t;case 'out':return 1-Math.pow(1-t,3);case 'back':var c=1.70158;return 1+(c+1)*Math.pow(t-1,3)+c*Math.pow(t-1,2);
  case 'elastic':return t===0||t===1?t:Math.pow(2,-10*t)*Math.sin((t*10-.75)*2*Math.PI/3)+1;
  case 'bounce':var n=7.5625,d=2.75;if(t<1/d)return n*t*t;if(t<2/d)return n*(t-=1.5/d)*t+.75;if(t<2.5/d)return n*(t-=2.25/d)*t+.9375;return n*(t-=2.625/d)*t+.984375;
  default:return t<.5?4*t*t*t:1-Math.pow(-2*t+2,3)/2;}}
 function center(i){var b=box(i);return[(b.l+b.r)/2,(b.t+b.b)/2];}
 function mouseWorld(i){var l=layers[i.layer]||{px:1,py:1},z=zf(l,i);return[mouse.sx/z+(i.hud?0:cam.x*l.px),mouse.sy/z+(i.hud?0:cam.y*l.py)];}
 function rayFree(a,b,self,other){var dx=b[0]-a[0],dy=b[1]-a[1],L=Math.hypot(dx,dy),n=Math.ceil(L/6);for(var k=1;k<n;k++){var x=a[0]+dx*k/n,y=a[1]+dy*k/n;for(var m=0;m<solids.length;m++){var s=solids[m];if(s===self||s===other||s.dead)continue;var sh=sshape(s);if(sh){if(x>=sh.b.l&&x<=sh.b.r&&y>=sh.b.t&&y<=sh.b.b&&ptInPoly(x,y,sh.p))return false;}else{var q=box(s);if(x>=q.l&&x<q.r&&y>=q.t&&y<q.b)return false;}}}return true;}
 function canSee(i,j){var st=i.b.los;if(!st)return false;var a=center(i),b=center(j),dx=b[0]-a[0],dy=b[1]-a[1],d=Math.hypot(dx,dy);if(d>st.p.range)return false;
  if(st.p.cone<360){var face=(i.mirrored?180-(i.angle||0):(i.angle||0)),ang=Math.atan2(dy,dx)*180/Math.PI,df=((ang-face)%360+540)%360-180;if(Math.abs(df)>st.p.cone/2)return false;}
  return rayFree(a,b,i,j);}
 function emitParts(i,n){ptEmit(i.t.def.pt||{},i.ps,i.x,i.y,i.angle,i.mirrored,n);}
 function psStep(i,dt){var P=i.t.def.pt||{},st=i.ps;ptStep(P,st,dt,i.x,i.y,i.angle,i.mirrored);if(P.mode==='burst'&&st.started&&!st.parts.length&&P.destroyEnd!==false)i._kill=true;}
 /* ---- Route 3D: simulation (the drawing is in rdDraw) ---- */
 function mmSource(me){var out=[];for(var k=0;k<all.length;k++){var j=all[k];if(j.dead||j===me||j.hud)continue;var ly=layers[j.layer];if(ly&&ly.px===0&&ly.py===0)continue;var b=box(j);out.push({id:j.t.id,l:b.l,t:b.t,r:b.r,b:b.b,x:(b.l+b.r)/2,y:(b.t+b.b)/2,angle:j.angle||0,solid:!!j.t.bh.solid});}return{w:layout.w,h:layout.h,items:out};}
 function rdMax(i){return rdBuild(i.t.def).L*60;}
 function rdMaxKmh(i){return i.rd.maxKmh||rdNum(i.t.def.maxKmh,220);}
 function rdKmh(i){return Math.round(i.rd.speed/rdMax(i)*rdNum(i.t.def.maxKmh,220));}
 function rdCurve(i){var R=rdBuild(i.t.def);return R.segs[Math.floor(((i.rd.pos%R.len)+R.len)%R.len/R.L)%R.segs.length].curve;}
 function rdProg(i){var R=rdBuild(i.t.def);return (((i.rd.pos%R.len)+R.len)%R.len)/R.len*100;}
 function rdRank(i){var me=i.rd.dist,n=1;i.rd.cars.forEach(function(c){if(c.rival&&c.dist>me)n++;});return n;}
 function rdInit(i,reset){var o=i.t.def,R=rdBuild(o),old=i.rd||{};i.rd={pos:0,dist:0,speed:0,x:0,lap:0,lapTime:0,last:old.last||0,best:reset?(old.best||0):0,lapT:-1,hitT:-1,ctl:reset&&old.ctl!=null?old.ctl:o.controls!==false,sim:{},sky:0,steer:0,maxKmh:old.maxKmh||0,cars:[]};
  var n=Math.max(0,Math.min(40,Math.round(rdNum(o.rivals,0)))),cid=o.rivalObj,L=R.L;if(!cid||!types[cid])n=0;
  for(var k=0;k<n;k++){var z=(k+1)*rdNum(o.gridGap,5)*L;i.rd.cars.push({id:cid,z:z,dist:z,off:(k%2?.45:-.45),spd:rdNum(o.rivalMin,60)/100+(rdNum(o.rivalMax,85)-rdNum(o.rivalMin,60))/100*((k*37%11)/10),rival:true,sway:k*1.7});}
  var tn=Math.max(0,Math.min(60,Math.round(rdNum(o.traffic,0)))),tid=o.trafficObj;if(tid&&types[tid])for(var t2=0;t2<tn;t2++){var z2=(t2+1)*R.len/(tn+1);i.rd.cars.push({id:tid,z:z2,dist:z2,off:(t2%3-1)*.55,spd:.25+(t2*13%7)/20,rival:false,sway:t2});}}
 function rdOverlap(x1,w1,x2,w2){return !(x1+w1/2<x2-w2/2||x1-w1/2>x2+w2/2);}
 function rdStep(i,dt){var o=i.t.def,R=rdBuild(o),S=R.segs,N=S.length,st=i.rd,L=R.L,mx=rdMax(i)*rdMaxKmh(i)/rdNum(o.maxKmh,220),rw=rdNum(o.roadWidth,2000),camH=rdNum(o.camHeight,1000),depth=1/Math.tan(rdNum(o.fov,100)/2*Math.PI/180),pZ=camH*depth,unit=2*camH/Math.max(1,i.w),sc=unit*rdNum(o.spriteScale,2);
  var c=st.ctl&&!cineLock(),sm=st.sim,G=sm.gas||(c&&(keys.ArrowUp||keys.KeyW)),B=sm.brake||(c&&(keys.ArrowDown||keys.KeyS)),Lf=sm.left||(c&&(keys.ArrowLeft||keys.KeyA)),Rt=sm.right||(c&&(keys.ArrowRight||keys.KeyD));st.sim={};
  var pseg=S[Math.floor((st.pos+pZ)/L)%N],sp=st.speed/Math.max(1,mx),dx=dt*2*sp*rdNum(o.steering,1);
  st.steer=(Rt?1:0)-(Lf?1:0);st.x+=st.steer*dx;st.x-=dx*sp*pseg.curve*rdNum(o.centrifugal,.3);st.sky+=pseg.curve*sp*dt*60;
  var acc=mx/Math.max(.2,rdNum(o.accelTime,5));if(G&&!B)st.speed+=acc*dt;else if(B)st.speed-=mx*rdNum(o.brakePower,1)*dt;else st.speed-=acc*dt;
  if(Math.abs(st.x)>1){if(st.speed>mx*rdNum(o.offroadMax,30)/100)st.speed-=mx*.5*dt*rdNum(o.offroadDrag,1);
   for(var q=0;q<pseg.spr.length;q++){var sp0=pseg.spr[q],d0=sp0.id==='#sign'?{defW:50}:i.lk(sp0.id);if(!d0)continue;var sw=(d0.defW||32)*sc/rw;if(!(st.hitCD>0)&&rdOverlap(st.x,rdNum(o.playerW,.3),sp0.off+sw/2*(sp0.off>0?1:-1),sw)){/* bounce back toward the road, keep rolling */st.speed=Math.min(st.speed,mx*.22);st.x-=(st.x>0?1:-1)*.35;st.hitT=tick;st.hitCD=.7;break;}}}
  if(st.hitCD>0)st.hitCD-=dt;
  st.speed=Math.max(0,Math.min(mx,st.speed));st.x=Math.max(-rdNum(o.maxOut,2.5),Math.min(rdNum(o.maxOut,2.5),st.x));
  /* other cars: rivals keep racing, traffic drives slowly; both bump the player */
  if(st.ctl)st.go=true;
  if(st.go)st.cars.forEach(function(car){var d=i.lk(car.id),cw=d?(d.defW||48)*unit/rw:.3,cs=S[Math.floor(car.z/L)%N],ahead=((st.pos+pZ)-car.z+R.len*2)%R.len;
   var v=car.spd*mx*(car.rival?(1-.15*Math.max(0,Math.abs(cs.curve)/6)):1);car.z=(car.z+v*dt)%R.len;car.dist+=v*dt;car.sway+=dt;car.off+=Math.sin(car.sway*.7)*.12*dt;car.off=Math.max(-.85,Math.min(.85,car.off));
   var gap=(car.z-(st.pos+pZ)+R.len*2)%R.len;if(gap<L*1.2&&st.speed>v&&!(st.hitCD>0)&&rdOverlap(st.x,rdNum(o.playerW,.3),car.off,cw*.9)){st.speed=v*.85;st.x+=(st.x>=car.off?1:-1)*.22;st.hitT=tick;st.hitCD=.5;}});
  var before=st.pos;st.pos+=st.speed*dt;st.dist+=st.speed*dt;st.lapTime+=dt;
  if(o.engine!==false)engSet(i,st.speed/Math.max(1,rdMax(i)),G&&st.ctl?1:0,(st.steer&&st.speed>mx*.55?(st.speed/mx-.55)*2.2:0)+(st.hitT===tick?1:0),Math.abs(st.x)>1?st.speed/mx*1.4:0);
  if(st.pos>=R.len){st.pos-=R.len;st.lap++;st.lapT=tick;st.last=st.lapTime;if(!st.best||st.lapTime<st.best)st.best=st.lapTime;st.lapTime=0;}}
 function behave(i,dt){var b=i.b,st;if(i.ps)psStep(i,dt);if(i.rd)rdStep(i,dt);
  if(b.eightdir&&b.eightdir.en){st=b.eightdir;var p=st.p,c=p.controls&&!cineLock(),sm=st.sim;
   var ix=((sm.right||(c&&(keys.ArrowRight||keys.KeyD)))?1:0)-((sm.left||(c&&(keys.ArrowLeft||keys.KeyA)))?1:0),iy=((sm.down||(c&&(keys.ArrowDown||keys.KeyS)))?1:0)-((sm.up||(c&&(keys.ArrowUp||keys.KeyW)))?1:0);
   if(p.dirs==='h')iy=0;else if(p.dirs==='v')ix=0;else if(p.dirs==='4'&&ix&&iy)iy=0;
   var L=Math.hypot(ix,iy);if(L){ix/=L;iy/=L;}
   var tx=ix*p.maxSpeed,ty=iy*p.maxSpeed,ax=(L?p.accel:p.decel)*dt;
   var dvx=tx-st.vx,dvy=ty-st.vy,dv=Math.hypot(dvx,dvy);if(dv<=ax){st.vx=tx;st.vy=ty;}else{st.vx+=dvx/dv*ax;st.vy+=dvy/dv*ax;}
   var h=slide(i,st.vx*dt,st.vy*dt);if(h[0])st.vx=0;if(h[1])st.vy=0;
   if(p.rotate&&Math.hypot(st.vx,st.vy)>1)i.angle=Math.atan2(st.vy,st.vx)*180/Math.PI;st.sim={};}
  if(b.car&&b.car.en){st=b.car;var cp2=st.p,cc=st.ctl&&!cineLock(),cs=st.sim,G=cs.gas||(cc&&(keys.ArrowUp||keys.KeyW)),Br=cs.brake||(cc&&(keys.ArrowDown||keys.KeyS)),TL=cs.left||(cc&&(keys.ArrowLeft||keys.KeyA)),TR=cs.right||(cc&&(keys.ArrowRight||keys.KeyD));
   if(G&&!Br)st.sp+=(st.sp<0?cp2.brake:cp2.accel)*dt;else if(Br&&!G)st.sp-=(st.sp>0?cp2.brake:cp2.accel*.6)*dt;else st.sp=approach(st.sp,0,cp2.friction*dt);
   st.sp=Math.max(-cp2.reverse,Math.min(cp2.maxSpeed,st.sp));
   var tdir=(TR?1:0)-(TL?1:0);if(tdir&&Math.abs(st.sp)>2){var a0=i.angle||0,f=Math.min(1,Math.abs(st.sp)/(Math.max(1,cp2.maxSpeed)*.3));i.angle=a0+tdir*cp2.steer*f*(st.sp<0?-1:1)*dt;if(blockedAny(i))i.angle=a0;}
   i.angle=((i.angle||0)%360+360)%360;
   var ra=i.angle*Math.PI/180,hx=Math.cos(ra),hy=Math.sin(ra),grip=Math.min(1,(1-cp2.drift)*14*dt+.02);st.vx+=(hx*st.sp-st.vx)*grip;st.vy+=(hy*st.sp-st.vy)*grip;
   var spd=Math.hypot(st.vx,st.vy),hc=slide(i,st.vx*dt,st.vy*dt);
   if(st.ctl&&cp2.engine!==false){var vf=Math.abs(st.sp)/Math.max(1,cp2.maxSpeed),slipA=Math.abs(Math.atan2(st.vy,st.vx)-ra);if(slipA>Math.PI)slipA=2*Math.PI-slipA;engSet(i,vf,G?1:0,spd>120?Math.min(1,slipA*2.5)*(spd/cp2.maxSpeed):0,0);}
   if(hc[0]||hc[1]){if(spd>20&&st.hit!==tick-1){st.hit=tick;st.hitV=spd;}if(hc[0])st.vx*=-.25;if(hc[1])st.vy*=-.25;st.sp*=-.25;}
   st.sim={};}
  if(b.drag&&b.drag.en){st=b.drag;if(st.on){if(!mouse.down.left){st.on=false;st.dr=tick;}else{var mw=mouseWorld(i);if(st.p.axes!=='v')i.x=mw[0]-st.gx;if(st.p.axes!=='h')i.y=mw[1]-st.gy;}}}
  if(b.turret){st=b.turret;var best=null,bd=st.p.range,me=center(i);st.types.forEach(function(id){var t=types[id];if(!t)return;t.insts.forEach(function(j){if(j.dead||j===i)return;var c2=center(j),dd=Math.hypot(c2[0]-me[0],c2[1]-me[1]);if(dd<=bd){bd=dd;best=j;}});});
   st.target=best;st.cd=Math.max(0,st.cd-dt);var aimed=true;
   if(best&&st.p.rotate){var tc=center(best),want=Math.atan2(tc[1]-me[1],tc[0]-me[0])*180/Math.PI,df=((want-i.angle)%360+540)%360-180,mx=st.p.rotSpeed>0?st.p.rotSpeed*dt:360;i.angle+=Math.max(-mx,Math.min(mx,df));aimed=Math.abs(df)<=Math.max(12,mx);}
   if(best&&aimed&&st.cd<=0){st.fired=tick;st.cd=st.p.rate;}}
  if(b.tween&&b.tween.list.length){st=b.tween;st.list=st.list.filter(function(tw){tw.t+=dt;var k=Math.min(1,tw.t/tw.d);tset(i,tw.prop,tw.from+(tw.to-tw.from)*ease(tw.ease,k));
    if(k<1)return true;st.done[tw.tag]=tick;if(tw.mode==='loop'){tw.t=0;return true;}if(tw.mode==='pingpong'){tw.t=0;var f=tw.from;tw.from=tw.to;tw.to=f;return true;}return false;});}
  if(b.fade&&b.fade.run){st=b.fade;var q=st.p;st.t+=dt;var t1=q.fadeIn,t2=t1+q.wait,t3=t2+q.fadeOut;
   if(st.t<t1)i.opacity=st.base*st.t/t1;else if(st.t<t2)i.opacity=st.base;else if(st.t<t3)i.opacity=st.base*(1-(st.t-t2)/q.fadeOut);else{i.opacity=q.fadeOut>0?0:st.base;st.run=false;st.done=tick;if(q.destroy)i._kill=true;}}
 }
 function pinStep(i){var st=i.b.pin,j=st.to;if(!j)return;if(j.dead){st.to=null;if(st.p.destroy)i.dead=true;return;}
  if(st.p.mirror)i.mirrored=st.mi?!j.mirrored:j.mirrored;if(st.p.angle)i.angle=(j.angle||0)+st.oa;
  if(st.pt){var q=imgPoint(j.t.def,j,st.pt);i.x=q.x;i.y=q.y;return;}
  var ux=st.ox,uy=st.oy;if(st.p.mirror&&j.mirrored)ux=-ux;if(st.p.angle&&j.angle){var r=j.angle*Math.PI/180,c=Math.cos(r),s=Math.sin(r),x2=ux*c-uy*s;uy=ux*s+uy*c;ux=x2;}i.x=j.x+ux;i.y=j.y+uy;}
 function approach(v,t,d){return v<t?Math.min(v+d,t):Math.max(v-d,t);}
 function canPush(b){var ph=b&&b.b&&b.b.physics;return !!(ph&&!ph.fixed&&ph.p.pushable&&!b.dead);}
 /* what a push moves: the row of bodies in front of the pushed one (crate against crate) + everything resting on them */
 function pushChain(b,dir,reach){var l=[b],m=physMass(b),k,n;while(l.length<12){var c=l[l.length-1],ox=c.x;moveX(c,dir*(reach+.5),c.b.physics.ground);var nb=lastBlk;c.x=ox;if(!canPush(nb)||l.indexOf(nb)>=0)break;l.push(nb);m+=physMass(nb);}
  l.forEach(function(q){q._plv=0;});var rid=[],seen=l.slice(),B=all.filter(function(o){return canPush(o)&&seen.indexOf(o)<0;});
  for(k=0;k<seen.length&&seen.length<30;k++){var sb=box(seen[k]);for(n=0;n<B.length;n++){var o=B[n];if(seen.indexOf(o)>=0)continue;var ob=box(o);if(ob.r>sb.l+1&&ob.l<sb.r-1&&Math.abs(ob.b-sb.t)<=2.5){seen.push(o);rid.push([o,seen[k],(seen[k]._plv||0)+1]);o._plv=(seen[k]._plv||0)+1;m+=physMass(o);}}}
  rid.sort(function(a,c){return a[2]-c[2]||(c[0].x-a[0].x)*dir;});return{l:l,r:rid,m:m};}
 /* front body first, each following one goes as far as the one ahead allowed; riders follow their support */
 function pushMove(ch,want){var l=ch.l,mv={},k;for(k=l.length-1;k>=0;k--){var b=l[k],x0=b.x;moveX(b,want,b.b.physics.ground);b._pmv=b.x-x0;}
  for(k=0;k<ch.r.length;k++){var o=ch.r[k][0],sup=ch.r[k][1],d=sup._pmv||0,x1=o.x;if(d)moveX(o,d,true);o._pmv=o.x-x1;}
  l.concat(ch.r.map(function(q){return q[0];})).forEach(function(b){var ph=b.b.physics;if(Math.abs(b._pmv)>.001){ph.vx=ph.p.shape==='circle'&&b===l[l.length-1]?b._pmv/DT:0;ph.pushT=tick;}b._pmv=0;});
  return l[0];}
 function platform(i,dt){
  var st=i.b.platform,p=st.p;st.landed=false;st.jumped=false;if(!st.en){st.sim={};return;}
  var c=p.controls&&!cineLock(),L=st.sim.left||(c&&(keys.ArrowLeft||keys.KeyA)),R=st.sim.right||(c&&(keys.ArrowRight||keys.KeyD)),
   J=st.hold||(c&&(keys.ArrowUp||keys.KeyW||keys.Space)),JP=st.sim.jump||(c&&(kp.ArrowUp||kp.KeyW||kp.Space)),U=st.sim.up||(c&&(keys.ArrowUp||keys.KeyW)),D=st.sim.down||(c&&(keys.ArrowDown||keys.KeyS));
  if(st.floor&&!st.floor.dead){if(st.floor.dy)i.y+=st.floor.dy;if(st.floor.dx)moveX(i,st.floor.dx);}
  var dir=(R?1:0)-(L?1:0);
  if(st.lock>0){st.lock-=dt;if(st.wallDir)dir=-st.wallDir;}
  if(p.climbSpeed>0&&ladders.length){var b0=box(i),cx0=(b0.l+b0.r)/2,inL=function(lb){return cx0>=lb.l-2&&cx0<=lb.r+2;};
   if(!st.climb){for(var q=0;q<ladders.length;q++){var ld=ladders[q];if(ld.dead)continue;var lb=box(ld);if(!inL(lb))continue;
     if(U&&b0.b>lb.t+2&&b0.t<lb.b){st.climb=ld;break;}if(D&&st.floor===ld&&Math.abs(b0.b-lb.t)<=3){st.climb=ld;i.y+=3;break;}}}
   if(st.climb){var cl=st.climb,clb=box(cl),JS=st.sim.jump||(c&&kp.Space);
    if(cl.dead||!inL(clb)||b0.t>=clb.b){st.climb=null;}
    else if(JS){st.climb=null;st.vy=-p.jump*.85;st.jumped=true;st.floor=null;st.jumps=1;st.buf=0;}
    else{var cs=p.climbSpeed,cvy=((D?1:0)-(U?1:0))*cs;st.vx=dir*cs*.7;st.vy=0;st.floor=null;i._nojt=true;moveX(i,st.vx*dt,false);var hit=moveY(i,cvy*dt);i._nojt=false;
     var bb=box(i);if(cvy<0&&bb.b<=clb.t+.5){i.y+=clb.t-bb.b;st.climb=null;st.floor=cl;}else if(cvy>0&&hit){st.climb=null;st.floor=hit;}
     st.coyote=0;st.sim={};return;}}}
  if(dir){var a=p.accel;if(st.vx*dir<0)a+=p.decel;st.vx=approach(st.vx,dir*p.maxSpeed,a*dt);}else st.vx=approach(st.vx,0,p.decel*dt);
  if(Math.abs(st.vx)>p.maxSpeed)st.vx=approach(st.vx,dir*p.maxSpeed,p.decel*dt);
  st.coyote=st.floor?.1:st.coyote-dt;if(JP)st.buf=.12;else st.buf-=dt;if(st.wallT>0)st.wallT-=dt;
  if(st.buf>0&&p.wallJump&&!st.floor&&st.coyote<=0&&st.wallT>0){st.vy=-p.jump;st.vx=-st.wallDir*p.wallKick;st.lock=.16;st.jumped=true;st.wj=tick;st.buf=0;st.wallT=0;st.jumps=1;st.hold=!!st.sim.jump;}
  if(st.buf>0){if(st.floor||st.coyote>0){st.vy=-p.jump;st.jumped=true;st.buf=0;st.coyote=0;st.floor=null;st.jumps=1;st.hold=!!st.sim.jump;}
   else if(p.doubleJump&&st.jumps<2){st.vy=-p.jump*.9;st.jumped=true;st.buf=0;st.jumps=2;st.hold=!!st.sim.jump;}}
  if(st.vy>=0)st.hold=false;
  var g=p.gravity;if(st.vy<0&&!J&&!st.hold)g*=2.4;
  st.vy=Math.min(st.vy+g*dt,p.maxFall);
  var was=!!st.floor,gr=was&&st.vy>=0;i._sf=null;
  var pvx=st.vx,hx0=i.x,dxw=st.vx*dt,wallHit=moveX(i,dxw,gr);st.pushing=null;if(wallHit){var pb=lastBlk;if(pb&&pb.b&&pb.b.physics&&!pb.b.physics.fixed&&pb.b.physics.p.pushable&&dir&&dir===wallHit&&st.floor!==pb){var ph=pb.b.physics,rem=dxw-(i.x-hx0),ch=pushChain(pb,dir,Math.abs(rem)),ps=Math.min(p.maxSpeed*.75,260/Math.sqrt(Math.max(.2,ch.m))),want=dir*Math.min(Math.abs(rem),ps*dt),bx0=pb.x;
   if(want){pushMove(ch,want);var mv=pb.x-bx0;if(Math.abs(mv)>.001)moveX(i,mv,gr);}st.pushing=pb;st.vx=dir*Math.min(Math.abs(pvx),ps);}else st.vx=0;}
  if(!st.floor&&dir&&(p.wallJump||p.wallSlide>0)){var blk=wallHit&&st.vx===0;if(!blk){var ox=i.x;if(moveX(i,dir*.6,false))blk=true;i.x=ox;}if(blk&&(st.lock<=0||dir!==-st.wallDir)){st.wallT=.1;st.wallDir=dir;}}
  if(p.wallSlide>0&&st.wallT>0&&!st.floor&&st.vy>p.wallSlide&&dir===st.wallDir)st.vy=p.wallSlide;
  var f=moveY(i,st.vy*dt);
  if(st.vy>0){st.floor=f;if(f)st.vy=0;}else{st.floor=null;if(f&&st.vy<0)st.vy=0;}
  if(!st.floor&&gr){var gs=groundSnap(i,climbOf(box(i),st.vx*dt));if(gs){st.floor=gs;st.vy=0;}}
  if(!st.floor&&i._sf&&st.vy>=0){st.floor=i._sf;st.vy=0;}
  if(st.floor&&!was){st.landed=true;st.jumps=0;}
  st.sim={};
 }
 function pathStep(i,dt){var st=i.b.path,P=st.pts,n=P.length;if(!st.en||n<2)return;if(st.wt>0){st.wt-=dt;return;}
  var a=P[st.k],b=P[st.nx],len=Math.hypot(b[0]-a[0],b[1]-a[1]);st.u+=len>.001?st.p.speed*dt/len:1;
  if(st.u>=1){st.u=0;st.last=st.nx;st.arr=tick;st.wt=st.p.pause||0;var m=st.p.mode;
   if(m==='loop'){st.k=st.nx;st.nx=(st.nx+1)%n;}
   else if(m==='once'){if(st.nx>=n-1){st.en=false;st.fin=tick;st.k=st.nx;}else{st.k=st.nx;st.nx++;}}
   else{st.k=st.nx;if(st.nx>=n-1)st.dir=-1;else if(st.nx<=0)st.dir=1;st.nx=st.k+st.dir;}
   i.x=st.bx+b[0];i.y=st.by+b[1];return;}
  var e=st.p.ease?st.u*st.u*(3-2*st.u):st.u;i.x=st.bx+a[0]+(b[0]-a[0])*e;i.y=st.by+a[1]+(b[1]-a[1])*e;if(st.p.rotate&&len>.001)i.angle=Math.atan2(b[1]-a[1],b[0]-a[0])*180/Math.PI;}
 function physC(i){var b=box(i);return{x:(b.l+b.r)/2,y:(b.t+b.b)/2};}
 function physMass(i){var b=box(i),ph=i.b.physics;return Math.max(.01,(b.r-b.l)*(b.b-b.t)/1024*(ph?ph.p.density:1));}
 function physInv(i){var ph=i.b.physics;return !ph||ph.fixed?0:1/physMass(i);}
 function physShape(i){var b=box(i),ph=i.b.physics;if(ph&&ph.p.shape==='circle'){return{c:true,x:(b.l+b.r)/2,y:(b.t+b.b)/2,r:Math.min(b.r-b.l,b.b-b.t)/2,b:b};}return{c:false,b:b};}
 function boxBox(A,B){var ox=Math.min(A.r,B.r)-Math.max(A.l,B.l),oy=Math.min(A.b,B.b)-Math.max(A.t,B.t);if(ox<=0||oy<=0)return null;var acx=(A.l+A.r)/2,bcx=(B.l+B.r)/2,acy=(A.t+A.b)/2,bcy=(B.t+B.b)/2;if(ox<oy)return{nx:acx<bcx?-1:1,ny:0,d:ox};return{nx:0,ny:acy<bcy?-1:1,d:oy};}
 function circBox(C,B){var qx=Math.max(B.l,Math.min(C.x,B.r)),qy=Math.max(B.t,Math.min(C.y,B.b)),dx=C.x-qx,dy=C.y-qy,d2=dx*dx+dy*dy;if(d2>C.r*C.r)return null;if(d2>1e-9){var d=Math.sqrt(d2);return{nx:dx/d,ny:dy/d,d:C.r-d};}
  var m=boxBox({l:C.x-C.r,r:C.x+C.r,t:C.y-C.r,b:C.y+C.r},B);return m;}
 function circPoly(C,P){var best=null,bd=1e9,inside=ptInPoly(C.x,C.y,P);for(var k=0;k<P.length;k++){var a=P[k],b=P[(k+1)%P.length],ex=b[0]-a[0],ey=b[1]-a[1],L2=ex*ex+ey*ey,t=L2?Math.max(0,Math.min(1,((C.x-a[0])*ex+(C.y-a[1])*ey)/L2)):0,qx=a[0]+ex*t,qy=a[1]+ey*t,d=Math.hypot(C.x-qx,C.y-qy);if(d<bd){bd=d;best=[qx,qy];}}
  if(!best)return null;if(!inside&&bd>=C.r)return null;var nx=C.x-best[0],ny=C.y-best[1],l=Math.hypot(nx,ny)||1;nx/=l;ny/=l;if(inside){nx=-nx;ny=-ny;return{nx:nx,ny:ny,d:C.r+bd};}return{nx:nx,ny:ny,d:C.r-bd};}
 function manifold(SA,SB,polyB){if(SA.c){if(polyB)return circPoly(SA,polyB);if(SB.c){var dx=SA.x-SB.x,dy=SA.y-SB.y,d=Math.hypot(dx,dy),rr=SA.r+SB.r;if(d>=rr)return null;d=d||.0001;return{nx:dx/d,ny:dy/d,d:rr-d};}return circBox(SA,SB.b);}
  if(SB.c){var m=circBox(SB,SA.b);return m?{nx:-m.nx,ny:-m.ny,d:m.d}:null;}return boxBox(SA.b,polyB?polyBB(polyB):SB.b);}
 function physResolve(A,B,m,ia,ib,isStatic){var pa=A.b.physics,pb=B&&B.b&&B.b.physics,tot=ia+ib;if(tot<=0)return;var corr=Math.max(0,m.d-.05)/tot*.9;A.x+=m.nx*corr*ia;A.y+=m.ny*corr*ia;if(B&&!isStatic&&ib>0){B.x-=m.nx*corr*ib;B.y-=m.ny*corr*ib;}
  var bvx=pb?pb.vx:(B&&B.dx?B.dx*60:0),bvy=pb?pb.vy:(B&&B.dy?B.dy*60:0),rvx=pa.vx-bvx,rvy=pa.vy-bvy,vn=rvx*m.nx+rvy*m.ny;if(m.ny<-.5)pa.ground=true;if(pb&&m.ny>.5)pb.ground=true;if(vn>=0)return;
  var e=Math.max(pa.p.bounce,pb?pb.p.bounce:0);if(-vn<60)e=0;var j=-(1+e)*vn/tot;pa.vx+=j*m.nx*ia;pa.vy+=j*m.ny*ia;if(pb&&ib>0){pb.vx-=j*m.nx*ib;pb.vy-=j*m.ny*ib;}
  if(-vn>pa.hitV||pa.hitT!==tick){pa.hitT=tick;pa.hitV=-vn;}if(pb){pb.hitT=tick;pb.hitV=Math.max(pb.hitV,-vn);}
  var tx=-m.ny,ty=m.nx,vt=rvx*tx+rvy*ty,mu=Math.sqrt(pa.p.friction*(pb?pb.p.friction:.6)),jt=-vt/tot,lim=Math.abs(j)*mu;if(jt>lim)jt=lim;if(jt<-lim)jt=-lim;pa.vx+=jt*tx*ia;pa.vy+=jt*ty*ia;if(pb&&ib>0){pb.vx-=jt*tx*ib;pb.vy-=jt*ty*ib;}}
 function physStep(dt){var B=all.filter(function(i){return !i.dead&&i.b.physics;});if(!B.length&&!joints.length)return;joints=joints.filter(function(j){return !j.a.dead&&(!j.b||!j.b.dead);});
  var SUB=2,h=dt/SUB;for(var sub2=0;sub2<SUB;sub2++){
   B.forEach(function(i){var ph=i.b.physics;ph.ground=false;if(ph.fixed){ph.vx=ph.vy=0;return;}ph.vy=Math.min(ph.vy+ph.p.gravity*h,1600);ph.vx*=.9995;ph.vy*=.9995;i.x+=ph.vx*h;i.y+=ph.vy*h;});
   var stat=solids.filter(function(s){return !s.dead&&!(s.b&&s.b.physics);}),chars=all.filter(function(c){return !c.dead&&(c.b.platform||c.b.patrol||c.b.eightdir||c.b.car);});
   for(var it=0;it<3;it++){
    B.forEach(function(i){var ph=i.b.physics;if(ph.fixed)return;var ia=physInv(i);
     stat.forEach(function(s){var SA=physShape(i),sh=sshape(s),sb=sh?sh.b:box(s);if(SA.b.r<sb.l||SA.b.l>sb.r||SA.b.b<sb.t||SA.b.t>sb.b)return;var m=manifold(SA,{c:false,b:sb},sh?sh.p:null);if(m)physResolve(i,s,m,ia,0,true);});
     jts.forEach(function(s){if(s.dead||s===i||s.t.bh.ladder)return;var SA=physShape(i),sb=box(s);if(ph.vy<0||SA.b.b>sb.t+Math.max(6,ph.vy*h+2)||SA.b.r<=sb.l||SA.b.l>=sb.r||SA.b.b<sb.t)return;physResolve(i,s,{nx:0,ny:-1,d:SA.b.b-sb.t},ia,0,true);});
     chars.forEach(function(c){var st=c.b.platform||c.b.patrol;if(st&&st.floor===i)return;var SA=physShape(i),cb=box(c);var m=manifold(SA,{c:false,b:cb});if(m&&m.ny<.3)physResolve(i,c,m,ia,0,true);});});
    for(var a=0;a<B.length;a++)for(var b2=a+1;b2<B.length;b2++){var A=B[a],C=B[b2],ia2=physInv(A),ib2=physInv(C);if(ia2+ib2<=0)continue;var m2=manifold(physShape(A),physShape(C));if(m2){if(ia2>0)physResolve(A,C,m2,ia2,ib2,false);else physResolve(C,A,{nx:-m2.nx,ny:-m2.ny,d:m2.d},ib2,0,true);}}
    joints.forEach(function(j){var ca=physC(j.a),ia3=physInv(j.a),bx,by,ib3=0;if(j.b){var cb=physC(j.b);bx=cb.x;by=cb.y;ib3=physInv(j.b);}else{bx=j.px;by=j.py;}var dx=ca.x-bx,dy=ca.y-by,d=Math.hypot(dx,dy)||.0001;if(d<=j.len&&!j.rigid)return;var tot=ia3+ib3;if(tot<=0)return;var nx=dx/d,ny=dy/d,err=d-j.len;
     j.a.x-=nx*err*ia3/tot;j.a.y-=ny*err*ia3/tot;if(j.b){j.b.x+=nx*err*ib3/tot;j.b.y+=ny*err*ib3/tot;}var pa=j.a.b.physics,pbb=j.b&&j.b.b.physics,rv=((pa?pa.vx:0)-(pbb?pbb.vx:0))*nx+((pa?pa.vy:0)-(pbb?pbb.vy:0))*ny;if(rv>0||j.rigid){var jj=rv/tot;if(pa){pa.vx-=jj*nx*ia3;pa.vy-=jj*ny*ia3;}if(pbb){pbb.vx+=jj*nx*ib3;pbb.vy+=jj*ny*ib3;}}});}}
  B.forEach(function(i){var ph=i.b.physics,bb=box(i);if(bb.l<0){i.x-=bb.l;ph.vx=Math.abs(ph.vx)*ph.p.bounce;}else if(bb.r>layout.w){i.x-=bb.r-layout.w;ph.vx=-Math.abs(ph.vx)*ph.p.bounce;}if(bb.t>layout.h+400&&!joints.some(function(j){return j.a===i||j.b===i;}))i.dead=true;if(ph.ground&&Math.abs(ph.vx)<4)ph.vx=0;if(ph.p.shape==='circle'&&!ph.fixed){var r=Math.min(i.w,i.h)/2;if(ph.ground||Math.abs(ph.vx)>1)i.angle=((i.angle||0)+ph.vx*dt/Math.max(1,r)*180/Math.PI)%360;}});}
 function drawJoints(c,li){joints.forEach(function(j){if(j.a.layer!==li||j.a.dead)return;var a=physC(j.a),b=j.b?physC(j.b):{x:j.px,y:j.py};c.strokeStyle='#5a3a22';c.lineWidth=2.5;c.lineCap='round';c.beginPath();c.moveTo(a.x,a.y);c.lineTo(b.x,b.y);c.stroke();c.strokeStyle='#c98f4f';c.lineWidth=1.2;c.beginPath();c.moveTo(a.x,a.y);c.lineTo(b.x,b.y);c.stroke();if(!j.b){c.fillStyle='#3b3f4a';c.fillRect(b.x-4,b.y-3,8,6);}});}
 function ledgeOk(i,dir){var b=box(i),x=dir>0?b.r+2:b.l-2,drop=polyCount?(b.r-b.l)/2*1.25+8:4;for(var dy=2;dy<=drop;dy+=2)if(pointSolid(x,b.b+dy))return true;return false;}
 function patrol(i,dt){
  var st=i.b.patrol,p=st.p;st.vy=Math.min(st.vy+p.gravity*dt,1200);
  var tgt=null,spd=p.speed,mv=st.dir,b0=box(i),cx=(b0.l+b0.r)/2,cy=(b0.t+b0.b)/2,was=st.chasing;st.chasing=false;
  if(p.chase){var tt=byName[p.chase],bd=1e9;if(tt)tt.insts.forEach(function(o){if(o.dead||o===i||!o.visible)return;var ob=box(o),d=Math.hypot((ob.l+ob.r)/2-cx,(ob.t+ob.b)/2-cy);if(d<bd){bd=d;tgt=o;}});if(tgt&&bd>(p.range||0)*(was?1.25:1))tgt=null;}
  if(tgt){st.chasing=true;if(!was)st.spot=tick;var tb=box(tgt),dx=(tb.l+tb.r)/2-cx;if(Math.abs(dx)>6)st.dir=dx>0?1:-1;mv=Math.abs(dx)>6?st.dir:0;spd=p.chaseSpeed;
   if(st.floor&&mv&&!ledgeOk(i,mv)&&!(tb.t>b0.b+8)){if(p.jump>0){st.vy=-p.jump;st.floor=null;}else mv=0;}}
  var pg=!!st.floor;
  if(mv&&moveX(i,mv*spd*dt,pg)){if(tgt){if(pg&&p.jump>0&&st.vy>=0){st.vy=-p.jump;st.floor=null;}}else st.dir*=-1;}
  var f=moveY(i,st.vy*dt);if(st.vy>0){st.floor=f;if(f)st.vy=0;}else if(f)st.vy=0;else if(st.vy<0)st.floor=null;
  if(!st.floor&&pg&&st.vy>=0){var gs=groundSnap(i,climbOf(box(i),spd*dt));if(gs){st.floor=gs;st.vy=0;}}
  if(!tgt&&st.floor&&p.ledges&&!ledgeOk(i,st.dir))st.dir*=-1;
  if(p.flip&&i.t.def.kind==='sprite')i.mirrored=st.dir<0;
 }
 function animate(i,dt){
  var o=i.t.def;if(o.kind!=='sprite')return;var a=o.anims[i.anim];if(!a||a.frames.length<2||!(a.speed>0))return;
  i.ft+=dt*a.speed;while(i.ft>=1){i.ft-=1;i.frame++;if(i.frame>=a.frames.length){if(a.loop!==false)i.frame=0;else{i.frame=a.frames.length-1;i.ft=0;if(i.animEnd!==tick)i.animEnd=tick;break;}}}
 }

 /* ---- expressions ---- */
 function prop(i,p){var o=i.t.def;switch(p){
  case 'X':return i.x;case 'Y':return i.y;case 'Width':return i.w;case 'Height':return i.h;case 'Angle':return i.angle;case 'Opacity':return Math.round(i.opacity*100);
  case 'UID':return i.uid;case 'AnimationName':return o.kind==='sprite'&&o.anims[i.anim]?o.anims[i.anim].name:'';case 'AnimationFrame':return i.frame;case 'Text':return i.text;
  case 'Distance':return i.rd?i.rd.dist:0;case 'Lap':return i.rd?i.rd.lap:0;case 'LapTime':return i.rd?i.rd.lapTime:0;case 'LastLap':return i.rd?i.rd.last:0;case 'BestLap':return i.rd?i.rd.best:0;case 'Rank':return i.rd?rdRank(i):0;case 'Rivals':return i.rd?i.rd.cars.length:0;case 'PlayerX':return i.rd?i.rd.x:0;case 'Curve':return i.rd?rdCurve(i):0;case 'SkyOffset':return i.rd?i.rd.sky:0;case 'Progress':return i.rd?rdProg(i):0;case 'Steer':return i.rd?i.rd.steer:0;case 'Horizon':return i.rd?i.y+(i.rd.hz==null?i.h/2:i.rd.hz):0;
  case 'Speed':return i.rd?rdKmh(i):i.b.car?Math.abs(i.b.car.sp):i.b.eightdir?Math.hypot(i.b.eightdir.vx,i.b.eightdir.vy):i.b.platform?Math.abs(i.b.platform.vx):i.b.bullet?Math.hypot(i.b.bullet.vx,i.b.bullet.vy):i.b.physics?Math.hypot(i.b.physics.vx,i.b.physics.vy):0;case 'VectorX':return i.b.car?i.b.car.vx:i.b.platform?i.b.platform.vx:i.b.bullet?i.b.bullet.vx:i.b.physics?i.b.physics.vx:0;case 'VectorY':return i.b.car?i.b.car.vy:i.b.platform?i.b.platform.vy:i.b.bullet?i.b.bullet.vy:i.b.physics?i.b.physics.vy:0;
  case 'IsOnFloor':return i.b.platform&&i.b.platform.floor?1:0;
  case 'PathPoint':return i.b.path?i.b.path.last:0;case 'Value':return i.bar?i.bar.v:0;case 'MaxValue':return i.bar?i.bar.max:0;case 'ParticleCount':return i.ps?i.ps.parts.length:0;case 'Columns':return i.tm?i.tm.cols:0;case 'Rows':return i.tm?i.tm.rows:0;}
  return p in i.vars?i.vars[p]:0;}
 var ENV={
  o:function(n,p){var i;var dtp=byName[n];if(dtp&&(dtp.def.kind==='array'||dtp.def.kind==='dict')){var dd=DATA[dtp.id];if(p==='Length')return dd.list?dd.list.length:Object.keys(dd.map).length;if(p==='KeyCount')return dd.map?Object.keys(dd.map).length:dd.list.length;if(p==='AsJSON')return JSON.stringify(dd.list||dd.map);return 0;}
   if(n==='Mouse'){var ml=mainLayer();return p==='ScreenX'?mouse.sx:p==='ScreenY'?mouse.sy:p==='X'?mouse.sx/zoom+cam.x*ml.px:p==='Y'?mouse.sy/zoom+cam.y*ml.py:0;}if(n==='Self')i=cur;else{var t=byName[n];if(!t)return 0;if(p==='Count')return t.insts.filter(alive).length;if(cur&&inType(cur,t))i=cur;else i=picked(curSol,t)[0];}return i?prop(i,p):0;},
  m:function(n,p,a){var i;var dtm=byName[n];if(dtm&&(dtm.def.kind==='array'||dtm.def.kind==='dict')){var dm=DATA[dtm.id];if(p==='At'){var av=dm.list?dm.list[Math.floor(num(a[0]))]:undefined;return av==null?0:av;}if(p==='IndexOf'){var iv=dval(a[0]);return dm.list?dm.list.findIndex(function(x){return x==iv;}):-1;}if(p==='Get'){var gv=dm.map?dm.map[String(a[0])]:undefined;return gv==null?0:gv;}if(p==='Has')return dm.map&&String(a[0]) in dm.map?1:0;return 0;}
   if(n==='Self')i=cur;else{var t=byName[n];if(!t)return 0;if(cur&&inType(cur,t))i=cur;else i=picked(curSol,t)[0];}if(!i)return 0;if(p==='TileAt')return tmGet(i,num(a[0]),num(a[1]));if(p==='TileAtXY')return i.tm?tmGet(i,tmCol(i,num(a[0])),tmRow(i,num(a[1]))):-1;if(p==='ColumnAt')return i.tm?tmCol(i,num(a[0])):0;if(p==='RowAt')return i.tm?tmRow(i,num(a[0])):0;var q=imgPoint(i.t.def,i,a[0]);return p==='ImagePointX'?q.x:p==='ImagePointY'?q.y:0;},
  g:function(n){var vs=findVarScope(n);if(vs)return vs.vars[n];if(n in globals)return globals[n];switch(n){case 'newline':return '\n';case 'dt':return DT;case 'time':return time;case 'TickCount':return tick;case 'LayoutWidth':return layout.w;case 'LayoutHeight':return layout.h;case 'LayoutName':return layout.name;case 'ViewportWidth':return W;case 'ViewportHeight':return H;case 'ScrollX':return cam.x;case 'ScrollY':return cam.y;case 'pi':return Math.PI;case 'PlayerName':return hsName;case 'LastRank':return hsRank;case 'CurValue':for(var lq=loopStack.length-1;lq>=0;lq--)if('val' in loopStack[lq])return loopStack[lq].val;return 0;case 'CurKey':for(var lk=loopStack.length-1;lk>=0;lk--)if('key' in loopStack[lk])return loopStack[lk].key;return '';case 'CameraZoom':return Math.round(zoom*100);case 'CheckpointX':return cp&&cp.layout===layout.id?cp.x:0;case 'CheckpointY':return cp&&cp.layout===layout.id?cp.y:0;case 'DialogueChoice':return dlg.choice;case 'DialogueChoiceText':return dlg.choiceText;case 'TimeScale':return Math.round(tscale*100);case 'loopindex':return loopStack.length?loopStack[loopStack.length-1].i:0;}return 0;}
 };
 function X(src){try{return compile(src)(ENV,F);}catch(e){if(errors.length<20&&errors.indexOf(e.message)<0)errors.push(e.message);return 0;}}
 function num(v){return +v||0;}

 /* ---- conditions ---- */
 var SYSC={
  start:function(){return firstTick;},tick:function(){return true;},
  ctlopen:function(){return ctl.open;},
  gamepaused:function(){return gpaused;},
  hsqual:function(p){var L=hsList(p[0]),mx=Math.max(1,Math.floor(num(X(p[2])))||10);return hsFits(L,num(X(p[1])),mx,p[3]==='asc');},hsnamed:function(){return hsNamedTick===tick;},hsopen:function(){return !!hsIn;},
  cineplaying:function(){return !!cine;},cineended:function(p){return cineEndTick===tick&&(!p[0]||p[0]===cineEndId);},
  hassave:function(p){return !!sget('save:'+(p[0]||'partie'));},
  hascp:function(){return !!cp&&cp.layout===layout.id;},
  dlgopen:function(){return !!dlg.cur;},dlgend:function(){return dlg.endTick===tick;},dlgchoice:function(p){var n=Math.floor(num(X(p[0])));return dlg.choiceTick===tick&&(n<=0||n===dlg.choice);},
  every:function(p,c){var s=cstate[c.id]||(cstate[c.id]={t:0});s.t+=DT;if(s.t>=num(X(p[0]))-1e-9){s.t=0;return true;}return false;},
  compare:function(p){return cmp(X(p[0]),p[1],X(p[2]));},
  gcmp:function(p){return cmp(getVar(p[0]),p[1],X(p[2]));},
  groupactive:function(p){var g=groupsByName[p[0]];return !!g&&g.some(groupOn);},
  chance:function(p){return Math.random()*100<num(X(p[0]));},
  islayout:function(p){return layout.id===p[0];},
  pickall:function(p,c,sol){var t=types[p[0]];if(!t)return false;delete sol[t.id];return true;},
  count:function(p){var t=types[p[0]];return cmp(t?t.insts.filter(alive).length:0,p[1],X(p[2]));}
 };
 var MOUSEC={click:function(p){return !!mouse.pr[p[0]];},down:function(p){return !!mouse.down[p[0]];},released:function(p){return !!mouse.rl[p[0]];}};
 var AUDC={musicplaying:function(){return !!music;},tagplaying:function(p){return voices.some(function(v){return v.tag===p[0];});}};
 var STC={exists:function(p){return sget(p[0])!==undefined;}};
 var KEYC={pressed:function(p){return !!kp[p[0]];},down:function(p){return !!keys[p[0]];},released:function(p){return !!kr[p[0]];},any:function(){return anyPress;}};
 var OBJC={
  common:{
   cmpx:function(i,p){return cmp(i.x,p[0],X(p[1]));},cmpy:function(i,p){return cmp(i.y,p[0],X(p[1]));},
   onscreen:function(i){var b=drawBox(i.t.def,i),l=layers[i.layer]||{px:1,py:1};if(i.hud)return true;var ox=cam.x*l.px,oy=cam.y*l.py,z=zf(l,i);return b.r>ox&&b.l<ox+W/z&&b.b>oy&&b.t<oy+H/z;},
   onlayer:function(i,p){var l=layers[i.layer];return !!l&&l.name===p[0];},
   pickby:function(i,p){return cmp(X(p[0]),p[1],X(p[2]));},
   outside:function(i){var b=drawBox(i.t.def,i);return b.r<0||b.l>layout.w||b.b<0||b.t>layout.h;},
   ivar:function(i,p){return cmp(i.vars[p[0]],p[1],X(p[2]));},visible:function(i){return i.visible;}
  },
  sprite:{
   animis:function(i,p){var a=i.t.def.anims[i.anim];return !!a&&a.name===p[0];},
   animend:function(i,p){var a=i.t.def.anims[i.anim];return i.animEnd===tick&&(!p[0]||(a&&a.name===p[0]));},
   mirrored:function(i){return i.mirrored;}
  },
  text:{cmptext:function(i,p){return String(i.text)===String(X(p[0]));}},
  'b:platform':{
   floor:function(i){return !!i.b.platform.floor;},onobj:function(i,p){var f=i.b.platform.floor;return !!f&&!f.dead&&f.t.id===p[0];},moving:function(i){return Math.abs(i.b.platform.vx)>1;},
   jumping:function(i){var s=i.b.platform;return !s.floor&&!s.climb&&s.vy<0;},falling:function(i){var s=i.b.platform;return !s.floor&&!s.climb&&s.vy>0;},onwall:function(i){var s=i.b.platform;return !s.floor&&s.wallT>0;},walljumped:function(i){return i.b.platform.wj===tick;},climbing:function(i){return !!i.b.platform.climb;},
   landed:function(i){return i.b.platform.landed;},jumped:function(i){return i.b.platform.jumped;}
  },
  'b:patrol':{goingleft:function(i){return i.b.patrol.dir<0;},chasing:function(i){return !!i.b.patrol.chasing;},spotted:function(i){return i.b.patrol.spot===tick;}},
  'b:timer':{ontimer:function(i,p){return i.b.timer.fired[p[0]]===tick;},running:function(i,p){return !!i.b.timer.timers[p[0]];}},
  'b:eightdir':{moving:function(i){var s=i.b.eightdir;return Math.abs(s.vx)+Math.abs(s.vy)>1;}},
  'b:car':{moving:function(i){return Math.abs(i.b.car.sp)>4;},reversing:function(i){return i.b.car.sp<-4;},driven:function(i){return !!i.b.car.ctl;},hit:function(i,p){return i.b.car.hit===tick&&i.b.car.hitV>=num(X(p[0]));},cmpspeed:function(i,p){return cmp(Math.abs(i.b.car.sp),p[0],X(p[1]));}},
  tilemap:{cmptile:function(i,p){return cmp(tmGet(i,num(X(p[0])),num(X(p[1]))),p[2],num(X(p[3])));},cmptileat:function(i,p){return cmp(tmGet(i,tmCol(i,num(X(p[0]))),tmRow(i,num(X(p[1])))),p[2],num(X(p[3])));}},
  particles:{empty:function(i){return !i.ps.parts.length;}},
  road3d:{lapdone:function(i){return i.rd.lapT===tick;},hit:function(i){return i.rd.hitT===tick;},offroad:function(i){return Math.abs(i.rd.x)>1;},cmpspeed:function(i,p){return cmp(rdKmh(i),p[0],X(p[1]));},driven:function(i){return !!i.rd.ctl;}},
  bar:{cmpvalue:function(i,p){return cmp(i.bar.v,p[0],X(p[1]));},empty:function(i){return i.bar.v<=0;},full:function(i){return i.bar.v>=i.bar.max;}},
  button:{clicked:function(i){return i.btn.clicked===tick;},hover:function(i){return i.btn.hover;},focused:function(i){return i.btn.focus;}},
  'b:tween':{done:function(i,p){return i.b.tween.done[p[0]]===tick;},running:function(i,p){return i.b.tween.list.some(function(t){return t.tag===p[0];});}},
  'b:fade':{done:function(i){return i.b.fade.done===tick;}},
  'b:physics':{moving:function(i){var b=i.b.physics;return Math.abs(b.vx)+Math.abs(b.vy)>12;},onground:function(i){return i.b.physics.ground;},hit:function(i,p){var b=i.b.physics;return b.hitT===tick&&b.hitV>=num(X(p[0]));},roped:function(i){return joints.some(function(j){return j.a===i||j.b===i;});}},
  'b:path':{arrived:function(i){return i.b.path.arr===tick;},finished:function(i){return i.b.path.fin===tick;},moving:function(i){var st=i.b.path;return st.en&&st.pts.length>1&&!(st.wt>0);}},
  'b:pin':{pinned:function(i){var t=i.b.pin.to;return !!t&&!t.dead;}},
  'b:drag':{dragging:function(i){return i.b.drag.on;},start:function(i){return i.b.drag.st===tick;},drop:function(i){return i.b.drag.dr===tick;}},
  'b:turret':{shoot:function(i){return i.b.turret.fired===tick;},target:function(i){var t=i.b.turret.target;return !!t&&!t.dead;}},
  'b:los':{sees:function(i,p){var t=types[p[0]];if(!t)return false;var seen=t.insts.filter(function(j){return alive(j)&&j!==i&&canSee(i,j);});if(seen.length&&curSol){var prev=curSol['_los'+t.id]||[];seen.forEach(function(j){if(prev.indexOf(j)<0)prev.push(j);});curSol['_los'+t.id]=prev;}return seen.length>0;}}
 };
 var DATAC={array:{contains:function(d,p){var v=dval(X(p[0]));return d.list.some(function(x){return x==v;});},empty:function(d){return !d.list.length;},cmpat:function(d,p){return cmp(d.list[Math.floor(num(X(p[0])))],p[1],X(p[2]));}},
  dict:{haskey:function(d,p){return String(X(p[0])) in d.map;},empty:function(d){return !Object.keys(d.map).length;},cmpkey:function(d,p){return cmp(d.map[String(X(p[0]))],p[1],X(p[2]));}}};
 var DATAA={array:{push:function(d,p){var v=dval(X(p[0]));if(p[1]==='start')d.list.unshift(v);else d.list.push(v);},set:function(d,p){var k=Math.floor(num(X(p[0])));if(k>=0&&k<100000){while(d.list.length<k)d.list.push(0);d.list[k]=dval(X(p[1]));}},
   insert:function(d,p){var k=Math.max(0,Math.min(d.list.length,Math.floor(num(X(p[0])))));d.list.splice(k,0,dval(X(p[1])));},removeat:function(d,p){var k=Math.floor(num(X(p[0])));if(k>=0&&k<d.list.length)d.list.splice(k,1);},
   removeval:function(d,p){var v=dval(X(p[0])),k=d.list.findIndex(function(x){return x==v;});if(k>=0)d.list.splice(k,1);},clear:function(d){d.list=[];},
   sort:function(d,p){d.list.sort(function(a,b){var r=typeof a==='number'&&typeof b==='number'?a-b:String(a).localeCompare(String(b));return p[0]==='desc'?-r:r;});},
   shuffle:function(d){for(var k=d.list.length-1;k>0;k--){var j=Math.random()*(k+1)|0,t=d.list[k];d.list[k]=d.list[j];d.list[j]=t;}},
   save:function(d,p){sset('arr:'+p[0],d.list);},load:function(d,p){var v=sget('arr:'+p[0]);if(Array.isArray(v))d.list=v;}},
  dict:{set:function(d,p){d.map[String(X(p[0]))]=dval(X(p[1]));},add:function(d,p){var k=String(X(p[0])),o=d.map[k];d.map[k]=typeof o==='string'?o+X(p[1]):num(o)+num(X(p[1]));},remove:function(d,p){delete d.map[String(X(p[0]))];},clear:function(d){d.map={};},
   save:function(d,p){sset('dict:'+p[0],d.map);},load:function(d,p){var v=sget('dict:'+p[0]);if(v&&typeof v==='object')d.map=v;}}};
 function pickOne(c,sol,t,key){var list=picked(sol,t);if(!list.length){sol[t.id]=[];return false;}var best=list[0];
  if(key==='prand')best=list[Math.random()*list.length|0];
  else{var x=num(X(c.p[0])),y=num(X(c.p[1])),bd=key==='pnear'?Infinity:-1;list.forEach(function(i){var d=(i.x-x)*(i.x-x)+(i.y-y)*(i.y-y);if(key==='pnear'?d<bd:d>bd){bd=d;best=i;}});}
  sol[t.id]=[best];return true;}
 function collCond(c,sol,t,trig){
  var o=types[c.p[0]];if(!o)return false;var A=picked(sol,t),B=picked(sol,o),st=cstate[c.id]||(cstate[c.id]={pairs:{}}),now={},ra=[],rb=[],a,b,n,m,ba;
  var SB=B.map(shape);
  if(c.not&&!trig){ra=A.filter(function(a){var ba=shape(a);return !B.some(function(b,k){return b!==a&&hits(ba,SB[k]);});});sol[t.id]=ra;return ra.length>0;}
  for(n=0;n<A.length;n++){a=A[n];ba=shape(a);var any=false;for(m=0;m<B.length;m++){b=B[m];if(a===b)continue;if(hits(ba,SB[m])){var key=a.uid+':'+b.uid;now[key]=1;if(!trig||!st.pairs[key]){any=true;if(rb.indexOf(b)<0)rb.push(b);}}}if(any)ra.push(a);}
  if(trig)st.pairs=now;
  if(!ra.length)return false;sol[t.id]=ra;if(o!==t)sol[o.id]=rb;return true;
 }
 function evalCond(c,sol,prevOk){
  if(c.off)return true;
  var dot=c.k.lastIndexOf('.'),pr=c.k.slice(0,dot),key=c.k.slice(dot+1),r;curSol=sol;cur=null;
  if(c.o==='system'){
   if(key==='else')return !prevOk;
   if(key==='once'){var s=cstate[c.id]||(cstate[c.id]={last:-5});r=s.last!==tick-1;s.last=tick;return r;}
   r=SYSC[key]?!!SYSC[key](c.p||[],c,sol):false;return c.not?!r:r;}
  if(c.o==='keyboard'){r=KEYC[key]?!!KEYC[key](c.p||[]):false;return c.not?!r:r;}
  if(c.o==='audio'){r=AUDC[key]?!!AUDC[key](c.p||[]):false;return c.not?!r:r;}
  if(c.o==='storage'){r=STC[key]?!!STC[key](c.p||[]):false;return c.not?!r:r;}
  if(c.o==='mouse'){var pp=c.p||[];
   if(key==='clickobj'||key==='overobj'){var mt=types[key==='clickobj'?pp[1]:pp[0]];if(!mt)return false;if(key==='clickobj'&&!mouse.pr[pp[0]])return false;
    var hl=picked(sol,mt).filter(function(i){return mouseOver(i)!==(!!c.not&&key==='overobj');});sol[mt.id]=hl;return hl.length>0;}
   r=MOUSEC[key]?!!MOUSEC[key](pp):false;return c.not?!r:r;}
  var t=types[c.o];if(!t)return false;
  if(t.def.kind==='array'||t.def.kind==='dict'){var df=DATAC[t.def.kind][key];r=df?!!df(DATA[t.id],c.p||[]):false;return c.not?!r:r;}
  if(key==='collide'||key==='overlap')return collCond(c,sol,t,key==='collide');
  if(key==='pnear'||key==='pfar'||key==='prand')return pickOne(c,sol,t,key);
  var fn=OBJC[pr]&&OBJC[pr][key];if(!fn)return false;
  var list=picked(sol,t),res=[],lt=pr==='b:los'?types[(c.p||[])[0]]:null;if(lt)delete sol['_los'+lt.id];
  for(var n=0;n<list.length;n++){var i=list[n];if(pr.indexOf('b:')===0&&!i.b[pr.slice(2)])continue;cur=i;if((!!fn(i,c.p||[],c))!==!!c.not)res.push(i);}
  if(lt){var seen=sol['_los'+lt.id];delete sol['_los'+lt.id];if(seen&&!c.not&&lt!==t)sol[lt.id]=seen;}
  cur=null;sol[t.id]=res;return res.length>0;
 }

 /* ---- actions ---- */
 function typed(old,v){return typeof old==='string'?String(v):num(v);}
 var SYSA={
  gset:function(p){setVar(p[0],typed(getVar(p[0]),X(p[1])));},
  gadd:function(p){var o=getVar(p[0]);setVar(p[0],typeof o==='string'?o+X(p[1]):num(o)+num(X(p[1])));},
  gsub:function(p){setVar(p[0],num(getVar(p[0]))-num(X(p[1])));},
  call:function(p){var args=String(p[1]||'').trim()?X('__a('+p[1]+')'):[];runFunc(p[0],Array.isArray(args)?args:[]);},
  retval:function(p){for(var k=scopes.length-1;k>=0;k--)if(scopes[k].fn){scopes[k].ret=X(p[0]);return;}},
  setgroup:function(p){(groupsByName[p[0]]||[]).forEach(function(g){groupState[g.id]=p[1]==='toggle'?!groupOn(g):p[1]==='on';});},
  resetg:function(){resetG();},
  gotolayout:function(p){if(p[0])pending=p[0];},
  controlsmenu:function(){ctlOpen();},
  pause:function(p){gpaused=p[0]==='pause'?true:p[0]==='resume'?false:!gpaused;},
  savegame:function(p){sset('save:'+(p[0]||'partie'),{layout:layout.id,globals:Object.assign({},globals),data:JSON.parse(JSON.stringify(DATA)),cp:cp,at:Date.now()});},
  loadgame:function(p){var sv=sget('save:'+(p[0]||'partie'));if(!sv||!P.layouts.some(function(L){return L.id===sv.layout;}))return;Object.keys(sv.globals||{}).forEach(function(k){if(k in globals)globals[k]=sv.globals[k];});Object.keys(sv.data||{}).forEach(function(k){if(DATA[k])DATA[k]=sv.data[k];});cpLoad=sv.cp||null;pending=sv.layout;},
  hsadd:function(p){var tb=String(p[0]||'scores'),L=hsList(tb),sc=num(X(p[1])),nm=String(X(p[2])).trim()||'???',mx=Math.max(1,Math.floor(num(X(p[3])))||10),asc=p[4]==='asc';L=L.slice(0,mx);if(!hsFits(L,sc,mx,asc)){hsRank=0;return;}
   var e={n:nm.slice(0,24),s:sc,d:Date.now()},k=0;while(k<L.length&&(asc?L[k].s<=sc:L[k].s>=sc))k++;L.splice(k,0,e);L=L.slice(0,mx);hsRank=k+1;hsLast={t:tb,d:e.d};sset('hs:'+tb,L);hsRefresh();},
  hsask:function(p){var mx=Math.max(1,Math.min(24,Math.floor(num(X(p[2])))||10)),nm=String(X(p[1])).slice(0,mx);hsIn={title:String(X(p[0])),name:nm,def:nm,max:mx,cur:nm.length,t:0,hit:null};},
  hsclear:function(p){sdel('hs:'+String(p[0]||'scores'));hsRefresh();},
  cineplay:function(p){cinePlay(p[0]);},cinestop:function(){cineSkip();},
  deletesave:function(p){sdel('save:'+(p[0]||'partie'));},
  camzoom:function(p){var z=Math.max(.1,Math.min(10,num(X(p[0]))/100)),d=Math.max(0,num(X(p[1])));if(d<=0){zoom=z;zoomA=null;}else zoomA={from:zoom,to:z,t:0,d:d};},
  camlook:function(p){camLook={x:num(X(p[0])),y:num(X(p[1]))};},camfollow:function(){camLook=null;},
  cambounds:function(p){camB={x:num(X(p[0])),y:num(X(p[1])),w:num(X(p[2])),h:num(X(p[3]))};},camnobounds:function(){camB=null;},
  setcp:function(p){cp={layout:layout.id,x:num(X(p[0])),y:num(X(p[1]))};},
  clearcp:function(){cp=null;},
  respawn:function(p){var t=types[p[0]];if(!t||!cp||cp.layout!==layout.id)return;t.insts.filter(alive).slice(0,1).forEach(function(i){i.x=cp.x;i.y=cp.y;var pl=i.b.platform;if(pl){pl.vx=0;pl.vy=0;pl.floor=null;pl.hold=false;}if(i.b.eightdir){i.b.eightdir.vx=0;i.b.eightdir.vy=0;}if(i.b.sine){i.b.sine.bx=i.x;i.b.sine.by=i.y;}});updateCam();},
  transition:function(p){trans={type:p[0]||'fade',dur:Math.max(0,num(X(p[1]))),color:String(p[2]||'#000000')};},
  darkness:function(p){dark={a:Math.max(0,Math.min(1,num(X(p[0]))/100)),c:String(p[1]||'#0b0b1e')};},
  timescale:function(p){tscale=Math.max(0,Math.min(4,num(X(p[0]))/100));},
  screenflash:function(p){var d=Math.max(.01,num(X(p[1])));sflash={c:String(p[0]||'#ffffff'),t:d,d:d};},
  say:function(p){dlgPush({name:String(X(p[0])),text:String(X(p[1])),portrait:p[2]||'',top:p[3]==='top'});},
  ask:function(p){var ch=String(X(p[2])).split('|').map(function(x){return x.trim();}).filter(function(x){return x.length;});dlgPush({name:String(X(p[0])),text:String(X(p[1])),portrait:p[3]||'',top:p[4]==='top',choices:ch.length?ch:['OK']});},
  dlgclose:function(){dlg.queue=[];if(dlg.cur){dlg.cur=null;dlg.endTick=tick;}},
  dlgspeed:function(p){dlg.speed=Math.max(0,num(X(p[0])));},
  next:function(){var k=P.layouts.indexOf(layout);pending=P.layouts[(k+1)%P.layouts.length].id;},
  restart:function(){pending=layout.id;},
  create:function(p,sol){var t=types[p[0]];if(!t)return;var li=p[3]?layerIdx(p[3]):-1;var i=createInst(t,null,num(X(p[1])),num(X(p[2])),li>=0?li:0);sol[t.id]=[i];},
  layervis:function(p){var k=layerIdx(p[0]);if(k>=0)layers[k].visible=!!p[1];},
  layeropa:function(p){var k=layerIdx(p[0]);if(k>=0)layers[k].opacity=Math.max(0,Math.min(1,num(X(p[1]))/100));},
  shake:function(p){shakeA=num(X(p[0]));shakeD=shakeT=Math.max(.01,num(X(p[1])));}
 };
 var OBJA={
  common:{
   destroy:function(i){i.dead=true;},
   spawn:function(i,p,sol){var t=types[p[0]];if(!t)return;var n=createInst(t,null,i.x,i.y,i.layer);sol[t.id]=[n];},
   tolayer:function(i,p){var k=layerIdx(p[0]);if(k>=0)i.layer=k;},
   posto:function(i,p,sol){var t=types[p[0]];if(!t)return;var j=(sol&&sol[t.id]&&sol[t.id].filter(alive)[0])||t.insts.filter(alive)[0];if(!j)return;var nm=p[1]==null?'':String(p[1]).trim();var q=imgPoint(t.def,j,nm);i.x=q.x;i.y=q.y;},
   setpos:function(i,p){var x=num(X(p[0])),y=num(X(p[1]));i.x=x;i.y=y;},setx:function(i,p){i.x=num(X(p[0]));},sety:function(i,p){i.y=num(X(p[0]));},
   move:function(i,p){i.x+=num(X(p[0]));i.y+=num(X(p[1]));},
   setsize:function(i,p){var w=num(X(p[0])),h=num(X(p[1]));i.w=w;i.h=h;},setangle:function(i,p){i.angle=num(X(p[0]));},
   setstyle:function(i,p){i.style=Object.assign({},i.style||{});i.style[p[0]]=String(p[1]||'');},setvis:function(i,p){i.visible=!!p[0];},setopa:function(i,p){i.opacity=Math.max(0,Math.min(1,num(X(p[0]))/100));},
   flash:function(i,p){i.flash=num(X(p[0]));},
   front:function(i){var k=all.indexOf(i);if(k>=0){all.splice(k,1);all.push(i);}},
   ivset:function(i,p){i.vars[p[0]]=typed(i.vars[p[0]],X(p[1]));},
   tint:function(i,p){var a=Math.max(0,Math.min(1,num(X(p[1]))/100));i.tint=a>0?{c:String(p[0]||'#ffffff'),a:a}:null;},
   tintflash:function(i,p){var d=Math.max(.01,num(X(p[1])));i.tf={c:String(p[0]||'#ffffff'),t:d,d:d};},
   ivadd:function(i,p){var o=i.vars[p[0]];i.vars[p[0]]=typeof o==='string'?o+X(p[1]):num(o)+num(X(p[1]));},
   ivsub:function(i,p){i.vars[p[0]]=num(i.vars[p[0]])-num(X(p[1]));}
  },
  sprite:{
   setanim:function(i,p){var a=i.t.def.anims,k=a.findIndex(function(x){return x.name===p[0];});if(k>=0&&k!==i.anim){i.anim=k;i.frame=0;i.ft=0;}},
   mirror:function(i,p){i.mirrored=!!p[0];},
   spawnpt:function(i,p,sol){var t=types[p[0]];if(!t)return;var q=imgPoint(i.t.def,i,p[1]);var n=createInst(t,null,q.x,q.y,i.layer);
    if(p[2]!=='none'){n.mirrored=i.mirrored;n.angle=i.angle;var u=n.b.bullet;if(u){var a=u.p.angle+i.angle;if(i.mirrored)a=180-a;a*=Math.PI/180;u.vx=Math.cos(a)*u.p.speed;u.vy=Math.sin(a)*u.p.speed;n.angle=i.mirrored?0:i.angle;}}
    sol[t.id]=[n];}
  },
  text:{settext:function(i,p){i.text=String(X(p[0]));},append:function(i,p){i.text=String(i.text)+X(p[0]);}},
  'b:platform':{
   simjump:function(i){i.b.platform.sim.jump=true;},simleft:function(i){i.b.platform.sim.left=true;},simright:function(i){i.b.platform.sim.right=true;},
   setvy:function(i,p){var s=i.b.platform;s.vy=num(X(p[0]));if(s.vy<0){s.floor=null;s.hold=true;}},setvx:function(i,p){i.b.platform.vx=num(X(p[0]));},
   setmax:function(i,p){i.b.platform.p.maxSpeed=num(X(p[0]));},setjump:function(i,p){i.b.platform.p.jump=num(X(p[0]));},
   enable:function(i,p){i.b.platform.en=!!p[0];},controls:function(i,p){i.b.platform.p.controls=!!p[0];}
  },
  'b:patrol':{setchase:function(i,p){var t=types[p[0]];i.b.patrol.p.chase=t?t.name:'';},reverse:function(i){i.b.patrol.dir*=-1;},speed:function(i,p){i.b.patrol.p.speed=num(X(p[0]));}},
  'b:bullet':{
   speed:function(i,p){var s=i.b.bullet,a=Math.atan2(s.vy,s.vx),v=num(X(p[0]));s.p.speed=v;s.vx=Math.cos(a)*v;s.vy=Math.sin(a)*v;},
   angle:function(i,p){var s=i.b.bullet,a=num(X(p[0]))*Math.PI/180;s.vx=Math.cos(a)*s.p.speed;s.vy=Math.sin(a)*s.p.speed;},
   bounceoff:function(i,p,sol){var t=types[p[0]];if(!t)return;var s=i.b.bullet,a=box(i),best=null,bo=0;picked(sol,t).forEach(function(o){if(o===i||o.dead)return;var b=box(o),ox=Math.min(a.r,b.r)-Math.max(a.l,b.l),oy=Math.min(a.b,b.b)-Math.max(a.t,b.t);if(ox>0&&oy>0&&ox*oy>bo){bo=ox*oy;best={b:b,ox:ox,oy:oy};}});if(!best)return;var b=best.b,acx=(a.l+a.r)/2,acy=(a.t+a.b)/2;
    if(best.ox<best.oy){var dx=acx<(b.l+b.r)/2?-best.ox:best.ox;i.x+=dx;s.vx=dx<0?-Math.abs(s.vx):Math.abs(s.vx);}else{var dy=acy<(b.t+b.b)/2?-best.oy:best.oy;i.y+=dy;s.vy=dy<0?-Math.abs(s.vy):Math.abs(s.vy);}}
  },
  'b:sine':{active:function(i,p){i.b.sine.en=!!p[0];}},
  'b:physics':{impulse:function(i,p){var b=i.b.physics,a=num(X(p[0]))*Math.PI/180,f=num(X(p[1]));b.vx+=Math.cos(a)*f;b.vy+=Math.sin(a)*f;b.sleep=0;},setvel:function(i,p){var b=i.b.physics;b.vx=num(X(p[0]));b.vy=num(X(p[1]));b.sleep=0;},immovable:function(i,p){i.b.physics.fixed=!!p[0];i.b.physics.vx=0;i.b.physics.vy=0;},
   ropeto:function(i,p,sol){var t=types[p[0]];if(!t)return;var o=picked(sol,t).filter(function(x){return x!==i&&!x.dead;})[0];if(!o)return;var L=num(X(p[1]));var ca=physC(i),cb=physC(o);joints.push({a:i,b:o,len:L>0?L:Math.hypot(ca.x-cb.x,ca.y-cb.y),rigid:!!p[2]});},
   ropepoint:function(i,p){var x=num(X(p[0])),y=num(X(p[1])),L=num(X(p[2])),ca=physC(i);joints.push({a:i,px:x,py:y,len:L>0?L:Math.hypot(ca.x-x,ca.y-y)});},
   cutropes:function(i){joints=joints.filter(function(j){return j.a!==i&&j.b!==i;});}},
  'b:path':{enable:function(i,p){i.b.path.en=!!p[0];},speed:function(i,p){i.b.path.p.speed=num(X(p[0]));},restart:function(i){var st=i.b.path;st.k=0;st.nx=1;st.dir=1;st.u=0;st.wt=0;i.x=st.bx;i.y=st.by;st.en=true;}},
  tilemap:{settile:function(i,p){tmSet(i,num(X(p[0])),num(X(p[1])),num(X(p[2])));},settileat:function(i,p){tmSet(i,tmCol(i,num(X(p[0]))),tmRow(i,num(X(p[1]))),num(X(p[2])));},
   fillrect:function(i,p){var c0=Math.floor(num(X(p[0]))),r0=Math.floor(num(X(p[1]))),c1=Math.floor(num(X(p[2]))),r1=Math.floor(num(X(p[3]))),v=num(X(p[4])),a,b;if(c1<c0){a=c0;c0=c1;c1=a;}if(r1<r0){b=r0;r0=r1;r1=b;}for(var r=r0;r<=r1;r++)for(var c=c0;c<=c1;c++)tmSet(i,c,r,v);}},
  minimap:{setradius:function(i,p){i.mmR=Math.max(50,num(X(p[0])));}},
  road3d:{sim:function(i,p){i.rd.sim[p[0]]=true;},controls:function(i,p){i.rd.ctl=!!p[0];},setspeed:function(i,p){i.rd.speed=Math.max(0,num(X(p[0]))/rdNum(i.t.def.maxKmh,220)*rdMax(i));},setmax:function(i,p){i.rd.maxKmh=Math.max(1,num(X(p[0])));},reset:function(i){rdInit(i,true);},settrack:function(i,p){i.t.def=Object.assign({},i.t.def,{track:String(X(p[0])).replace(/;/g,'\n')});rdInit(i,true);}},
  bar:{setvalue:function(i,p){barSet(i,num(X(p[0])));},addvalue:function(i,p){barSet(i,i.bar.v+num(X(p[0])));},setmax:function(i,p){i.bar.max=Math.max(.0001,num(X(p[0])));barSet(i,i.bar.v);},setcolor:function(i,p){i.bar.col=String(p[0]||'');}},
 leaderboard:{settable:function(i,p){i.lbt=String(X(p[0]));i.lb=hsList(i.lbt);}},
 button:{settext:function(i,p){i.text=String(X(p[0]));},enable:function(i,p){i.btn.en=!!p[0];if(!i.btn.en&&btnFocus===i){i.btn.focus=false;btnFocus=null;}},focus:function(i){if(btnFocus)btnFocus.btn.focus=false;btnFocus=i;i.btn.focus=true;btnUsedKeys=true;}},
  particles:{burst:function(i,p){emitParts(i,Math.min(500,Math.max(0,Math.floor(num(X(p[0]))))));},active:function(i,p){i.ps.active=!!p[0];},setrate:function(i,p){i.ps.rate=Math.max(0,num(X(p[0])));},clear:function(i){i.ps.parts=[];}},
  'b:car':{sim:function(i,p){i.b.car.sim[p[0]]=true;},setspeed:function(i,p){i.b.car.sp=num(X(p[0]));},setmax:function(i,p){i.b.car.p.maxSpeed=num(X(p[0]));},stop:function(i){var c=i.b.car;c.sp=0;c.vx=0;c.vy=0;},controls:function(i,p){i.b.car.ctl=!!p[0];},enable:function(i,p){i.b.car.en=!!p[0];}},
  'b:eightdir':{sim:function(i,p){i.b.eightdir.sim[p[0]]=true;},setmax:function(i,p){i.b.eightdir.p.maxSpeed=num(X(p[0]));},stop:function(i){i.b.eightdir.vx=0;i.b.eightdir.vy=0;},enable:function(i,p){i.b.eightdir.en=!!p[0];}},
  'b:tween':{
   start:function(i,p){var st=i.b.tween,pr=p[0],to=num(X(p[1])),d=Math.max(0,num(X(p[2])));st.list=st.list.filter(function(t){return t.prop!==pr;});
    var from=tprop(i,pr);if(d<=0){tset(i,pr,to);st.done[p[5]]=tick+1;return;}st.list.push({prop:pr,from:from,to:to,d:d,t:0,ease:p[3]||'inout',mode:p[4]||'once',tag:p[5]||'tween'});},
   stop:function(i,p){var st=i.b.tween;st.list=st.list.filter(function(t){return t.tag!==p[0];});},
   stopall:function(i){i.b.tween.list=[];}},
  'b:fade':{start:function(i){var f=i.b.fade;f.t=0;f.run=true;f.base=f.base||1;if(f.p.fadeIn>0)i.opacity=0;else i.opacity=f.base;},
   out:function(i,p){var f=i.b.fade,d=Math.max(.01,num(X(p[0])));f.base=i.opacity;f.p=Object.assign({},f.p,{fadeIn:0,wait:0,fadeOut:d});f.t=0;f.run=true;}},
  'b:pin':{
   pinto:function(i,p,sol){var t=types[p[0]];if(!t)return;var j=(sol&&sol[t.id]&&sol[t.id].filter(alive)[0])||t.insts.filter(alive)[0];if(!j||j===i)return;var st=i.b.pin,nm=p[1]==null?'':String(p[1]).trim();
    st.to=j;st.pt=nm;var ux=i.x-j.x,uy=i.y-j.y;if(st.p.angle&&j.angle){var r=-j.angle*Math.PI/180,c=Math.cos(r),s=Math.sin(r),x2=ux*c-uy*s;uy=ux*s+uy*c;ux=x2;}
    if(st.p.mirror&&j.mirrored)ux=-ux;st.ox=ux;st.oy=uy;st.oa=i.angle-(j.angle||0);st.mi=i.mirrored!==j.mirrored;},
   unpin:function(i){i.b.pin.to=null;}},
  'b:drag':{drop:function(i){var d=i.b.drag;if(d.on){d.on=false;d.dr=tick+1;}},enable:function(i,p){var d=i.b.drag;d.en=!!p[0];if(!d.en)d.on=false;}},
  'b:turret':{addtarget:function(i,p){var st=i.b.turret;if(types[p[0]]&&st.types.indexOf(p[0])<0)st.types.push(p[0]);},cleartargets:function(i){i.b.turret.types=[];i.b.turret.target=null;},setrange:function(i,p){i.b.turret.p.range=num(X(p[0]));}},
  'b:los':{setrange:function(i,p){i.b.los.p.range=num(X(p[0]));}},
  'b:timer':{start:function(i,p){var d=Math.max(0,num(X(p[1])));i.b.timer.timers[p[0]]={left:d,dur:d,reg:p[2]==='regular'};},stop:function(i,p){delete i.b.timer.timers[p[0]];},stopall:function(i){i.b.timer.timers={};}}
 };
 function copySol(sol){var o={};for(var k in sol)o[k]=sol[k].slice();return o;}
 function runActs(acts,sol,from,subs){
  for(var n=from||0;n<acts.length;n++){var a=acts[n];if(a.off)continue;var dot=a.k.lastIndexOf('.'),pr=a.k.slice(0,dot),key=a.k.slice(dot+1),p=a.p||[];curSol=sol;cur=null;
   if(a.o==='system'&&key==='wait'){waits.push({t:time+Math.max(0,num(X(p[0]))),acts:acts,from:n+1,subs:subs,sol:copySol(sol),scopes:scopes.slice()});return true;}
   if(a.o==='system'){if(SYSA[key])SYSA[key](p,sol);}
   else if(a.o==='audio'){if(key==='play')sfx(p[0],mst());else if(key==='playfile')playFile(p[0],num(X(p[1]))/100,!!p[2],p[3]||'');else if(key==='music')playMusic(p[0],num(X(p[1]))/100,num(X(p[2])));else if(key==='stopmusic')stopMusic(num(X(p[0])));
    else if(key==='stoptag')voices.slice().forEach(function(v){if(v.tag===p[0]){try{v.src.stop();}catch(e){}}});else if(key==='stopall')stopAllSnd();else if(key==='volume'){mvol=Math.max(0,num(X(p[0]))/100);var mm=mst();if(mm&&!muted)mm.gain.value=mvol;}}
   else if(a.o==='storage'){if(key==='savevar')sset(p[0],globals[p[0]]);else if(key==='loadvar'){var sv=sget(p[0]);if(sv!==undefined&&p[0] in globals)globals[p[0]]=typeof globals[p[0]]==='string'?String(sv):num(sv);}else if(key==='save')sset(p[0],X(p[1]));else if(key==='remove')sdel(p[0]);else if(key==='clear')sclear();}
   else if(a.o==='mouse'){if(key==='cursor')canvas.style.cursor=p[0]||'';}
   else if(types[a.o]&&(types[a.o].def.kind==='array'||types[a.o].def.kind==='dict')){var dt0=types[a.o],da=DATAA[dt0.def.kind][key];if(da)da(DATA[dt0.id],p);}
   else{var t=types[a.o];if(!t)continue;var fn=OBJA[pr]&&OBJA[pr][key];if(!fn)continue;var list=picked(sol,t);
    for(var m=0;m<list.length;m++){var i=list[m];if(pr.indexOf('b:')===0&&!i.b[pr.slice(2)])continue;cur=i;curSol=sol;fn(i,p,sol);}cur=null;}
   if(pending)return true;}
  return false;
 }
 function runWaits(){
  if(!waits.length)return;var due=[],keep=[];waits.forEach(function(w){(w.t<=time+1e-9?due:keep).push(w);});if(!due.length)return;waits=keep;
  for(var n=0;n<due.length&&!pending;n++){var w=due[n],sol={};for(var k in w.sol)sol[k]=w.sol[k].filter(alive);var saved=scopes;scopes=w.scopes||[];
   try{if(!runActs(w.acts,sol,w.from,w.subs)&&w.subs&&w.subs.length)runList(w.subs,sol);}finally{scopes=saved;}}
 }
 var LOOPS={repeat:1,forl:1,foreach:1,'while':1,foreachitem:1};
 var evHits={},bps={},brkHit=null,dbgSpeed=1,fpsN=0,fpsT=0,fpsV=0;
 function finishEvent(ev,sol){evHits[ev.id]=(evHits[ev.id]||0)+1;if(bps[ev.id]&&!paused){paused=true;brkHit={id:ev.id,tick:tick};}var waited=runActs(ev.acts||[],sol,0,ev.subs);if(!waited&&ev.subs&&ev.subs.length&&!pending)runList(ev.subs,sol);return true;}
 function runOr(ev,sol,prevOk){
  var any=false,union={},full={};
  for(var n=0;n<ev.conds.length;n++){var c=ev.conds[n];if(c.off)continue;var s2=Object.assign({},sol);
   if(evalCond(c,s2,prevOk)){any=true;for(var k in s2){if(s2[k]===sol[k])continue;if(!union[k])union[k]=[];s2[k].forEach(function(i){if(union[k].indexOf(i)<0)union[k].push(i);});}
    for(var t in types){if(s2[t]===sol[t])full[t]=1;}}}
  if(!any)return false;for(var k2 in union)if(!full[k2])sol[k2]=union[k2];
  return finishEvent(ev,sol);
 }
 function groupOn(g){return g.id in groupState?groupState[g.id]:g.active!==false;}
 function runFunc(name,args){var fn=funcs[name];if(!fn||callDepth>64)return 0;var sc={vars:{},fn:true,ret:0};(fn.params||[]).forEach(function(pn,i){sc.vars[pn]=args[i]!==undefined?args[i]:0;});
  var sCur=cur,sSol=curSol;scopes.push(sc);callDepth++;
  try{var sol={};if(!runActs(fn.acts||[],sol,0,fn.subs)&&fn.subs&&fn.subs.length)runList(fn.subs,sol);}finally{callDepth--;var k=scopes.lastIndexOf(sc);if(k>=0)scopes.splice(k,1);cur=sCur;curSol=sSol;}
  return sc.ret;}
 function runEvent(ev,sol,from,prevOk){
  if(ev.or&&!from)return runOr(ev,sol,prevOk);
  var cs=ev.conds;
  for(var n=from;n<cs.length;n++){var c=cs[n];
   if(!c.off&&c.o==='system'){var key=c.k.slice(c.k.lastIndexOf('.')+1);if(LOOPS[key])return runLoop(ev,sol,n,key,c,prevOk);}
   if(!evalCond(c,sol,prevOk))return false;}
  return finishEvent(ev,sol);
 }
 function runLoop(ev,sol,n,key,c,prevOk){
  var p=c.p||[],any=false,fr={name:'',i:0},k;loopStack.push(fr);curSol=sol;cur=null;
  try{
   if(key==='repeat'){var N=Math.min(100000,Math.floor(num(X(p[0]))));for(k=0;k<N&&!pending;k++){fr.i=k;if(runEvent(ev,Object.assign({},sol),n+1,prevOk))any=true;}}
   else if(key==='forl'){fr.name=String(p[0]);var a=Math.round(num(X(p[1]))),b=Math.round(num(X(p[2]))),st=a<=b?1:-1,guard=0;for(k=a;(st>0?k<=b:k>=b)&&!pending&&guard<100000;k+=st,guard++){fr.i=k;if(runEvent(ev,Object.assign({},sol),n+1,prevOk))any=true;}}
   else if(key==='foreach'){var t=types[p[0]];if(t){var list=picked(sol,t).slice();for(k=0;k<list.length&&!pending;k++){if(list[k].dead)continue;fr.i=k;var s2=Object.assign({},sol);s2[t.id]=[list[k]];if(runEvent(ev,s2,n+1,prevOk))any=true;}}}
   else if(key==='foreachitem'){var dtl=types[p[0]],dd2=dtl&&DATA[dtl.id];if(dd2){if(dd2.list){var L2=dd2.list.slice();for(k=0;k<L2.length&&!pending;k++){fr.i=k;fr.val=L2[k];fr.key=k;if(runEvent(ev,Object.assign({},sol),n+1,prevOk))any=true;}}else{var ks=Object.keys(dd2.map);for(k=0;k<ks.length&&!pending;k++){fr.i=k;fr.key=ks[k];fr.val=dd2.map[ks[k]];if(runEvent(ev,Object.assign({},sol),n+1,prevOk))any=true;}}}}
   else if(key==='while'){for(k=0;k<10000&&!pending;k++){fr.i=k;if(!runEvent(ev,Object.assign({},sol),n+1,prevOk))break;any=true;}}
  }finally{loopStack.pop();}
  return any;
 }
 function runList(list,parentSol){
  var sc=null,k,ev;
  for(k=0;k<list.length;k++){ev=list[k];if(ev.kind==='lvar'&&ev.name){if(!sc)sc={vars:{},st:[]};var key='lv:'+ev.id;
   if(ev['static']&&cstate[key]!==undefined)sc.vars[ev.name]=cstate[key];else sc.vars[ev.name]=ev.type==='text'?String(ev.value==null?'':ev.value):(+ev.value||0);if(ev['static'])sc.st.push(ev);}}
  if(sc)scopes.push(sc);
  try{
   var prevOk=true;
   for(k=0;k<list.length;k++){ev=list[k];if(ev.off)continue;
    if(ev.kind==='event')prevOk=runEvent(ev,Object.assign({},parentSol),0,prevOk);
    else if(ev.kind==='group'){if(groupOn(ev))runList(ev.subs||[],parentSol);}
    else if(ev.kind==='include'){var sh=(P.sheets||[]).find(function(x){return x.id===ev.sheet;});if(sh&&includeStack.indexOf(sh.id)<0&&(!sheet||sh.id!==sheet.id)){includeStack.push(sh.id);try{runList(sh.events||[],parentSol);}finally{includeStack.pop();}}}
    if(pending)return;
   }
  }finally{if(sc){sc.st.forEach(function(e){cstate['lv:'+e.id]=sc.vars[e.name];});var q=scopes.lastIndexOf(sc);if(q>=0)scopes.splice(q,1);}}
 }

 /* ---- step ---- */
 function step(dt){
  camPx=cam.x;camPy=cam.y;tick++;if(hsDone){hsDone=false;hsIn=null;hsNamedTick=tick;}if(hsIn&&!ctl.open)hsStep(dt);if(!ctl.open&&!gpaused&&!hsIn&&!(tr&&tr.phase==='out'))cineStep(dt);var frozen=!!dlg.cur||!!hsIn||!!(cine&&cine.def.freeze!==false);if(frozen&&!ctl.open)dlgStep(dt);frozen=frozen||!!dlg.cur||ctl.open;if(!frozen||gpaused)btnStep();frozen=frozen||gpaused;if(ctl.open){kp={};mouse.pr={};}if(!frozen)time+=dt;
  var n,i;
  solids=[];jts=[];ladders=[];polyCount=0;
  for(n=0;n<all.length;n++){i=all[n];i.px=i.x;i.py=i.y;if(i.dead)continue;if(i.tm){if(i.t.bh.solid){var TT=tmTiles(i);TT.rect.forEach(function(v){solids.push(v);});TT.poly.forEach(function(v){solids.push(v);polyCount++;});TT.jt.forEach(function(v){jts.push(v);});}continue;}
   if(i.t.bh.solid){solids.push(i);if(i.t.def.collision==='poly'&&i.t.def.kind==='sprite')polyCount++;}if(i.t.bh.jumpthru)jts.push(i);if(i.t.bh.ladder){jts.push(i);ladders.push(i);}if(i.b.physics&&!i.t.bh.solid)solids.push(i);}
  if(!frozen)for(n=0;n<all.length;n++){i=all[n];if(i.dead)continue;var b=i.b;
   if(b.sine&&b.sine.en){var s=b.sine;s.t+=dt;var off=Math.sin(s.t*Math.PI*2/Math.max(.05,s.p.period))*s.p.magnitude;if(s.p.axis==='v')i.y=s.by+off;else i.x=s.bx+off;}
   if(b.path)pathStep(i,dt);
   if(b.bullet){var u=b.bullet;u.vy+=u.p.gravity*dt;i.x+=u.vx*dt;i.y+=u.vy*dt;}
   if(b.rotate)i.angle+=b.rotate.p.speed*dt;}
  if(!frozen){if(pendingRopes.length){pendingRopes.forEach(function(r){if(r.i.dead)return;var c=physC(r.i);joints.push({a:r.i,px:c.x,py:c.y-r.len,len:r.len});});pendingRopes=[];}physStep(dt);}
  for(n=0;n<all.length;n++){i=all[n];i.dx=i.x-i.px;i.dy=i.y-i.py;}
  if(!frozen){for(n=0;n<all.length;n++){i=all[n];if(i.dead)continue;if(i.b.patrol)patrol(i,dt);if(i.b.platform)platform(i,dt);}
  if(mouse.pr.left&&mouse.psx!=null){var msx=mouse.sx,msy=mouse.sy;mouse.sx=mouse.psx;mouse.sy=mouse.psy;
   for(n=all.length-1;n>=0;n--){i=all[n];if(i.dead||!i.b.drag||!i.b.drag.en)continue;if(mouseOver(i)){var dg=i.b.drag,mw=mouseWorld(i);dg.on=true;dg.st=tick;dg.gx=mw[0]-i.x;dg.gy=mw[1]-i.y;break;}}
   mouse.sx=msx;mouse.sy=msy;}
  for(n=0;n<all.length;n++){i=all[n];if(!i.dead)behave(i,dt);}
  for(n=0;n<all.length;n++){i=all[n];if(!i.dead&&i.b.pin)pinStep(i);}}
  if(sflash&&sflash.t>0)sflash.t=Math.max(0,sflash.t-dt);
  for(n=0;n<all.length;n++){i=all[n];if(i.dead)continue;if(i.tf&&i.tf.t>0)i.tf.t=Math.max(0,i.tf.t-dt);if(frozen){animate(i,dt);continue;}
   if(i.b.bound){var bb=drawBox(i.t.def,i);if(bb.l<0)i.x-=bb.l;if(bb.r>layout.w)i.x-=bb.r-layout.w;if(bb.t<0)i.y-=bb.t;if(i.b.bound.p.bottom&&bb.b>layout.h)i.y-=bb.b-layout.h;}
   if(i.b.destroy){var db=drawBox(i.t.def,i);if(db.r<-64||db.l>layout.w+64||db.b<-400||db.t>layout.h+64)i.dead=true;}
   if(i.b.timer){var tm=i.b.timer.timers;for(var nm in tm){var q=tm[nm];q.left-=dt;if(q.left<=1e-9){i.b.timer.fired[nm]=tick;if(q.reg&&q.dur>0)q.left+=q.dur;else delete tm[nm];}}}
   animate(i,dt);if(i.bar)barStep(i,dt);if(i.flash>0)i.flash=Math.max(0,i.flash-dt);}
  runWaits();
  if(sheet&&!pending)runList(sheet.events||[],{});
  for(n=0;n<all.length;n++)if(all[n]._kill)all[n].dead=true;
  firstTick=false;kp={};pkp={};kr={};anyPress=false;mouse.pr={};mouse.rl={};
  if(all.some(function(i){return i.dead;})){all=all.filter(alive);for(var k in types)types[k].insts=types[k].insts.filter(alive);}
  if(shakeT>0)shakeT=Math.max(0,shakeT-dt);
  updateCam(dt);
  if(pending){var pl=pending;pending=null;if(trans.type!=='none'&&trans.dur>0&&!tr)tr={phase:'out',t:0,target:pl,d:trans.dur/2,type:trans.type,color:trans.color};else loadLayout(pl);}
 }
 function updateCam(dt){
  if(zoomA&&dt){zoomA.t+=dt;var zk=Math.min(1,zoomA.t/zoomA.d),ze=zk<.5?2*zk*zk:1-Math.pow(-2*zk+2,2)/2;var ocx=cam.x+W/zoom/2,ocy=cam.y+H/zoom/2;zoom=zoomA.from+(zoomA.to-zoomA.from)*ze;cam.x=ocx-W/zoom/2;cam.y=ocy-H/zoom/2;if(zk>=1)zoomA=null;}
  var vw=W/zoom,vh=H/zoom,tg=all.filter(function(i){return !i.dead&&i.b.scrollto;}),tx=null,ty=null,P0=tg.length?tg[0].b.scrollto.p:{};
  if(camLook){tx=camLook.x;ty=camLook.y;}
  else if(tg.length){var x=0,y=0;tg.forEach(function(i){var b=box(i);x+=(b.l+b.r)/2;y+=(b.t+b.b)/2;});tx=x/tg.length;ty=y/tg.length+(P0.offsetY||0);
   if(P0.lookahead){var mv=tg[0].b.platform?tg[0].b.platform.vx:tg[0].b.eightdir?tg[0].b.eightdir.vx:(tg[0].x-(tg[0].px==null?tg[0].x:tg[0].px))*60,dir=Math.abs(mv)>20?(mv>0?1:-1):0;camLA+=(dir*P0.lookahead-camLA)*(dt?Math.min(1,dt*2.5):1);tx+=camLA;}}
  if(tx!=null){var dx=tx-(cam.x+vw/2),dy=ty-(cam.y+vh/2),dzw=(P0.deadW||0)/2,dzh=(P0.deadH||0)/2;
   var mx=Math.abs(dx)>dzw?dx-(dx>0?dzw:-dzw):0,my=Math.abs(dy)>dzh?dy-(dy>0?dzh:-dzh):0,sm=camLook?Math.max(P0.smooth||0,60):(P0.smooth||0);
   var f=!dt||sm<=0?1:1-Math.exp(-dt*(30*Math.pow(1-sm/100,2)+1));cam.x+=mx*f;cam.y+=my*f;}
  var bx=camB?camB.x:0,by=camB?camB.y:0,bw=camB?camB.w:layout.w,bh=camB?camB.h:layout.h;
  cam.x=bw<=vw?bx+(bw-vw)/2:Math.max(bx,Math.min(cam.x,bx+bw-vw));cam.y=bh<=vh?by+(bh-vh)/2:Math.max(by,Math.min(cam.y,by+bh-vh));
 }

 /* ---- render ---- */
 function resize(){var r=canvas.getBoundingClientRect(),d=Math.min(window.devicePixelRatio||1,3),w=Math.max(1,Math.round(r.width*d)),h=Math.max(1,Math.round(r.height*d));if(w!==cw||h!==ch){cw=w;ch=h;canvas.width=w;canvas.height=h;}}
 function render(){
  resize();var c=ctx;c.setTransform(1,0,0,1,0,0);c.fillStyle='#000';c.fillRect(0,0,cw,ch);
  var s=Math.min(cw/W,ch/H),ox=Math.round((cw-W*s)/2),oy=Math.round((ch-H*s)/2);
  if(!ready){drawLoader(c,s,ox,oy);return;}
  c.save();c.beginPath();c.rect(ox,oy,W*s,H*s);c.clip();c.fillStyle=layout.bg||'#000';c.fillRect(ox,oy,W*s,H*s);
  c.imageSmoothingEnabled=!S.pixelArt;
  var sx=0,sy=0;if(shakeT>0){var m=shakeA*(shakeT/shakeD);sx=(Math.random()*2-1)*m;sy=(Math.random()*2-1)*m;}
  var al=ialpha,icx=Math.abs(cam.x-camPx)<200&&Math.abs(cam.y-camPy)<200?camPx+(cam.x-camPx)*al:cam.x,icy=Math.abs(cam.x-camPx)<200&&Math.abs(cam.y-camPy)<200?camPy+(cam.y-camPy)*al:cam.y;
  var cx=icx+sx,cy=icy+sy;if(S.pixelArt){cx=Math.round(cx*s)/s;cy=Math.round(cy*s)/s;}
  var n,i,li;
  var darkDone=false;
  for(li=0;li<layers.length;li++){var ly=layers[li];if(!darkDone&&dark.a>0&&ly.px===0&&ly.py===0){darkDone=true;drawDark(c,s,ox,oy,cx,cy);}if(!ly.visible||ly.opacity<=0)continue;
   var lz=zf(ly),ls=s*lz,lx=cx*ly.px,lyy=cy*ly.py;if(S.pixelArt){lx=Math.round(lx*ls)/ls;lyy=Math.round(lyy*ls)/ls;}
   c.globalAlpha=ly.opacity;
   for(n=0;n<all.length;n++){i=all[n];if(i.dead||i.layer!==li||!i.visible)continue;if(i.flash>0&&Math.floor(i.flash*12)%2)continue;
    var hx=i.hud?0:lx,hy=i.hud?0:lyy,is=i.hud?s:ls;c.setTransform(is,0,0,is,ox-hx*is,oy-hy*is);
    var rx=i.x,ry=i.y;if(al<1&&i.px!=null&&Math.abs(rx-i.px)<120&&Math.abs(ry-i.py)<120){i.x=i.px+(rx-i.px)*al;i.y=i.py+(ry-i.py)*al;}
    var b=drawBox(i.t.def,i),vz=i.hud?1:lz;if(!(!i.angle&&!i.ps&&(b.r<hx-8||b.l>hx+W/vz+8||b.b<hy-8||b.t>hy+H/vz+8)))drawInst(c,i.t.def,i,img);i.x=rx;i.y=ry;}
   if(joints.length){c.setTransform(ls,0,0,ls,ox-lx*ls,oy-lyy*ls);drawJoints(c,li);}
  }
  c.globalAlpha=1;if(!darkDone&&dark.a>0)drawDark(c,s,ox,oy,cx,cy);c.setTransform(s,0,0,s,ox,oy);
  if(sflash&&sflash.t>0){c.globalAlpha=sflash.t/sflash.d*.85;c.fillStyle=sflash.c;c.fillRect(0,0,W,H);c.globalAlpha=1;}
  if(showCol){for(n=0;n<all.length;n++){i=all[n];if(i.dead||!i.visible||i.t.def.kind==='text')continue;var cl=layers[i.layer]||{px:1,py:1},cs2=s*zf(cl,i);c.setTransform(cs2,0,0,cs2,ox-(i.hud?0:cx*cl.px)*cs2,oy-(i.hud?0:cy*cl.py)*cs2);
    var sh=shape(i),bh=i.t.bh,col=bh.solid?'#3ddc84':bh.jumpthru?'#4da3ff':bh.platform?'#ff8a3d':bh.patrol?'#ff4d6a':'rgba(255,255,255,.75)';
    c.lineWidth=1.5/s;c.strokeStyle=col;c.fillStyle=col;c.globalAlpha=.18;c.beginPath();var pp=sh.p||boxPoly(sh.b);pp.forEach(function(q,k){if(k)c.lineTo(q[0],q[1]);else c.moveTo(q[0],q[1]);});c.closePath();c.fill();c.globalAlpha=1;c.stroke();
    if(i.b.platform&&i.b.platform.floor){c.fillStyle='#ffd84d';c.fillRect(sh.b.l,sh.b.b-1.5/s,sh.b.r-sh.b.l,3/s);}}
   c.setTransform(s,0,0,s,ox,oy);}
  if(hl){var hi=all.find(function(i){return i.uid===hl&&!i.dead;});if(hi){var hly=layers[hi.layer]||{px:1,py:1},hb=drawBox(hi.t.def,hi),hx2=hi.hud?0:cx*hly.px,hy2=hi.hud?0:cy*hly.py;c.setTransform(s,0,0,s,ox-hx2*s,oy-hy2*s);c.strokeStyle='#ff4dd2';c.lineWidth=2/s;c.setLineDash([4/s,3/s]);c.strokeRect(hb.l,hb.t,hb.r-hb.l,hb.b-hb.t);c.setLineDash([]);c.setTransform(s,0,0,s,ox,oy);}}
  c.setTransform(s,0,0,s,ox,oy);drawCine(c);if(dlg.cur)drawDialog(c);if(hsIn)drawHs(c);if(ctl.open)drawCtl(c);
  if(tr){var pr=tr.phase==='out'?Math.min(1,tr.t/tr.d):Math.max(0,1-tr.t/tr.d);c.fillStyle=tr.color;
   if(tr.type==='iris'){var R=Math.hypot(W,H)/2*(1-pr);c.beginPath();c.rect(0,0,W,H);c.arc(W/2,H/2,Math.max(0,R),0,Math.PI*2,true);c.fill('evenodd');}
   else if(tr.type==='wipe'){var wx=tr.phase==='out'?W*pr:W*(1-pr);if(tr.phase==='out')c.fillRect(0,0,wx,H);else c.fillRect(W-W*pr,0,W*pr,H);}
   else{c.globalAlpha=pr;c.fillRect(0,0,W,H);c.globalAlpha=1;}}
  if(muted){c.globalAlpha=.6;c.fillStyle='#fff';c.font='600 11px '+FONTS.sans;c.textAlign='right';c.textBaseline='bottom';c.fillText('son coupé (M)',W-8,H-6);c.globalAlpha=1;}
  c.restore();
 }

 function drawDark(c,s,ox,oy,cx,cy){if(!DCV)DCV=document.createElement('canvas');var w=Math.ceil(W*s),h=Math.ceil(H*s);if(DCV.width!==w||DCV.height!==h){DCV.width=w;DCV.height=h;}
  var d=DCV.getContext('2d');d.globalCompositeOperation='source-over';d.clearRect(0,0,w,h);d.globalAlpha=dark.a;d.fillStyle=dark.c;d.fillRect(0,0,w,h);d.globalAlpha=1;d.globalCompositeOperation='destination-out';var glows=[];
  for(var n=0;n<all.length;n++){var i=all[n];if(i.dead||!i.b.light||!i.visible)continue;var L=i.b.light.p,ly=layers[i.layer]||{px:1,py:1},bx=drawBox(i.t.def,i),wx=(bx.l+bx.r)/2,wy=(bx.t+bx.b)/2;
   var lzz=zf(ly,i),sx2=(wx-(i.hud?0:cx*ly.px))*s*lzz,sy2=(wy-(i.hud?0:cy*ly.py))*s*lzz,r=L.radius*s*lzz*(L.flicker?1+Math.sin(time*13+i.uid)*.04+Math.sin(time*7.3+i.uid*3)*.03:1);if(r<=0)continue;
   var g=d.createRadialGradient(sx2,sy2,0,sx2,sy2,r),k=(L.intensity==null?100:L.intensity)/100;g.addColorStop(0,'rgba(0,0,0,'+k+')');g.addColorStop(.55,'rgba(0,0,0,'+(k*.7)+')');g.addColorStop(1,'rgba(0,0,0,0)');d.fillStyle=g;d.fillRect(sx2-r,sy2-r,r*2,r*2);
   if(L.glow>0)glows.push([sx2,sy2,r,L.color||'#ffd9a0',L.glow/100*k]);}
  d.globalCompositeOperation='source-over';c.save();c.setTransform(1,0,0,1,0,0);c.globalAlpha=1;c.drawImage(DCV,ox,oy);
  if(glows.length){c.globalCompositeOperation='lighter';glows.forEach(function(q){var g2=c.createRadialGradient(ox+q[0],oy+q[1],0,ox+q[0],oy+q[1],q[2]*.8);var rgb=parseInt(q[3].slice(1),16);var col=(rgb>>16&255)+','+(rgb>>8&255)+','+(rgb&255);g2.addColorStop(0,'rgba('+col+','+(q[4]*.6)+')');g2.addColorStop(1,'rgba('+col+',0)');c.fillStyle=g2;c.fillRect(ox+q[0]-q[2],oy+q[1]-q[2],q[2]*2,q[2]*2);});}
  c.restore();}
 function wrapLines(c,text,maxW){var out=[];String(text).split('\n').forEach(function(par){var words=par.split(' '),line='';words.forEach(function(w){var t=line?line+' '+w:w;if(c.measureText(t).width>maxW&&line){out.push(line);line=w;}else line=t;});out.push(line);});return out;}
 function drawDialog(c){var d=dlg.cur,fs=Math.max(14,Math.round(H/27)),lh=Math.round(fs*1.35),pad=Math.round(fs*.9),m=12,font=(FONTS[DS.font]||FONTS.sans);
  var por=null;if(d.portrait&&types[d.portrait]&&types[d.portrait].def.kind==='sprite'){var pa=types[d.portrait].def.anims[0];por=pa&&img(pa.frames[0]);}
  c.font='600 '+fs+'px '+font;var bw=W-m*2,ps=por?lh*3+pad:0,tx=m+pad+(por?ps+pad*.6:0),maxW=bw-(tx-m)-pad,full=d.text,lines=wrapLines(c,full,maxW);
  var nch=d.choices&&d.shown>=full.length?d.choices.length:0,bh=Math.max(lines.length+(nch?nch+.3:0),3)*lh+pad*2,by=d.top?Math.round(H*.13)+(d.name?fs*.4:0):H-m-bh;
  c.save();c.textAlign='left';c.globalAlpha=.94;c.fillStyle=DS.bg;rr(c,m,by,bw,bh,10);c.fill();c.globalAlpha=1;c.lineWidth=2;c.strokeStyle=DS.accent;c.globalAlpha=.55;rr(c,m+1,by+1,bw-2,bh-2,9);c.stroke();c.globalAlpha=1;
  if(por&&por.complete&&por.naturalWidth){c.imageSmoothingEnabled=!S.pixelArt;var pk=Math.min(ps/por.naturalWidth,ps/por.naturalHeight);c.fillStyle='rgba(255,255,255,.06)';rr(c,m+pad,by+pad,ps,ps,8);c.fill();c.drawImage(por,m+pad+(ps-por.naturalWidth*pk)/2,by+pad+(ps-por.naturalHeight*pk)/2,por.naturalWidth*pk,por.naturalHeight*pk);}
  if(d.name){c.font='800 '+Math.round(fs*.85)+'px '+font;var nw=c.measureText(d.name).width+pad*1.4,nh=Math.round(fs*1.5),nx=m+pad*.6,ny=by-nh*.6;c.fillStyle=DS.accent;rr(c,nx,ny,nw,nh,7);c.fill();c.fillStyle=DS.bg;c.textBaseline='middle';c.fillText(d.name,nx+pad*.7,ny+nh/2+1);}
  c.font='600 '+fs+'px '+font;c.fillStyle=DS.fg;c.textBaseline='top';var left=d.shown,ty=by+pad;
  for(var k=0;k<lines.length&&left>0;k++){var L=lines[k],part=L.slice(0,left);c.fillText(part,tx,ty+k*lh);left-=L.length+1;}
  d.hit=null;if(nch){d.hit=[];var cy0=by+pad+Math.max(lines.length,1)*lh+pad*.4;d.choices.forEach(function(ch,q){var yy=cy0+q*lh,on=q===d.sel;if(on){c.fillStyle=DS.accent;c.globalAlpha=.18;rr(c,tx-8,yy-3,maxW+8,lh,6);c.fill();c.globalAlpha=1;}c.fillStyle=on?DS.accent:DS.fg;c.fillText((on?'▶ ':'   ')+ch,tx,yy);d.hit.push([tx-8,yy-3,tx+maxW,yy+lh-3]);});}
  else if(d.shown>=full.length&&Math.floor(time*0+d.t*2.5)%2===0){c.fillStyle=DS.accent;var ax=m+bw-pad-8,ay=by+bh-pad-6;c.beginPath();c.moveTo(ax,ay);c.lineTo(ax+12,ay);c.lineTo(ax+6,ay+7);c.closePath();c.fill();}
  c.restore();}
 /* ---- input ---- */
 function kd(e){var tg=e.target&&e.target.tagName;if(tg==='INPUT'||tg==='TEXTAREA')return;if(GAMEKEYS[e.code]||ctl.open)e.preventDefault();audio();if(e.repeat)return;if(splash){splash=false;boot();return;}if(hsIn&&!ctl.open){hsKey(e);return;}if(ctl.open){ctlKey(e);return;}pkp[e.code]=true;if(CTL.length)ctlPress(e.code,true);else{keys[e.code]=true;kp[e.code]=true;anyPress=true;}if(e.code==='KeyM'&&!opts.embedded){muted=!muted;if(master)master.gain.value=muted?0:mvol;}if(e.code==='KeyF'&&!opts.embedded&&opts.fullscreenKey!==false){try{if(document.fullscreenElement)document.exitFullscreen();else document.documentElement.requestFullscreen();}catch(err){}}}
 function ku(e){if(ctl.open)return;if(CTL.length){ctlPress(e.code,false);return;}keys[e.code]=false;kr[e.code]=true;}
 function bl(){keys={};physDown={};}
 function toView(e){var r=canvas.getBoundingClientRect(),s=Math.min(r.width/W,r.height/H)||1,ox=(r.width-W*s)/2,oy=(r.height-H*s)/2;mouse.sx=(e.clientX-r.left-ox)/s;mouse.sy=(e.clientY-r.top-oy)/s;}
 function pd(e){audio();toView(e);if(splash){splash=false;boot();return;}if(ctl.open){ctlClick();return;}if(hsIn){hsClick();return;}var b=BTN[e.button]||'left';mouse.down[b]=true;mouse.pr[b]=true;mouse.psx=mouse.sx;mouse.psy=mouse.sy;}
 function pmv(e){toView(e);}
 function pup(e){if(e.target===canvas)toView(e);var b=BTN[e.button]||'left';if(mouse.down[b]){mouse.down[b]=false;mouse.rl[b]=true;}}
 function cmenu(e){e.preventDefault();}
 window.addEventListener('keydown',kd);window.addEventListener('keyup',ku);window.addEventListener('blur',bl);canvas.addEventListener('pointerdown',pd);canvas.addEventListener('pointermove',pmv);window.addEventListener('pointerup',pup);window.addEventListener('pointercancel',pup);canvas.addEventListener('contextmenu',cmenu);
 var padPrev={};
 function pollPad(){var gps=navigator.getGamepads?navigator.getGamepads():[],gp=null;for(var n=0;n<gps.length;n++)if(gps[n]){gp=gps[n];break;}if(!gp)return;
  var ax=gp.axes[0]||0,bt=function(k){return !!(gp.buttons[k]&&gp.buttons[k].pressed);},st={ArrowLeft:ax<-.4||bt(14),ArrowRight:ax>.4||bt(15),ArrowUp:bt(0)||bt(12),ArrowDown:bt(13)||ax>.6&&false,Space:bt(1),Enter:bt(9)};
  for(var k in st){if(st[k]&&!padPrev[k]){keys[k]=true;kp[k]=true;anyPress=true;}else if(!st[k]&&padPrev[k]){keys[k]=false;kr[k]=true;}}padPrev=st;
  var ay=gp.axes[1]||0,nv={ArrowUp:bt(12)||ay<-.6,ArrowDown:bt(13)||ay>.6,ArrowLeft:bt(14)||ax<-.6,ArrowRight:bt(15)||ax>.6,Enter:bt(0)||bt(9),Escape:bt(1)};for(var k2 in nv){if(nv[k2]&&!padNav[k2])pkp[k2]=true;}padNav=nv;}
 var touchEl=null;
 if(opts.touch&&(('ontouchstart' in window)||navigator.maxTouchPoints>0)){
  touchEl=document.createElement('div');touchEl.style.cssText='position:fixed;inset:0;pointer-events:none;z-index:10;user-select:none;-webkit-user-select:none';
  var mk=function(code,txt,css){var b=document.createElement('div');b.textContent=txt;b.style.cssText='position:absolute;bottom:26px;width:76px;height:76px;border-radius:50%;background:rgba(255,255,255,.14);border:2px solid rgba(255,255,255,.35);color:#fff;font:700 26px sans-serif;display:flex;align-items:center;justify-content:center;pointer-events:auto;touch-action:none;'+css;
   var on=function(e){e.preventDefault();audio();keys[code]=true;kp[code]=true;pkp[code]=true;anyPress=true;b.style.background='rgba(255,255,255,.32)';},off=function(){if(keys[code]){keys[code]=false;kr[code]=true;}b.style.background='rgba(255,255,255,.14)';};
   b.addEventListener('pointerdown',on);b.addEventListener('pointerup',off);b.addEventListener('pointercancel',off);b.addEventListener('pointerleave',off);touchEl.appendChild(b);};
  mk('ArrowLeft','◀','left:20px');mk('ArrowRight','▶','left:112px');mk('ArrowUp','▲','right:24px;width:90px;height:90px');
  document.body.appendChild(touchEl);
 }
 var paused=false,stepReq=0,hl=null,showCol=false;
 var trans=Object.assign({type:'fade',dur:.5,color:'#000000'},S.transition||{}),tr=null,dark={a:0,c:'#0b0b1e'},tscale=1,sflash=null,DCV=null;
 var DS=Object.assign({bg:'#1b1f2a',fg:'#ffffff',accent:'#ffd84d',font:'round',blip:true},S.dialog||{});
 var hsIn=null,hsDone=false,hsNamedTick=-1,hsName=String(sget('hs:name')||''),hsRank=0,hsLast=null,hsInp=null;
 function barSet(i,v){var B=i.bar;v=Math.max(0,Math.min(B.max,v));if(v<B.v){B.gt=.45;if(B.ghost<B.v)B.ghost=B.v;}B.v=v;}
 function barStep(i,dt){var B=i.bar,sm=i.t.def.smooth!==false;B.shown=sm?B.shown+(B.v-B.shown)*Math.min(1,dt*12):B.v;if(B.gt>0)B.gt-=dt;else B.ghost+=(B.shown-B.ghost)*Math.min(1,dt*6);if(B.ghost<B.shown)B.ghost=B.shown;}
 function hsList(t){var v=sget('hs:'+String(t==null||t===''?'scores':t));return Array.isArray(v)?v:[];}
 function hsFits(L,s,mx,asc){if(L.length<mx)return true;var w=L[mx-1];return asc?s<w.s:s>w.s;}
 function hsRefresh(){all.forEach(function(i){if(i.t.def.kind==='leaderboard'){i.lb=hsList(i.lbt);i.lbHi=hsLast&&hsLast.t===i.lbt?hsLast.d:null;}});}
 var CHARS=' ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_.!';
 function hsConfirm(){if(!hsIn)return;var v=hsIn.name.trim()||hsIn.def.trim()||'Joueur';hsName=v;sset('hs:name',v);hsDone=true;if(hsInp){hsInp.blur();}}
 function hsCycle(d){var q=hsIn,ch=q.name.charAt(q.cur)||' ',k=CHARS.indexOf(ch.toUpperCase());k=(k<0?0:k+d+CHARS.length)%CHARS.length;var nc=CHARS.charAt(k);if(q.cur>=q.name.length){if(q.name.length>=q.max)return;q.name+=nc;}else q.name=q.name.slice(0,q.cur)+nc+q.name.slice(q.cur+1);}
 function hsKey(e){var q=hsIn,k=e.key;e.preventDefault();if(hsDone)return;if(k==='Enter'){hsConfirm();return;}if(k==='Escape'){if(!q.name.trim())q.name=q.def;hsConfirm();return;}
  if(k==='Backspace'){if(q.cur>0){q.name=q.name.slice(0,q.cur-1)+q.name.slice(q.cur);q.cur--;}}else if(k==='Delete'){q.name=q.name.slice(0,q.cur)+q.name.slice(q.cur+1);}else if(k==='ArrowLeft')q.cur=Math.max(0,q.cur-1);else if(k==='ArrowRight')q.cur=Math.min(q.name.length,q.max-1,q.cur+1);else if(k==='ArrowUp')hsCycle(1);else if(k==='ArrowDown')hsCycle(-1);else if(k==='Home')q.cur=0;else if(k==='End')q.cur=q.name.length;
  else if(k.length===1&&!e.ctrlKey&&!e.metaKey&&q.name.length<q.max){q.name=q.name.slice(0,q.cur)+k+q.name.slice(q.cur);q.cur++;}q.t=0;}
 function hsStep(dt){var q=hsIn;q.t+=dt;var nk=navKeys();if(nk.Enter||nk.Space&&false)hsConfirm();else{if(nk.ArrowUp||nk.KeyW)hsCycle(1);if(nk.ArrowDown||nk.KeyS)hsCycle(-1);if(nk.ArrowLeft||nk.KeyA)q.cur=Math.max(0,q.cur-1);if(nk.ArrowRight||nk.KeyD)q.cur=Math.min(q.name.length,q.max-1,q.cur+1);}kp={};pkp={};anyPress=false;mouse.pr={};}
 function hsClick(){var q=hsIn;if(!q)return;var h=q.hit;if(h&&mouse.sx>=h[0]&&mouse.sx<=h[2]&&mouse.sy>=h[1]&&mouse.sy<=h[3]){hsConfirm();return;}if(('ontouchstart' in window)||navigator.maxTouchPoints>0){if(!hsInp){hsInp=document.createElement('input');hsInp.style.cssText='position:fixed;left:-200px;top:0;opacity:0;width:10px';document.body.appendChild(hsInp);hsInp.addEventListener('input',function(){if(!hsIn)return;hsIn.name=hsInp.value.slice(0,hsIn.max);hsIn.cur=hsIn.name.length;});hsInp.addEventListener('keydown',function(ev){if(ev.key==='Enter'){ev.preventDefault();hsConfirm();}ev.stopPropagation();});}hsInp.value=q.name;hsInp.maxLength=q.max;hsInp.focus();}}
 var cine=null,cineEndTick=-1,cineEndId='',cineBars=0,cineTitle=null,cineFade={a:0,c:'#000000'};
 function cineLock(){return !!(cine&&cine.def.lock!==false);}
 function cineEase(e,k){return k==='linear'?e:k==='in'?e*e:k==='out'?1-(1-e)*(1-e):e<.5?2*e*e:1-Math.pow(-2*e+2,2)/2;}
 function cineInst(id){var t=types[id];return t?t.insts.filter(alive)[0]||null:null;}
 function cineCenter(o){var b=box(o);return{x:(b.l+b.r)/2,y:(b.t+b.b)/2};}
 function camCenter(){if(camLook)return{x:camLook.x,y:camLook.y};return{x:cam.x+W/zoom/2,y:cam.y+H/zoom/2};}
 function heroCenter(){var tg=all.filter(function(i){return !i.dead&&i.b.scrollto;});if(!tg.length)return camCenter();var x=0,y=0;tg.forEach(function(i){var c=cineCenter(i);x+=c.x;y+=c.y;});return{x:x/tg.length,y:y/tg.length+((tg[0].b.scrollto.p||{}).offsetY||0)};}
 function cineZoom(z,d){z=+z;if(!(z>0))return;z=Math.max(.1,Math.min(10,z/100));if(d<=0){zoom=z;zoomA=null;}else zoomA={from:zoom,to:z,t:0,d:d};}
 function cinePlay(id){var d=(P.cines||[]).find(function(c){return c.id===id||c.name===id;});if(!d)return;if(cine)cineSkip();cine={def:d,k:0,run:[],follow:null};cineNext();}
 function cineNext(){if(!cine)return;var S=cine.def.steps||[];if(cine.k>=S.length){cineEnd();return;}var g=[S[cine.k]];cine.k++;while(cine.k<S.length&&S[cine.k].w){g.push(S[cine.k]);cine.k++;}cine.run=g.map(cineStart);if(cine.run.every(function(r){return r.done;}))cineNext();}
 function cineStart(st){var r={s:st,t:0,d:Math.max(0,+st.d||0),done:false},o;
  switch(st.t){
   case 'say':dlgPush({name:String(st.who||''),text:String(st.text||''),portrait:st.o||'',top:st.pos==='top'});r.say=true;break;
   case 'camto':r.f=camCenter();r.to={x:num(X(st.x)),y:num(X(st.y))};cine.follow=null;if(st.zoom)cineZoom(st.zoom,r.d);break;
   case 'camobj':r.f=camCenter();r.o=cineInst(st.o);cine.follow=r.o;if(st.zoom)cineZoom(st.zoom,r.d);break;
   case 'camback':r.f=camCenter();cine.follow=null;if(zoom!==1)cineZoom(100,r.d);break;
   case 'move':o=cineInst(st.o);if(!o){r.done=true;break;}r.o=o;r.f={x:o.x,y:o.y};var tx=num(X(st.x)),ty=num(X(st.y));r.to=st.rel?{x:o.x+tx,y:o.y+ty}:{x:tx,y:ty};if(o.t.def.kind==='sprite'&&st.face!==false&&Math.abs(r.to.x-o.x)>.5)o.mirrored=r.to.x<o.x;if(o.b.platform){o.b.platform.vx=0;o.b.platform.vy=0;}break;
   case 'anim':o=cineInst(st.o);if(o&&o.t.def.kind==='sprite'){var ai=o.t.def.anims.findIndex(function(a){return a.name===st.a;});if(ai>=0&&ai!==o.anim){o.anim=ai;o.frame=0;o.ft=0;}}r.done=true;break;
   case 'title':cineTitle={text:String(st.text||''),t:0,d:Math.max(.5,r.d)};break;
   case 'fade':r.from=cineFade.a;r.to=st.mode==='in'?0:1;cineFade.c=st.color||'#000000';if(r.d<=0){cineFade.a=r.to;r.done=true;}break;
   case 'sound':if(st.s){if(String(st.s).indexOf('file:')===0)playFile(String(st.s).slice(5),1,false,'');else sfx(st.s,mst());}r.done=true;break;
   case 'vis':(types[st.o]?types[st.o].insts:[]).forEach(function(i){i.visible=st.v!==false;});r.done=true;break;
   case 'call':if(st.f)runFunc(st.f,[]);r.done=true;break;
   case 'shake':shakeA=+st.a||6;shakeD=shakeT=Math.max(.01,r.d);break;
  }
  if(!r.say&&r.d<=0&&['wait','camto','camobj','camback','move','title','shake'].indexOf(st.t)>=0){cineApply(r,1);r.done=true;}
  return r;}
 function cineApply(r,e){var st=r.s,k=cineEase(e,st.ease);
  if(st.t==='camto')camLook={x:r.f.x+(r.to.x-r.f.x)*k,y:r.f.y+(r.to.y-r.f.y)*k};
  else if(st.t==='camobj'){if(r.o&&!r.o.dead){var c=cineCenter(r.o);camLook={x:r.f.x+(c.x-r.f.x)*k,y:r.f.y+(c.y-r.f.y)*k};}}
  else if(st.t==='camback'){var h=heroCenter();camLook={x:r.f.x+(h.x-r.f.x)*k,y:r.f.y+(h.y-r.f.y)*k};if(e>=1)camLook=null;}
  else if(st.t==='move'){if(r.o&&!r.o.dead){r.o.x=r.f.x+(r.to.x-r.f.x)*k;r.o.y=r.f.y+(r.to.y-r.f.y)*k;}}
  else if(st.t==='fade')cineFade.a=r.from+(r.to-r.from)*k;}
 function cineStep(dt){if(!cine){if(cineBars>0)cineBars=Math.max(0,cineBars-dt*3);if(cineTitle){cineTitle.t+=dt;if(cineTitle.t>=cineTitle.d)cineTitle=null;}return;}
  if(cine.def.bars!==false)cineBars=Math.min(1,cineBars+dt*3);if(cineTitle){cineTitle.t+=dt;if(cineTitle.t>=cineTitle.d)cineTitle=null;}
  if(cine.def.skip!==false&&pkp.Escape){pkp={};kp={};cineSkip();return;}
  var busyCam=false;cine.run.forEach(function(r){if(r.done)return;r.t+=dt;if(r.say){if(r.t>.05&&!dlg.cur&&!dlg.queue.length)r.done=true;return;}var e=r.d>0?Math.min(1,r.t/r.d):1;cineApply(r,e);if(['camto','camobj','camback'].indexOf(r.s.t)>=0)busyCam=true;if(e>=1)r.done=true;});
  if(!busyCam&&cine&&cine.follow){if(cine.follow.dead)cine.follow=null;else camLook=cineCenter(cine.follow);}
  if(cine&&cine.run.every(function(r){return r.done;}))cineNext();}
 function cineSkip(){if(!cine)return;var d=cine;cine.run.forEach(function(r){if(!r.done&&!r.say)cineApply(r,1);});var S=d.def.steps||[];for(var k=d.k;k<S.length;k++){var st=S[k];if(['move','anim','vis','fade','call'].indexOf(st.t)<0)continue;var r=cineStart(st);if(!r.done)cineApply(r,1);}
  dlg.queue=[];if(dlg.cur){dlg.cur=null;dlg.endTick=tick;}cineTitle=null;cineEnd();}
 function cineEnd(){if(!cine)return;cineEndId=cine.def.id;cineEndTick=tick+1;cine=null;camLook=null;if(zoom!==1)cineZoom(100,.4);}
 function drawCine(c){if(cineFade.a>0){c.globalAlpha=Math.min(1,cineFade.a);c.fillStyle=cineFade.c;c.fillRect(0,0,W,H);c.globalAlpha=1;}
  if(cineBars>0){var bh=Math.round(H*.11*(cineBars<1?1-Math.pow(1-cineBars,3):1));c.fillStyle='#000';c.fillRect(0,0,W,bh);c.fillRect(0,H-bh,W,bh);}
  if(cineTitle){var tt=cineTitle,a=Math.max(0,Math.min(1,tt.t/.4,(tt.d-tt.t)/.4)),fs=Math.round(H/14);c.save();c.globalAlpha=a;c.font='800 '+fs+'px '+(FONTS[DS.font]||FONTS.round);c.textAlign='center';c.textBaseline='middle';c.lineJoin='round';c.lineWidth=Math.max(3,fs*.14);c.strokeStyle='rgba(0,0,0,.75)';c.strokeText(tt.text,W/2,H*.72);c.fillStyle='#ffffff';c.fillText(tt.text,W/2,H*.72);c.restore();}
  if(cine&&cine.def.skip!==false){c.save();c.globalAlpha=.6;c.font='600 '+Math.max(10,Math.round(H/40))+'px '+FONTS.sans;c.textAlign='right';c.textBaseline='bottom';c.fillStyle='#ffffff';c.fillText('Échap : passer',W-12,H-8);c.restore();}}
 var DATA={},gpaused=false,btnFocus=null,btnUsedKeys=false;
 function dataInit(){DATA={};(P.objects||[]).forEach(function(o){if(o.kind==='array')DATA[o.id]={list:(o.items||[]).map(function(v){return v;})};else if(o.kind==='dict')DATA[o.id]={map:Object.assign({},o.entries||{})};});}
 dataInit();
 function dval(v){if(typeof v==='number')return v;var n=Number(v);return v!==''&&!isNaN(n)&&typeof v!=='boolean'?n:v;}
 var cp=null,camPx=0,camPy=0,ialpha=1,zoom=1,zoomA=null,camLook=null,camB=null,camLA=0;
 function zf(ly,i){return (i&&i.hud)||(ly&&ly.px===0&&ly.py===0)?1:zoom;}
 /* ---- remappable controls ---- */
 var CTL=(S.controls||[]).filter(function(c){return c&&c.keys&&c.keys.length;}),ckey='pfkeys:'+(S.title||'jeu'),cbind=[],ctl={open:false,sel:0,wait:false,hit:null},physDown={},v2p={},p2v={},blocked={};
 function ctlLoad(){var saved=null;try{saved=JSON.parse(localStorage.getItem(ckey)||'null');}catch(e){}cbind=CTL.map(function(c,k){var sv=saved&&saved[c.name];return sv&&sv.length?sv.slice():c.keys.slice();});ctlMap();}
 function ctlSave(){var o={};CTL.forEach(function(c,k){o[c.name]=cbind[k];});try{localStorage.setItem(ckey,JSON.stringify(o));}catch(e){}}
 function ctlMap(){p2v={};v2p={};blocked={};CTL.forEach(function(c,k){c.keys.forEach(function(v){blocked[v]=1;});cbind[k].forEach(function(ph){(p2v[ph]=p2v[ph]||[]);c.keys.forEach(function(v){if(p2v[ph].indexOf(v)<0)p2v[ph].push(v);(v2p[v]=v2p[v]||[]);if(v2p[v].indexOf(ph)<0)v2p[v].push(ph);});});});}
 function ctlPress(code,down){var vs=p2v[code];if(!vs){if(blocked[code])return;vs=[code];}physDown[code]=down;
  vs.forEach(function(v){var any=false;(v2p[v]||[code]).forEach(function(ph){if(physDown[ph])any=true;});if(!v2p[v])any=down;if(any&&!keys[v]){keys[v]=true;kp[v]=true;anyPress=true;}else if(!any&&keys[v]){keys[v]=false;kr[v]=true;}});}
 function ctlOpen(){if(!CTL.length)return;ctl.open=true;ctl.sel=0;ctl.wait=false;keys={};physDown={};}
 function ctlKey(e){var c=e.code;if(ctl.wait){if(c==='Escape'){ctl.wait=false;return;}CTL.forEach(function(x,k){if(k!==ctl.sel)cbind[k]=cbind[k].filter(function(ph){return ph!==c;});});cbind[ctl.sel]=[c];CTL.forEach(function(x,k){if(!cbind[k].length)cbind[k]=x.keys.slice(0,1).filter(function(d){return d!==c;});});ctl.wait=false;ctlMap();ctlSave();sfx('blip',mst());return;}
  if(c==='ArrowUp'||c==='KeyW')ctl.sel=(ctl.sel+CTL.length)%(CTL.length+1);
  else if(c==='ArrowDown'||c==='KeyS')ctl.sel=(ctl.sel+1)%(CTL.length+1);
  else if(c==='Enter'||c==='Space'||c==='NumpadEnter'){if(ctl.sel===CTL.length){ctl.open=false;}else ctl.wait=true;}
  else if(c==='Backspace'||c==='Delete'){if(ctl.sel<CTL.length){cbind[ctl.sel]=CTL[ctl.sel].keys.slice();ctlMap();ctlSave();}}
  else if(c==='Escape'||c==='KeyC')ctl.open=false;}
 function ctlClick(){if(!ctl.hit)return;for(var q=0;q<ctl.hit.length;q++){var h=ctl.hit[q];if(mouse.sx>=h[0]&&mouse.sx<=h[2]&&mouse.sy>=h[1]&&mouse.sy<=h[3]){ctl.sel=q;if(q===CTL.length)ctl.open=false;else ctl.wait=true;return;}}}
 function keyLabel(code){return KEYNAME[code]||(code.indexOf('Key')===0?code.slice(3):code.indexOf('Digit')===0?code.slice(5):code);}
 function drawHs(c){var q=hsIn,fs=Math.max(14,Math.round(H/26)),font=FONTS[DS.font]||FONTS.sans,pw=Math.min(W-40,Math.max(380,fs*(q.max*1.05+6))),ph=fs*9.4,px=(W-pw)/2,py=(H-ph)/2;
  c.save();c.textBaseline='middle';c.fillStyle='rgba(0,0,0,.6)';c.fillRect(0,0,W,H);c.fillStyle=DS.bg;rr(c,px,py,pw,ph,14);c.fill();c.strokeStyle=DS.accent;c.globalAlpha=.6;c.lineWidth=2;rr(c,px+1,py+1,pw-2,ph-2,13);c.stroke();c.globalAlpha=1;
  c.textAlign='center';c.fillStyle=DS.accent;c.font='800 '+Math.round(fs*1.3)+'px '+font;c.fillText(q.title,W/2,py+fs*1.4);
  var bw=fs*(q.max*.95+1.6),bx=W/2-bw/2,by=py+fs*2.7,bh=fs*2.1;c.fillStyle='rgba(255,255,255,.08)';rr(c,bx,by,bw,bh,8);c.fill();c.strokeStyle='rgba(255,255,255,.25)';c.lineWidth=1.5;rr(c,bx,by,bw,bh,8);c.stroke();
  c.font='800 '+Math.round(fs*1.2)+'px '+(FONTS.mono||font);c.textAlign='left';var cw=c.measureText('M').width,tw=cw*q.max,tx=W/2-tw/2,ty=by+bh/2;
  for(var k=0;k<q.max;k++){var ch=q.name.charAt(k);c.fillStyle=k<q.name.length?DS.fg:'rgba(255,255,255,.18)';c.fillText(k<q.name.length?ch:'_',tx+k*cw,ty);}
  if(Math.floor(q.t*2.5)%2===0){c.fillStyle=DS.accent;c.fillRect(tx+q.cur*cw,ty+fs*.62,cw*.9,3);}
  var okw=fs*6,okh=fs*1.7,okx=W/2-okw/2,oky=py+ph-fs*2.6;c.fillStyle=DS.accent;rr(c,okx,oky,okw,okh,8);c.fill();c.fillStyle=DS.bg;c.textAlign='center';c.font='800 '+Math.round(fs*.95)+'px '+font;c.fillText('Valider',W/2,oky+okh/2+1);q.hit=[okx,oky,okx+okw,oky+okh];
  c.fillStyle=DS.fg;c.globalAlpha=.55;c.font='500 '+Math.round(fs*.68)+'px '+font;c.fillText('Tapez votre nom · ↑↓ changer la lettre · Entrée valider',W/2,by+bh+fs*.85);c.restore();}
 function drawCtl(c){var fs=Math.max(13,Math.round(H/32)),lh=Math.round(fs*2.1),pw=Math.min(W-40,460),rows=CTL.length+1,ph=rows*lh+fs*4.6,px=(W-pw)/2,py=Math.max(10,(H-ph)/2),font=FONTS[DS.font]||FONTS.sans;
  c.save();c.textAlign='left';c.fillStyle='rgba(0,0,0,.55)';c.fillRect(0,0,W,H);c.globalAlpha=.97;c.fillStyle=DS.bg;rr(c,px,py,pw,ph,12);c.fill();c.globalAlpha=1;c.strokeStyle=DS.accent;c.globalAlpha=.5;c.lineWidth=2;rr(c,px+1,py+1,pw-2,ph-2,11);c.stroke();c.globalAlpha=1;
  c.fillStyle=DS.fg;c.font='800 '+Math.round(fs*1.25)+'px '+font;c.textBaseline='top';c.fillText('Touches',px+fs*1.2,py+fs*.9);
  c.font='500 '+Math.round(fs*.78)+'px '+font;c.globalAlpha=.6;c.fillText('↑ ↓ choisir · Entrée changer · Retour : par défaut · Échap fermer',px+fs*1.2,py+fs*2.5);c.globalAlpha=1;
  ctl.hit=[];var y0=py+fs*3.8;
  for(var k=0;k<rows;k++){var y=y0+k*lh,on=k===ctl.sel;if(on){c.fillStyle=DS.accent;c.globalAlpha=.16;rr(c,px+fs*.6,y,pw-fs*1.2,lh-6,8);c.fill();c.globalAlpha=1;}
   c.font='700 '+fs+'px '+font;c.textBaseline='middle';c.fillStyle=on?DS.accent:DS.fg;
   if(k===CTL.length){c.textAlign='center';c.fillText('Fermer',W/2,y+(lh-6)/2);c.textAlign='left';}
   else{c.fillText(CTL[k].name,px+fs*1.3,y+(lh-6)/2);var lab=ctl.wait&&on?'Appuyez sur une touche…':cbind[k].map(keyLabel).join(' / ');c.textAlign='right';c.font='600 '+fs+'px '+font;c.fillStyle=ctl.wait&&on?DS.accent:DS.fg;c.globalAlpha=ctl.wait&&on?(Math.floor(time*0+performance.now()/400)%2?1:.5):.85;c.fillText(lab,px+pw-fs*1.3,y+(lh-6)/2);c.globalAlpha=1;c.textAlign='left';}
   ctl.hit.push([px+fs*.6,y,px+pw-fs*.6,y+lh-6]);}
  c.restore();}
 ctlLoad();
 var dlg={queue:[],cur:null,speed:45,choice:0,choiceText:'',choiceTick:-1,endTick:-1};
 var NAVK={ArrowUp:1,ArrowDown:1,ArrowLeft:1,ArrowRight:1,KeyW:1,KeyA:1,KeyS:1,KeyD:1,Space:1,Enter:1,NumpadEnter:1,KeyE:1,Escape:1};
 function navKeys(){var r={};for(var ph in pkp){if(NAVK[ph]){r[ph]=1;continue;}var vs=p2v[ph]||[ph];if(vs.indexOf('Space')>=0||vs.indexOf('Enter')>=0){r.Enter=1;continue;}vs.forEach(function(v){r[v]=1;});}return r;}
 function btnStep(){var bl=all.filter(function(i){return !i.dead&&i.btn&&i.visible&&i.btn.en&&(!layers[i.layer]||layers[i.layer].visible);});
  all.forEach(function(i){if(i.btn){i.btn.hover=false;i.btn.press=false;}});if(btnFocus&&bl.indexOf(btnFocus)<0){btnFocus.btn.focus=false;btnFocus=null;}if(!bl.length)return;
  var hov=null;for(var k=bl.length-1;k>=0;k--)if(mouseOver(bl[k])){hov=bl[k];break;}if(hov){hov.btn.hover=true;if(mouse.down.left)hov.btn.press=true;if(mouse.pr.left){hov.btn.clicked=tick;mouse.pr={};sfx('blip',mst());}}
  var nav=gpaused||!all.some(function(i){return !i.dead&&((i.b.platform&&i.b.platform.en&&i.b.platform.p.controls)||(i.b.eightdir&&i.b.eightdir.en&&i.b.eightdir.p.controls)||(i.b.car&&i.b.car.en&&i.b.car.ctl));});if(!nav)return;
  bl.sort(function(a,b){return Math.abs(a.y-b.y)>4?a.y-b.y:a.x-b.x;});if(!btnFocus){btnFocus=bl[0];bl[0].btn.focus=true;}
  var cur0=bl.indexOf(btnFocus),nk=navKeys(),mv=(nk.ArrowDown||nk.KeyS||nk.ArrowRight||nk.KeyD)?1:(nk.ArrowUp||nk.KeyW||nk.ArrowLeft||nk.KeyA)?-1:0;
  if(mv){btnFocus.btn.focus=false;btnFocus=bl[(cur0+mv+bl.length)%bl.length];btnFocus.btn.focus=true;btnUsedKeys=true;sfx('blip',mst());kp={};pkp={};anyPress=false;}
  if(nk.Enter||nk.Space||nk.NumpadEnter){btnFocus.btn.clicked=tick;btnFocus.btn.press=true;kp={};pkp={};anyPress=false;sfx('blip',mst());}}
 function dlgPush(d){d.shown=0;d.acc=0;d.sel=0;d.t=0;dlg.queue.push(d);if(!dlg.cur)dlgNext();}
 function dlgNext(){dlg.cur=dlg.queue.shift()||null;}
 var DLGADV={Space:1,Enter:1,KeyE:1,NumpadEnter:1};
 function dlgStep(dt){var d=dlg.cur;if(!d)return;d.t+=dt;var len=d.text.length,adv=false;var nk0=navKeys();for(var k in DLGADV)if(nk0[k])adv=true;var clk=mouse.pr.left;
  if(d.shown<len){if(dlg.speed<=0)d.shown=len;else{d.acc+=dt*dlg.speed;while(d.acc>=1&&d.shown<len){d.acc-=1;var ch=d.text.charAt(d.shown);d.shown++;if(/[.!?…]/.test(ch))d.acc-=6;else if(/[,;:]/.test(ch))d.acc-=2.5;if(DS.blip&&d.shown%3===0&&/\S/.test(ch))sfx('blip',mst(),.25);}}
   if((adv||clk)&&d.t>.12){d.shown=len;}}
  else if(d.choices){var nk=navKeys(),up=nk.ArrowUp||nk.KeyW,dn=nk.ArrowDown||nk.KeyS;if(up)d.sel=(d.sel+d.choices.length-1)%d.choices.length;if(dn)d.sel=(d.sel+1)%d.choices.length;
   if(clk&&d.hit){for(var q=0;q<d.hit.length;q++){var h=d.hit[q];if(mouse.sx>=h[0]&&mouse.sx<=h[2]&&mouse.sy>=h[1]&&mouse.sy<=h[3]){d.sel=q;adv=true;}}}
   if(adv&&d.t>.15){dlg.choice=d.sel+1;dlg.choiceText=d.choices[d.sel];dlg.choiceTick=tick;sfx('blip',mst());dlgNext();if(!dlg.cur)dlg.endTick=tick;}}
  else if((adv||clk)&&d.t>.15){dlgNext();if(!dlg.cur)dlg.endTick=tick;}
  kp={};pkp={};mouse.pr={};anyPress=false;}
 function frame(ts){if(!running)return;raf=requestAnimationFrame(frame);engIdle();pollPad();fpsN++;if(!fpsT)fpsT=ts;if(ts-fpsT>=1000){fpsV=Math.round(fpsN*1000/(ts-fpsT));fpsN=0;fpsT=ts;}
  if(ready&&(paused||stepReq)){last=ts;acc=0;ialpha=1;if(stepReq){stepReq=0;step(DT);}}
  else if(ready){var dt=Math.min(.1,Math.max(0,(ts-(last||ts))/1000));last=ts;
   if(tr){tr.t+=dt;if(tr.phase==='out'){if(tr.t>=tr.d){loadLayout(tr.target);tr.phase='in';tr.t=0;acc=0;}}else if(tr.t>=tr.d)tr=null;}
   if(!tr||tr.phase==='in'){acc+=dt*dbgSpeed*(dlg.cur||ctl.open?1:tscale);var k=0;while(acc>=DT&&k<6&&!(tr&&tr.phase==='out')){step(DT);acc-=DT;k++;}if(k>=6)acc=0;}
   ialpha=(paused||dlg.cur||ctl.open||tr&&tr.phase==='out')?1:Math.max(0,Math.min(1,acc/DT));}
  else ialpha=1;
  render();}
 var startId=opts.startLayout||S.startLayout||(P.layouts[0]&&P.layouts[0].id);
 var EX=Object.assign({bg:'#0d0f13',accent:'#ff8a3d',showTitle:true,logo:''},S.export||{}),loadedN=0,splash=false;
 function drawLoader(c,s,ox,oy){c.setTransform(s,0,0,s,ox,oy);c.fillStyle=EX.bg;c.fillRect(0,0,W,H);c.textAlign='center';c.textBaseline='middle';var cy=H/2,fg='#ffffff';
  var lo=EX.logo&&types[EX.logo]&&types[EX.logo].def.anims?img(types[EX.logo].def.anims[0].frames[0]):null;if(lo&&lo.complete&&lo.naturalWidth){var k=Math.min(96/lo.naturalWidth,96/lo.naturalHeight);c.imageSmoothingEnabled=!S.pixelArt;c.drawImage(lo,W/2-lo.naturalWidth*k/2,cy-70-lo.naturalHeight*k/2,lo.naturalWidth*k,lo.naturalHeight*k);}
  if(EX.showTitle!==false){c.fillStyle=fg;c.font='800 '+Math.round(H/16)+'px '+FONTS.round;c.fillText(S.title||'',W/2,cy);}
  if(splash){c.globalAlpha=.55+.45*Math.sin(performance.now()/300);c.fillStyle=EX.accent;c.font='700 '+Math.round(H/26)+'px '+FONTS.round;c.fillText(('ontouchstart' in window)?'Touchez l’écran pour jouer':'Cliquez ou appuyez sur une touche pour jouer',W/2,cy+H/9);c.globalAlpha=1;return;}
  var bw=Math.min(280,W*.5),pr=srcs.length?loadedN/srcs.length:1;c.fillStyle='rgba(255,255,255,.12)';rr(c,W/2-bw/2,cy+H/10,bw,8,4);c.fill();c.fillStyle=EX.accent;rr(c,W/2-bw/2,cy+H/10,Math.max(8,bw*pr),8,4);c.fill();}
 function boot(){ready=true;(P.sounds||[]).forEach(function(s){loadSnd(s.id);});loadLayout(startId);}
 var pend=srcs.map(function(s){return new Promise(function(r0){var res=function(){loadedN++;r0();};var m=img(s);if(m.complete&&m.naturalWidth)res();else{m.addEventListener('load',res);m.addEventListener('error',res);}});});
 Promise.all(pend).then(function(){if(!running)return;if(opts.splash)splash=true;else boot();});
 raf=requestAnimationFrame(frame);
 return{
  stop:function(){engStopAll();running=false;cancelAnimationFrame(raf);window.removeEventListener('keydown',kd);window.removeEventListener('keyup',ku);window.removeEventListener('blur',bl);canvas.removeEventListener('pointerdown',pd);canvas.removeEventListener('pointermove',pmv);window.removeEventListener('pointerup',pup);window.removeEventListener('pointercancel',pup);canvas.removeEventListener('contextmenu',cmenu);canvas.style.cursor='';stopAllSnd();if(master){try{master.disconnect();}catch(e){}master=null;}if(touchEl)touchEl.remove();},
  restart:function(){if(!ready)return;stopAllSnd();resetG();dataInit();cp=null;tr=null;loadLayout(layout?layout.id:startId);},
  cineActive:function(){return !!cine;},bench:function(n){var t0=performance.now();for(var k=0;k<n;k++)step(DT);var t1=performance.now();return{step:(t1-t0)/n};},
  debug:function(){return{groups:groupState,funcs:Object.keys(funcs),music:music&&music.id,voices:voices.length,mouse:mouse,storageGet:sget,layout:layout&&layout.name,globals:globals,count:all.length,errors:errors,find:function(n){var t=byName[n];return t?t.insts:[];}};},
  layoutName:function(){return layout?layout.name:'';},
  setPaused:function(b){paused=!!b;if(!b)brkHit=null;},hits:function(){return evHits;},setBreakpoints:function(m){bps=m||{};},breakHit:function(){return brkHit;},setSpeed:function(f){dbgSpeed=Math.max(.05,Math.min(4,+f||1));},fps:function(){return fpsV;},
  evalExpr:function(src){try{var sv=cur,ss=curSol;cur=null;curSol={};var v;try{v=compile(String(src))(ENV,F);}finally{cur=sv;curSol=ss;}return{v:v};}catch(e){return{err:e.message||String(e)};}},
  isPaused:function(){return paused;},stepFrame:function(){stepReq=1;},highlight:function(u){hl=u||null;},showCollisions:function(b){showCol=!!b;},openControls:function(){ctlOpen();},
  setGlobal:function(n,v){if(n in globals)globals[n]=typeof globals[n]==='string'?String(v):(+v||0);},
  snapshot:function(){var gr={};Object.keys(groupsByName).forEach(function(n){gr[n]=groupsByName[n].some(groupOn);});var ty=[];for(var k in types){var t=types[k];ty.push({id:t.id,name:t.name,count:t.insts.filter(alive).length});}
   return{layout:layout?layout.name:'',tick:tick,time:time,paused:paused,globals:Object.assign({},globals),groups:gr,types:ty,count:all.filter(alive).length,errors:errors.slice(),music:!!music};},
  instances:function(tid){var t=types[tid];return t?t.insts.filter(alive).slice(0,60).map(function(i){return{uid:i.uid,x:i.x,y:i.y};}):[];},
  inspect:function(u){var i=all.find(function(x){return x.uid===u&&!x.dead;});if(!i)return null;var o=i.t.def,b={};
   for(var k in i.b){var st=i.b[k],v={};if(k==='platform'){v['vitesse X']=st.vx;v['vitesse Y']=st.vy;v['au sol']=!!st.floor;v['actif']=st.en;}else if(k==='patrol'){v.direction=st.dir<0?'gauche':'droite';}else if(k==='timer'){Object.keys(st.timers).forEach(function(n){v['minuterie '+n]=st.timers[n].left;});}else if(k==='bullet'){v['vitesse X']=st.vx;v['vitesse Y']=st.vy;}else if(k==='sine'){v.actif=st.en;}b[(BEHAVIORS[k]||{}).n||k]=v;}
   return{uid:i.uid,name:i.t.name,props:{X:i.x,Y:i.y,Largeur:i.w,Hauteur:i.h,Angle:i.angle,Opacité:Math.round(i.opacity*100),Visible:i.visible,Miroir:i.mirrored,Calque:(layers[i.layer]||{}).name,Animation:o.kind==='sprite'&&o.anims[i.anim]?o.anims[i.anim].name+' ('+i.frame+')':undefined,Texte:o.kind==='text'?i.text:undefined},vars:Object.assign({},i.vars),beh:b};}
 };
}

G.PF={rdSetLookup:function(f){RDLK=f;},mmSetSource:function(f){MMSRC=f;},rdBuild:rdBuild,ISTYLE:ISTYLE,CMP:CMP,CMPSYM:CMPSYM,KEYS:KEYS,KEYNAME:KEYNAME,SOUNDS:SOUNDS,BEHAVIORS:BEHAVIORS,CONDS:CONDS,ACTS:ACTS,PROVNAME:PROVNAME,providersOf:providersOf,bdefaults:bdefaults,
 EXF:EXF,SYSEXP:SYSEXP,OBJEXP:OBJEXP,compile:compile,checkExpr:checkExpr,drawInst:drawInst,drawBox:drawBox,colBox:colBox,tmDirty:tmDirty,TILETYPES:TILETYPES,tilePoly:tilePoly,PTPRESETS:PTPRESETS,ptDefaults:ptDefaults,ptEmit:ptEmit,ptStep:ptStep,ptDraw:ptDraw,imagePoint:imgPoint,colPoly:colPoly,polyOvl:polyOvl,OBJMETH:OBJMETH,FONTS:FONTS,sfx:sfx,audio:audio,Runtime:Runtime};
})(window);
