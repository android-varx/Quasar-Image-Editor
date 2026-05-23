use eframe::egui;

/// Couleur d'accent principale (violet Quasar)
pub const ACCENT: egui::Color32 = egui::Color32::from_rgb(140, 130, 255);

/// Arrière-plan des panneaux
pub const PANEL_BG: egui::Color32 = egui::Color32::from_rgb(30, 30, 38);

/// Arrière-plan des fenêtres
pub const WINDOW_BG: egui::Color32 = egui::Color32::from_rgb(35, 35, 45);

/// Arrière-plan de la barre supérieure
pub const TOP_BAR_BG: egui::Color32 = egui::Color32::from_rgb(22, 22, 30);

/// Arrière-plan de la barre latérale
pub const SIDEBAR_BG: egui::Color32 = egui::Color32::from_rgb(25, 25, 33);

/// Arrière-plan des boutons (inactif)
pub const BTN_BG: egui::Color32 = egui::Color32::from_rgb(40, 40, 55);

/// Arrière-plan des boutons (actif)
pub const BTN_ACTIVE_BG: egui::Color32 = egui::Color32::from_rgb(80, 70, 160);

/// Arrière-plan des boutons de la barre supérieure
pub const TOP_BTN_BG: egui::Color32 = egui::Color32::from_rgb(45, 45, 60);

/// Arrière-plan des cadres internes (paramètres, couleur)
pub const FRAME_BG: egui::Color32 = egui::Color32::from_rgb(35, 35, 48);

/// Texte secondaire / labels
pub const TEXT_MUTED: egui::Color32 = egui::Color32::from_rgb(160, 160, 180);

/// Texte inactif
pub const TEXT_INACTIVE: egui::Color32 = egui::Color32::from_rgb(180, 180, 200);

/// Texte très discret (dimensions, etc.)
pub const TEXT_DIM: egui::Color32 = egui::Color32::from_rgb(100, 100, 120);

/// Couleur de séparateur
pub const SEPARATOR: egui::Color32 = egui::Color32::from_rgb(55, 55, 70);


/// Bordure des cadres flottants (zoom, etc.)
pub const FLOAT_BORDER: egui::Color32 = egui::Color32::from_rgb(60, 60, 80);

/// Applique le thème sombre Quasar au contexte egui.
pub fn apply_theme(ctx: &egui::Context)
{
    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals::dark();
    style.visuals.window_corner_radius = egui::CornerRadius::same(12);
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(10.0, 6.0);
    style.visuals.panel_fill = PANEL_BG;
    style.visuals.window_fill = WINDOW_BG;
    style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(40, 40, 52);
    ctx.set_style(style);
}
