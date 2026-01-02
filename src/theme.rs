use egui::{Color32, Visuals};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub is_dark: bool,
    pub background: [u8; 3],
    pub foreground: [u8; 3],
    pub accent: [u8; 3],
    pub sidebar_bg: [u8; 3],
    pub editor_bg: [u8; 3],
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark() // Changed from light() to dark()
    }
}

impl Theme {
    pub fn light() -> Self {
        Self {
            name: "Light".to_string(),
            is_dark: false,
            background: [240, 240, 240],
            foreground: [20, 20, 20],
            accent: [0, 120, 215],
            sidebar_bg: [250, 250, 250],
            editor_bg: [255, 255, 255],
        }
    }
    
    pub fn dark() -> Self {
        Self {
            name: "Dark".to_string(),
            is_dark: true,
            background: [30, 30, 30],
            foreground: [230, 230, 230],
            accent: [100, 180, 255],
            sidebar_bg: [25, 25, 25],
            editor_bg: [35, 35, 35],
        }
    }
    
    pub fn solarized_light() -> Self {
        Self {
            name: "Solarized Light".to_string(),
            is_dark: false,
            background: [253, 246, 227],
            foreground: [101, 123, 131],
            accent: [38, 139, 210],
            sidebar_bg: [238, 232, 213],
            editor_bg: [253, 246, 227],
        }
    }
    
    pub fn solarized_dark() -> Self {
        Self {
            name: "Solarized Dark".to_string(),
            is_dark: true,
            background: [0, 43, 54],
            foreground: [131, 148, 150],
            accent: [38, 139, 210],
            sidebar_bg: [7, 54, 66],
            editor_bg: [0, 43, 54],
        }
    }
    
    pub fn nord() -> Self {
        Self {
            name: "Nord".to_string(),
            is_dark: true,
            background: [46, 52, 64],
            foreground: [216, 222, 233],
            accent: [136, 192, 208],
            sidebar_bg: [59, 66, 82],
            editor_bg: [46, 52, 64],
        }
    }
    
    pub fn dracula() -> Self {
        Self {
            name: "Dracula".to_string(),
            is_dark: true,
            background: [40, 42, 54],
            foreground: [248, 248, 242],
            accent: [189, 147, 249],
            sidebar_bg: [68, 71, 90],
            editor_bg: [40, 42, 54],
        }
    }
    
    pub fn monokai() -> Self {
        Self {
            name: "Monokai".to_string(),
            is_dark: true,
            background: [39, 40, 34],
            foreground: [248, 248, 240],
            accent: [249, 38, 114],
            sidebar_bg: [30, 31, 26],
            editor_bg: [39, 40, 34],
        }
    }
    
    pub fn apply_to_egui(&self, ctx: &egui::Context) {
        let mut visuals = if self.is_dark {
            Visuals::dark()
        } else {
            Visuals::light()
        };
        
        // Customize colors
        visuals.window_fill = Color32::from_rgb(self.background[0], self.background[1], self.background[2]);
        visuals.panel_fill = Color32::from_rgb(self.sidebar_bg[0], self.sidebar_bg[1], self.sidebar_bg[2]);
        visuals.extreme_bg_color = Color32::from_rgb(self.editor_bg[0], self.editor_bg[1], self.editor_bg[2]);
        
        ctx.set_visuals(visuals);
    }
    
    pub fn available_themes() -> Vec<Theme> {
        vec![
            Self::light(),
            Self::dark(),
            Self::solarized_light(),
            Self::solarized_dark(),
            Self::nord(),
            Self::dracula(),
            Self::monokai(),
        ]
    }
    
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }
    
    pub fn load(path: &Path) -> std::io::Result<Self> {
        let json = fs::read_to_string(path)?;
        let theme = serde_json::from_str(&json)?;
        Ok(theme)
    }
}

pub struct ThemeManager {
    pub current_theme: Theme,
    pub available_themes: Vec<Theme>,
}

impl ThemeManager {
    pub fn new() -> Self {
        Self {
            current_theme: Theme::default(),
            available_themes: Theme::available_themes(),
        }
    }
    
    pub fn load_or_default(config_path: &Path) -> Self {
        let theme = Theme::load(config_path).unwrap_or_default();
        Self {
            current_theme: theme,
            available_themes: Theme::available_themes(),
        }
    }
    
    pub fn set_theme(&mut self, theme: Theme) {
        self.current_theme = theme;
    }
    
    pub fn toggle_dark_mode(&mut self) {
        self.current_theme = if self.current_theme.is_dark {
            Theme::light()
        } else {
            Theme::dark()
        };
    }
}
