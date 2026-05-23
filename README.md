# Quasar - Éditeur d'Image

C'est le projet Quasar Secure, un éditeur d'image développé en Rust.

## Lancer le projet

Pour lancer l'application avec l'image par défaut, placez-vous dans le dossier `Code` et exécutez la commande suivante :

cargo run

Si vous n'avez pas de version récente du compilateur Rust, exécutez cette commande avant de compiler :

source $HOME/.cargo/env && cargo run

### Choisir une image au lancement

Vous pouvez spécifier une image différente au démarrage en renseignant son nom en argument :

cargo run "nom de l'image.png"


## Structure des dossiers

- **`Images`** : Ce dossier contient les images sources à traiter. L'application ira chercher les images dans ce dossier par défaut.
- **`Code`** : Contient tout le code source en Rust de l'application.
