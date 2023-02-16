export default {
    menu: {
        play: 'Jouer',
        changelog: 'Notes',
        mods: 'Mods',
        settings: 'Paramètres',
        dev: 'Dev'
    },

    generic: {
        error: 'Erreur',
        cancel: 'Annuler',
    },

    play: {
        button: {
            northstar_is_running: "En cours d'utilisation",
            select_game_dir: 'Sélectionner le dossier du jeu',
            install: 'Installer',
            installing: 'Installation...',
            update: 'Mettre à jour',
            updating: 'Mise à jour...',
            ready_to_play: 'Jouer'
        },

        unknown_version: "Version inconnue",
        see_patch_notes: "voir les notes de version",
        players: "joueurs",
        servers: "serveurs",
        unable_to_load_playercount: "Impossible de charger les statistiques",
        northstar_running: "Northstar est en cours d'exécution :",
        origin_running: "Origin est en cours d'exécution :"
    },

    mods: {
        local: {
            no_mods: "Aucun mod trouvé.",
            delete_confirm: "Êtes-vous certain de vouloir supprimer ce mod ?",
            delete: "Supprimer",
            part_of_ts_mod: "Ce mod Northstar fait partie d'un mod Thunderstore",
            success_deleting: "Succès de la suppression de {modName}"
        },

        online: {
            no_match: "Aucun mod correspondant n'a été trouvé.",
            try_another_search: "Essayez une autre recherche !"
        },

        menu: {
            local: 'Local',
            online: 'En ligne',
            filter: 'Filtrer',
            search: 'Chercher',
            sort_mods: 'Trier les mods',
            select_categories: 'Choisir les catégories',

            sort: {
                name_asc: 'Par nom (de A à Z)',
                name_desc: 'Par nom (de Z à A)',
                date_asc: 'Par date (du plus vieux)',
                date_desc: 'Par date (du plus récent)',
                most_downloaded: "Plus téléchargés",
                top_rated: "Mieux notés"
            }
        },

        card: {
            button: {
                being_installed: "Installation...",
                being_updated: "Mise à jour...",
                installed: "Installé",
                install: "Installer",
                outdated: "Mettre à jour"
            },

            more_info: "Plus d'informations",
            remove: "Supprimer le mod",
            remove_dialog_title: "Attention !",
            remove_dialog_text: "Voulez-vous vraiment supprimer ce mod Thunderstore ?",
            remove_success: "{modName} supprimé",
            install_success: "{modName} installé"
        }
    },

    settings: {
        manage_install: "Gérer l'installation",
        choose_folder: "Choisir le dossier d'installation du jeu",
        nb_ts_mods_per_page: "Nombre de mods Thunderstore par page",
        nb_ts_mods_per_page_desc1: "Ce paramètre a un impact sur les performances d'affichage des mods Thunderstore.",
        nb_ts_mods_per_page_desc2: "Réglez-le sur 0 pour désactiver la pagination.",
        nb_ts_mods_reset: "Valeur par défaut",
        language: 'Langue',
        language_select: "Sélectionnez votre langue",
        about: "À propos:",
        flightcore_version: "Version de FlightCore :",
        testing: "Tests :",
        enable_test_channels: "Activer le test de versions de pré-production",
        dev_mode_enabled_title: "Attention !",
        dev_mod_enabled_text: "Mode développeur activé."
    }
};
