use eframe::egui;
use std::sync::Arc;
use std::sync::Mutex;

mod note;
mod storage;
mod search;
mod theme;
mod encryption;
mod tags;
mod pdf_export;
mod images;
mod links;
mod version_control;
mod autocomplete;
mod spellcheck;

use storage::Storage;
use search::FuzzySearch;
use theme::ThemeManager;
use encryption::Encryption;
use tags::TagManager;
use links::LinkManager;
use version_control::VersionControl;
use autocomplete::Autocomplete;
use spellcheck::SpellChecker;
use std::path::PathBuf;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Notetaking App",
        options,
        Box::new(|_cc| Ok(Box::new(NoteTakingApp::new()))),
    )
}

struct NoteTakingApp {
    storage: Arc<Mutex<Storage>>,
    search: FuzzySearch,
    
    // Enhanced features
    theme_manager: ThemeManager,
    encryption: Encryption,
    tag_manager: TagManager,
    link_manager: LinkManager,
    version_control: Option<VersionControl>,
    autocomplete: Autocomplete,
    spellcheck: SpellChecker,
    
    // UI State
    selected_folder: Option<usize>,
    selected_note: Option<usize>,
    current_note_content: String,
    search_query: String,
    search_results: Vec<(usize, usize)>, // (folder_idx, note_idx)
    
    // Folder management
    new_folder_name: String,
    show_new_folder_dialog: bool,
    
    // Note management
    new_note_title: String,
    show_new_note_dialog: bool,
    
    // UI flags
    sidebar_open: bool,
    show_markdown_preview: bool,
    
    // Theme management
    show_theme_dialog: bool,
    
    // Tag management
    show_tag_dialog: bool,
    new_tag_name: String,
    selected_tag_filter: Option<usize>,
    show_tag_editor: bool,
    
    // Encryption
    show_encryption_dialog: bool,
    encryption_password: String,
    confirm_password: String,
    
    // Export
    show_export_dialog: bool,
    export_format: ExportFormat,
    
    // Links panel
    show_links_panel: bool,
    
    // Version history
    show_version_history: bool,
    note_versions: Vec<version_control::Version>,
    selected_version: Option<usize>,
    version_timeline_position: f32, // 0.0 to 1.0 for slider
    
    // Images
    show_image_dialog: bool,
    
    // Statistics
    show_statistics: bool,
    
    // Settings
    show_settings: bool,
    auto_save_enabled: bool,
    auto_save_interval: f32,
    last_save_time: std::time::Instant,
    autocomplete_enabled: bool,
    spellcheck_enabled: bool,
    
    // Autocomplete state
    autocomplete_suggestions: Vec<String>,
    show_autocomplete: bool,
    
    // Spell check state
    misspelled_words: Vec<(usize, usize, String)>,
    
    // Favorites
    favorite_notes: Vec<(usize, usize)>,
    show_favorites: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ExportFormat {
    PDF,
    Markdown,
    PlainText,
}

struct NoteStatistics {
    total_notes: usize,
    total_folders: usize,
    total_words: usize,
    total_chars: usize,
    encrypted_count: usize,
    total_tags: usize,
    favorite_count: usize,
}

impl NoteTakingApp {
    fn new() -> Self {
        let storage = Storage::new("./notes_data".to_string());
        let search = FuzzySearch::new();
        let theme_manager = ThemeManager::new();
        let encryption = Encryption::new();
        let tag_manager = TagManager::new();
        let link_manager = LinkManager::new();
        let autocomplete = Autocomplete::new();
        let spellcheck = SpellChecker::new();
        
        // Initialize version control
        let version_control = VersionControl::new(PathBuf::from("./notes_data"))
            .ok()
            .and_then(|vc| {
                vc.init().ok()?;
                Some(vc)
            });
        
        Self {
            storage: Arc::new(Mutex::new(storage)),
            search,
            theme_manager,
            encryption,
            tag_manager,
            link_manager,
            version_control,
            autocomplete,
            spellcheck,
            selected_folder: None,
            selected_note: None,
            current_note_content: String::new(),
            search_query: String::new(),
            search_results: Vec::new(),
            new_folder_name: String::new(),
            show_new_folder_dialog: false,
            new_note_title: String::new(),
            show_new_note_dialog: false,
            sidebar_open: true,
            show_markdown_preview: false,
            show_theme_dialog: false,
            show_tag_dialog: false,
            new_tag_name: String::new(),
            selected_tag_filter: None,
            show_tag_editor: false,
            show_encryption_dialog: false,
            encryption_password: String::new(),
            confirm_password: String::new(),
            show_export_dialog: false,
            export_format: ExportFormat::PDF,
            show_links_panel: false,
            show_version_history: false,
            note_versions: Vec::new(),
            selected_version: None,
            version_timeline_position: 1.0, // Start at most recent (1.0 = latest)
            show_image_dialog: false,
            show_statistics: false,
            show_settings: false,
            auto_save_enabled: true,
            auto_save_interval: 30.0,
            last_save_time: std::time::Instant::now(),
            autocomplete_enabled: true,
            spellcheck_enabled: true,
            autocomplete_suggestions: Vec::new(),
            show_autocomplete: false,
            misspelled_words: Vec::new(),
            favorite_notes: Vec::new(),
            show_favorites: false,
        }
    }
    
    fn save_current_note(&mut self) {
        if let (Some(folder_idx), Some(note_idx)) = (self.selected_folder, self.selected_note) {
            // First, build the note name map and update links
            let (note_name_map, file_path_string) = {
                let storage = self.storage.lock().unwrap();
                let map = self.build_note_name_map(&storage);
                let path = storage.folders.get(folder_idx)
                    .and_then(|f| f.notes.get(note_idx))
                    .map(|n| n.file_path.clone())
                    .unwrap_or_default();
                (map, path)
            };
            
            // Update the note content and links
            {
                let mut storage = self.storage.lock().unwrap();
                if let Some(folder) = storage.folders.get_mut(folder_idx) {
                    if let Some(note) = folder.notes.get_mut(note_idx) {
                        note.content = self.current_note_content.clone();
                        note.update_timestamp();
                        
                        // Update links after releasing the mutable borrow
                        self.link_manager.rebuild_links_for_note(
                            (folder_idx, note_idx),
                            &note.content,
                            &note_name_map,
                        );
                    }
                }
            }
            
            // Save to disk
            {
                let mut storage = self.storage.lock().unwrap();
                storage.save_note(folder_idx, note_idx).ok();
            }
            
            // Commit to version control
            if let Some(ref vc) = self.version_control {
                if !file_path_string.is_empty() {
                    let file_path = PathBuf::from(&file_path_string);
                    let storage = self.storage.lock().unwrap();
                    let title = storage.folders.get(folder_idx)
                        .and_then(|f| f.notes.get(note_idx))
                        .map(|n| n.title.clone())
                        .unwrap_or_default();
                    drop(storage);
                    vc.commit_note(&file_path, &format!("Updated: {}", title)).ok();
                }
            }
            
            self.last_save_time = std::time::Instant::now();
            println!("✓ Note saved and versioned");
        }
    }
    
    fn build_note_name_map(&self, storage: &Storage) -> std::collections::HashMap<String, (usize, usize)> {
        let mut map = std::collections::HashMap::new();
        for (folder_idx, folder) in storage.folders.iter().enumerate() {
            for (note_idx, note) in folder.notes.iter().enumerate() {
                map.insert(note.title.clone(), (folder_idx, note_idx));
            }
        }
        map
    }
    
    fn perform_search(&mut self) {
        self.search_results.clear();
        if self.search_query.is_empty() {
            return;
        }
        
        let storage = self.storage.lock().unwrap();
        self.search_results = self.search.search(&storage.folders, &self.search_query);
    }
    
    fn create_folder(&mut self) {
        if !self.new_folder_name.is_empty() {
            let mut storage = self.storage.lock().unwrap();
            storage.create_folder(&self.new_folder_name).ok();
            self.new_folder_name.clear();
            self.show_new_folder_dialog = false;
        }
    }
    
    fn create_note(&mut self) {
        if let Some(folder_idx) = self.selected_folder {
            if !self.new_note_title.is_empty() {
                let mut storage = self.storage.lock().unwrap();
                match storage.create_note(folder_idx, &self.new_note_title) {
                    Ok(note_idx) => {
                        println!("✓ Note created: {} in folder {}", self.new_note_title, folder_idx);
                        
                        // Auto-select and open the newly created note
                        self.selected_note = Some(note_idx);
                        if let Some(folder) = storage.folders.get(folder_idx) {
                            if let Some(note) = folder.notes.get(note_idx) {
                                self.current_note_content = note.content.clone();
                            }
                        }
                        
                        self.new_note_title.clear();
                        self.show_new_note_dialog = false;
                    }
                    Err(e) => {
                        eprintln!("✗ Failed to create note: {}", e);
                    }
                }
            } else {
                println!("⚠ Note title is empty");
            }
        } else {
            println!("⚠ No folder selected");
        }
    }
    
    fn sync_to_cloud(&mut self) {
        let storage = self.storage.lock().unwrap();
        match storage.export_to_cloud() {
            Ok(path) => {
                println!("Synced to: {}", path);
            }
            Err(e) => {
                eprintln!("Sync failed: {}", e);
            }
        }
    }
    
    // Theme management
    fn apply_theme(&mut self, ctx: &egui::Context) {
        self.theme_manager.current_theme.apply_to_egui(ctx);
    }
    
    fn toggle_dark_mode(&mut self) {
        self.theme_manager.toggle_dark_mode();
    }
    
    // Tag management
    fn add_tag(&mut self) {
        if !self.new_tag_name.is_empty() {
            self.tag_manager.add_tag(self.new_tag_name.clone());
            self.new_tag_name.clear();
        }
    }
    
    fn assign_tag_to_note(&mut self, tag_idx: usize) {
        if let (Some(folder_idx), Some(note_idx)) = (self.selected_folder, self.selected_note) {
            let mut storage = self.storage.lock().unwrap();
            if let Some(folder) = storage.folders.get_mut(folder_idx) {
                if let Some(note) = folder.notes.get_mut(note_idx) {
                    note.add_tag(tag_idx);
                    storage.save_note(folder_idx, note_idx).ok();
                }
            }
        }
    }
    
    // Encryption
    fn toggle_encryption(&mut self) {
        if self.encryption_password != self.confirm_password {
            eprintln!("✗ Passwords don't match!");
            return;
        }
        
        if let (Some(folder_idx), Some(note_idx)) = (self.selected_folder, self.selected_note) {
            let mut storage = self.storage.lock().unwrap();
            if let Some(folder) = storage.folders.get_mut(folder_idx) {
                if let Some(note) = folder.notes.get_mut(note_idx) {
                    if note.is_encrypted {
                        // Decrypt
                        if let Some(ref encrypted_data) = note.encrypted_data {
                            match self.encryption.decrypt(encrypted_data, &self.encryption_password) {
                                Ok(decrypted) => {
                                    note.content = decrypted;
                                    note.is_encrypted = false;
                                    note.encrypted_data = None;
                                    self.current_note_content = note.content.clone();
                                    println!("✓ Note decrypted");
                                }
                                Err(e) => eprintln!("✗ Decryption failed: {}", e),
                            }
                        }
                    } else {
                        // Encrypt
                        match self.encryption.encrypt(&note.content, &self.encryption_password) {
                            Ok(encrypted_data) => {
                                note.encrypted_data = Some(encrypted_data);
                                note.is_encrypted = true;
                                note.content = "[ENCRYPTED]".to_string();
                                self.current_note_content = "[ENCRYPTED - Enter password to decrypt]".to_string();
                                println!("✓ Note encrypted");
                            }
                            Err(e) => eprintln!("✗ Encryption failed: {}", e),
                        }
                    }
                    
                    storage.save_note(folder_idx, note_idx).ok();
                }
            }
        }
        
        self.encryption_password.clear();
        self.confirm_password.clear();
        self.show_encryption_dialog = false;
    }
    
    // Export
    fn export_note_to_pdf(&self) {
        if let (Some(folder_idx), Some(note_idx)) = (self.selected_folder, self.selected_note) {
            let storage = self.storage.lock().unwrap();
            if let Some(folder) = storage.folders.get(folder_idx) {
                if let Some(note) = folder.notes.get(note_idx) {
                    let output_path = PathBuf::from(format!("{}.pdf", note.title));
                    match pdf_export::PdfExporter::export_note(&note.title, &note.content, &output_path) {
                        Ok(_) => println!("✓ Exported to PDF: {:?}", output_path),
                        Err(e) => eprintln!("✗ PDF export failed: {}", e),
                    }
                }
            }
        }
    }
    
    fn export_folder_to_pdf(&self) {
        if let Some(folder_idx) = self.selected_folder {
            let storage = self.storage.lock().unwrap();
            if let Some(folder) = storage.folders.get(folder_idx) {
                let notes: Vec<(String, String)> = folder.notes.iter()
                    .map(|n| (n.title.clone(), n.content.clone()))
                    .collect();
                
                let output_path = PathBuf::from(format!("{}_folder.pdf", folder.name));
                match pdf_export::PdfExporter::export_multiple_notes(&notes, &output_path) {
                    Ok(_) => println!("✓ Exported folder to PDF: {:?}", output_path),
                    Err(e) => eprintln!("✗ PDF export failed: {}", e),
                }
            }
        }
    }
    
    // Version history
    fn load_version_history(&mut self) {
        if let (Some(folder_idx), Some(note_idx)) = (self.selected_folder, self.selected_note) {
            let storage = self.storage.lock().unwrap();
            if let Some(folder) = storage.folders.get(folder_idx) {
                if let Some(note) = folder.notes.get(note_idx) {
                    if let Some(ref vc) = self.version_control {
                        let file_path = PathBuf::from(&note.file_path);
                        match vc.get_file_history(&file_path) {
                            Ok(versions) => {
                                self.note_versions = versions;
                                println!("✓ Loaded {} versions", self.note_versions.len());
                            }
                            Err(e) => eprintln!("✗ Failed to load versions: {}", e),
                        }
                    }
                }
            }
        }
    }
    
    fn restore_version(&mut self, version_idx: usize) {
        if let (Some(folder_idx), Some(note_idx)) = (self.selected_folder, self.selected_note) {
            if let Some(version) = self.note_versions.get(version_idx) {
                let storage = self.storage.lock().unwrap();
                if let Some(folder) = storage.folders.get(folder_idx) {
                    if let Some(note) = folder.notes.get(note_idx) {
                        if let Some(ref vc) = self.version_control {
                            let file_path = PathBuf::from(&note.file_path);
                            match vc.restore_version(&file_path, &version.commit_id) {
                                Ok(content) => {
                                    self.current_note_content = content;
                                    println!("✓ Restored version from {}", version.timestamp);
                                }
                                Err(e) => eprintln!("✗ Failed to restore: {}", e),
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Favorites
    fn toggle_favorite(&mut self) {
        if let (Some(folder_idx), Some(note_idx)) = (self.selected_folder, self.selected_note) {
            let note_id = (folder_idx, note_idx);
            if let Some(pos) = self.favorite_notes.iter().position(|&id| id == note_id) {
                self.favorite_notes.remove(pos);
                println!("✓ Removed from favorites");
            } else {
                self.favorite_notes.push(note_id);
                println!("✓ Added to favorites");
            }
        }
    }
    
    fn is_favorite(&self) -> bool {
        if let (Some(folder_idx), Some(note_idx)) = (self.selected_folder, self.selected_note) {
            self.favorite_notes.contains(&(folder_idx, note_idx))
        } else {
            false
        }
    }
    
    // Statistics
    fn calculate_statistics(&self) -> NoteStatistics {
        let storage = self.storage.lock().unwrap();
        let total_notes: usize = storage.folders.iter().map(|f| f.notes.len()).sum();
        let total_folders = storage.folders.len();
        
        let total_words: usize = storage.folders.iter()
            .flat_map(|f| &f.notes)
            .map(|n| n.content.split_whitespace().count())
            .sum();
        
        let total_chars: usize = storage.folders.iter()
            .flat_map(|f| &f.notes)
            .map(|n| n.content.len())
            .sum();
        
        let encrypted_count: usize = storage.folders.iter()
            .flat_map(|f| &f.notes)
            .filter(|n| n.is_encrypted)
            .count();
        
        let total_tags = self.tag_manager.all_tags().len();
        
        NoteStatistics {
            total_notes,
            total_folders,
            total_words,
            total_chars,
            encrypted_count,
            total_tags,
            favorite_count: self.favorite_notes.len(),
        }
    }
    
    // Auto-save
    fn check_auto_save(&mut self) {
        if self.auto_save_enabled && !self.show_markdown_preview {
            // Only auto-save if we have a note selected and we're in edit mode (not preview)
            if self.selected_folder.is_some() && self.selected_note.is_some() {
                let elapsed = self.last_save_time.elapsed().as_secs_f32();
                if elapsed >= self.auto_save_interval {
                    self.save_current_note();
                    println!("✓ Auto-saved");
                }
            }
        }
    }
}

impl eframe::App for NoteTakingApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme every frame
        self.apply_theme(ctx);
        
        // Check auto-save
        self.check_auto_save();
        
        // Keyboard shortcuts
        ctx.input(|i| {
            // Ctrl/Cmd + S to save
            if i.modifiers.command && i.key_pressed(egui::Key::S) {
                self.save_current_note();
            }
            
            // Ctrl/Cmd + P to toggle preview
            if i.modifiers.command && i.key_pressed(egui::Key::P) {
                if self.selected_note.is_some() {
                    self.show_markdown_preview = !self.show_markdown_preview;
                }
            }
            
            // Ctrl/Cmd + N for new note
            if i.modifiers.command && i.key_pressed(egui::Key::N) {
                if self.selected_folder.is_some() {
                    self.show_new_note_dialog = true;
                }
            }
            
            // Ctrl/Cmd + F for search (focus search bar)
            if i.modifiers.command && i.key_pressed(egui::Key::F) {
                // Search bar will be auto-focused
            }
            
            // Ctrl/Cmd + E to encrypt/decrypt
            if i.modifiers.command && i.key_pressed(egui::Key::E) {
                if self.selected_note.is_some() {
                    self.show_encryption_dialog = true;
                }
            }
        });
        
        // Minimalist top panel
        egui::TopBottomPanel::top("top_panel")
            .frame(egui::Frame::none().inner_margin(egui::Margin::symmetric(12.0, 8.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Sidebar toggle
                    if ui.small_button(if self.sidebar_open { "◀" } else { "▶" }).clicked() {
                        self.sidebar_open = !self.sidebar_open;
                    }
                    
                    ui.add_space(8.0);
                    
                    // Clean search bar
                    let search_response = ui.add(
                        egui::TextEdit::singleline(&mut self.search_query)
                            .hint_text("Search...")
                            .desired_width(200.0)
                    );
                    if search_response.changed() {
                        self.perform_search();
                    }
                    
                    ui.add_space(8.0);
                    
                    // Minimalist menu buttons
                    let note_button_enabled = self.selected_folder.is_some();
                    ui.add_enabled_ui(note_button_enabled, |ui| {
                        if ui.small_button("+ Note").clicked() {
                            self.show_new_note_dialog = true;
                        }
                    });
                    if !note_button_enabled {
                        ui.label(egui::RichText::new("(select folder first)").small().weak());
                    }
                    
                    if ui.small_button("+ Folder").clicked() {
                        self.show_new_folder_dialog = true;
                    }
                    
                    ui.separator();
                    
                    // Compact menus
                    ui.menu_button("View", |ui| {
                        if ui.button(if self.show_markdown_preview { "Edit" } else { "Preview" }).clicked() {
                            if self.selected_note.is_some() {
                                self.show_markdown_preview = !self.show_markdown_preview;
                            }
                            ui.close_menu();
                        }
                        if ui.button("Statistics").clicked() {
                            self.show_statistics = !self.show_statistics;
                            ui.close_menu();
                        }
                    });
                    
                    ui.menu_button("Tools", |ui| {
                        if ui.button("Tags").clicked() {
                            self.show_tag_dialog = true;
                            ui.close_menu();
                        }
                        if ui.button("Export").clicked() {
                            self.show_export_dialog = true;
                            ui.close_menu();
                        }
                        if ui.button("Versions").clicked() {
                            self.load_version_history();
                            self.show_version_history = true;
                            ui.close_menu();
                        }
                        if ui.button("Encrypt").clicked() {
                            self.show_encryption_dialog = true;
                            ui.close_menu();
                        }
                    });
                    
                    ui.menu_button("⚙", |ui| {
                        if ui.button("Theme").clicked() {
                            self.show_theme_dialog = true;
                            ui.close_menu();
                        }
                        if ui.button(if self.theme_manager.current_theme.is_dark { "Light Mode" } else { "Dark Mode" }).clicked() {
                            self.toggle_dark_mode();
                            ui.close_menu();
                        }
                        ui.separator();
                        ui.checkbox(&mut self.auto_save_enabled, "Auto-save");
                        ui.checkbox(&mut self.spellcheck_enabled, "Spell Check");
                    });
                    
                    // Right-aligned current note with save button
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if self.selected_note.is_some() {
                            if ui.small_button("💾 Save").clicked() {
                                self.save_current_note();
                            }
                            
                            if let (Some(folder_idx), Some(note_idx)) = (self.selected_folder, self.selected_note) {
                                let storage = self.storage.lock().unwrap();
                                if let Some(folder) = storage.folders.get(folder_idx) {
                                    if let Some(note) = folder.notes.get(note_idx) {
                                        ui.add_space(8.0);
                                        let mut title_text = note.title.clone();
                                        if note.is_encrypted {
                                            title_text = format!("🔒 {}", title_text);
                                        }
                                        if self.is_favorite() {
                                            title_text = format!("⭐ {}", title_text);
                                        }
                                        ui.label(title_text);
                                    }
                                }
                            }
                        }
                    });
                });
            });
        
        // Rest of the UI remains the same but continues in sidebar and central panel
        if self.sidebar_open {
            self.render_sidebar(ctx);
        }
        
        self.render_central_panel(ctx);
        self.render_all_dialogs(ctx);
    }
}

impl NoteTakingApp {
    // Render helper methods
    fn render_sidebar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("sidebar")
            .resizable(true)
            .default_width(220.0)
            .frame(egui::Frame::none()
                .inner_margin(egui::Margin::same(12.0))
                .fill(egui::Color32::from_gray(25))) // Slightly lighter dark gray
            .show(ctx, |ui| {
                // Set text color to be clearly visible
                ui.style_mut().visuals.override_text_color = Some(egui::Color32::from_gray(220));
                
                ui.heading("Notes");
                ui.add_space(8.0);
                
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Show search results if searching
                    if !self.search_query.is_empty() && !self.search_results.is_empty() {
                        ui.label(egui::RichText::new("Search Results").strong().color(egui::Color32::from_gray(200)));
                        ui.add_space(4.0);
                        
                        let search_display: Vec<_> = {
                            let storage = self.storage.lock().unwrap();
                            self.search_results.iter().filter_map(|(folder_idx, note_idx)| {
                                storage.folders.get(*folder_idx).and_then(|folder| {
                                    folder.notes.get(*note_idx).map(|note| {
                                        (
                                            *folder_idx,
                                            *note_idx,
                                            note.title.clone(),
                                            folder.name.clone(),
                                            note.content.clone()
                                        )
                                    })
                                })
                            }).collect()
                        };
                        
                        for (folder_idx, note_idx, title, folder_name, content) in search_display {
                            let is_selected = self.selected_folder == Some(folder_idx) && 
                                             self.selected_note == Some(note_idx);
                            
                            if ui.selectable_label(is_selected, 
                                egui::RichText::new(&title).color(egui::Color32::from_gray(220))
                            ).clicked() {
                                self.save_current_note();
                                self.selected_folder = Some(folder_idx);
                                self.selected_note = Some(note_idx);
                                self.current_note_content = content.clone();
                                
                                if self.spellcheck_enabled {
                                    self.misspelled_words = self.spellcheck.check_text(&content);
                                }
                            }
                            ui.label(egui::RichText::new(&folder_name).small().color(egui::Color32::from_gray(150)));
                            ui.add_space(4.0);
                        }
                        
                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("All Notes").strong().color(egui::Color32::from_gray(200)));
                        ui.add_space(4.0);
                    }
                    
                    // Show folder tree
                    self.render_folder_tree(ui);
                });
            });
    }
    
    fn render_folder_tree(&mut self, ui: &mut egui::Ui) {
        let folders_display: Vec<_> = {
            let storage = self.storage.lock().unwrap();
            storage.folders.iter().enumerate().map(|(folder_idx, folder)| {
                let notes: Vec<_> = folder.notes.iter().enumerate()
                    .map(|(note_idx, note)| {
                        (note_idx, note.title.clone(), note.content.clone(), note.is_encrypted)
                    })
                    .collect();
                (folder_idx, folder.name.clone(), notes)
            }).collect()
        };
        
        for (folder_idx, folder_name, notes) in folders_display {
            let is_selected = self.selected_folder == Some(folder_idx);
            
            let header_response = ui.collapsing(
                egui::RichText::new(&folder_name).strong().color(egui::Color32::from_gray(230)),
                |ui| {
                    if notes.is_empty() {
                        ui.label(egui::RichText::new("No notes").color(egui::Color32::from_gray(140)).small());
                    }
                    for (note_idx, title, content, is_encrypted) in notes {
                        let mut label_text = title.clone();
                        if is_encrypted {
                            label_text = format!("🔒 {}", label_text);
                        }
                        
                        let is_note_selected = self.selected_folder == Some(folder_idx) && 
                                              self.selected_note == Some(note_idx);
                        
                        // Create colored text for better visibility
                        let note_label = egui::RichText::new(&label_text)
                            .color(egui::Color32::from_gray(210));
                        
                        if ui.selectable_label(is_note_selected, note_label).clicked() {
                            self.save_current_note();
                            self.selected_folder = Some(folder_idx);
                            self.selected_note = Some(note_idx);
                            self.current_note_content = content.clone();
                            
                            if self.spellcheck_enabled {
                                self.misspelled_words = self.spellcheck.check_text(&content);
                            }
                        }
                    }
                }
            );
            
            if header_response.header_response.clicked() {
                self.selected_folder = Some(folder_idx);
                self.selected_note = None;
                self.current_note_content.clear();
            }
            
            ui.add_space(4.0);
        }
    }
    
    fn render_central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if let (Some(folder_idx), Some(note_idx)) = (self.selected_folder, self.selected_note) {
                self.render_note_editor(ui, folder_idx, note_idx);
            } else {
                self.render_welcome_screen(ui);
            }
        });
    }
    
    fn render_note_editor(&mut self, ui: &mut egui::Ui, folder_idx: usize, note_idx: usize) {
        let note_data = {
            let storage = self.storage.lock().unwrap();
            storage.folders.get(folder_idx).and_then(|folder| {
                folder.notes.get(note_idx).map(|note| {
                    (note.title.clone(), note.created_at.clone(), note.updated_at.clone(), note.is_encrypted)
                })
            })
        };
        
        if let Some((title, created_at, updated_at, is_encrypted)) = note_data {
            // Minimal header - just title
            ui.add_space(8.0);
            
            // Main editor area - seamlessly editable or preview
            egui::ScrollArea::vertical().show(ui, |ui| {
                if self.show_markdown_preview {
                    // Clean preview mode
                    egui_commonmark::CommonMarkViewer::new()
                        .show(ui, &mut egui_commonmark::CommonMarkCache::default(), &self.current_note_content);
                } else {
                    // Clean edit mode
                    let text_edit = egui::TextEdit::multiline(&mut self.current_note_content)
                        .desired_width(f32::INFINITY)
                        .desired_rows(35)
                        .font(egui::TextStyle::Monospace);
                    
                    let response = ui.add(text_edit);
                    
                    // Check for content changes to trigger spell check
                    if response.changed() && self.spellcheck_enabled {
                        self.misspelled_words = self.spellcheck.check_text(&self.current_note_content);
                    }
                    
                    // Minimal spell check indicator at bottom
                    if self.spellcheck_enabled && !self.misspelled_words.is_empty() {
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(format!("⚠ {} spelling issue{}", 
                                self.misspelled_words.len(),
                                if self.misspelled_words.len() == 1 { "" } else { "s" }
                            )).small().weak());
                            
                            if ui.small_button("View").clicked() {
                                // Will expand below
                            }
                        });
                    }
                }
            });
        }
    }
    
    fn render_welcome_screen(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(150.0);
            
            ui.heading("Notes");
            ui.add_space(20.0);
            
            ui.label("Create a folder to get started");
            ui.add_space(10.0);
            
            ui.label(egui::RichText::new("Click '+ Folder' above").small().weak());
        });
    }
    
    fn render_all_dialogs(&mut self, ctx: &egui::Context) {
        self.render_new_folder_dialog(ctx);
        self.render_new_note_dialog(ctx);
        self.render_theme_dialog(ctx);
        self.render_tag_dialog(ctx);
        self.render_encryption_dialog(ctx);
        self.render_export_dialog(ctx);
        self.render_statistics_dialog(ctx);
        self.render_version_history_dialog(ctx);
    }
    
    fn render_new_folder_dialog(&mut self, ctx: &egui::Context) {
        if self.show_new_folder_dialog {
            egui::Window::new("New Folder")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Folder name:");
                    let response = ui.text_edit_singleline(&mut self.new_folder_name);
                    
                    // Auto-focus the text field
                    response.request_focus();
                    
                    // Support Enter key to create
                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) && !self.new_folder_name.is_empty() {
                        self.create_folder();
                    }
                    
                    ui.add_space(5.0);
                    
                    ui.horizontal(|ui| {
                        if ui.button("Create").clicked() {
                            self.create_folder();
                        }
                        if ui.button("Cancel").clicked() {
                            self.new_folder_name.clear();
                            self.show_new_folder_dialog = false;
                        }
                    });
                });
        }
    }
    
    fn render_new_note_dialog(&mut self, ctx: &egui::Context) {
        if self.show_new_note_dialog {
            egui::Window::new("New Note")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Note title:");
                    let response = ui.text_edit_singleline(&mut self.new_note_title);
                    
                    // Auto-focus the text field
                    response.request_focus();
                    
                    // Support Enter key to create
                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) && !self.new_note_title.is_empty() {
                        self.create_note();
                    }
                    
                    ui.add_space(5.0);
                    
                    ui.horizontal(|ui| {
                        if ui.button("Create").clicked() {
                            self.create_note();
                        }
                        if ui.button("Cancel").clicked() {
                            self.new_note_title.clear();
                            self.show_new_note_dialog = false;
                        }
                    });
                    
                    // Show selected folder
                    if let Some(folder_idx) = self.selected_folder {
                        let storage = self.storage.lock().unwrap();
                        if let Some(folder) = storage.folders.get(folder_idx) {
                            ui.add_space(5.0);
                            ui.label(egui::RichText::new(format!("In folder: {}", folder.name)).small().weak());
                        }
                    } else {
                        ui.add_space(5.0);
                        ui.label(egui::RichText::new("⚠ Please select a folder first").small().color(egui::Color32::from_rgb(255, 200, 100)));
                    }
                });
        }
    }
    
    fn render_theme_dialog(&mut self, ctx: &egui::Context) {
        if self.show_theme_dialog {
            egui::Window::new("Select Theme")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(egui::RichText::new(format!("Current: {}", self.theme_manager.current_theme.name)).weak());
                    ui.add_space(8.0);
                    
                    for theme in self.theme_manager.available_themes.clone() {
                        let is_current = theme.name == self.theme_manager.current_theme.name;
                        let button_text = if is_current {
                            format!("✓ {}", theme.name)
                        } else {
                            theme.name.clone()
                        };
                        
                        if ui.button(button_text).clicked() {
                            self.theme_manager.set_theme(theme);
                            self.show_theme_dialog = false;
                        }
                    }
                    
                    ui.add_space(8.0);
                    if ui.button("Close").clicked() {
                        self.show_theme_dialog = false;
                    }
                });
        }
    }
    
    fn render_tag_dialog(&mut self, ctx: &egui::Context) {
        if self.show_tag_dialog {
            egui::Window::new("🏷 Tag Manager")
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.new_tag_name);
                        if ui.button("Add Tag").clicked() {
                            self.add_tag();
                        }
                    });
                    ui.separator();
                    
                    // Collect tags first to avoid borrow conflicts
                    let tags: Vec<_> = self.tag_manager.all_tags().iter()
                        .map(|t| (t.name.clone(), t.color))
                        .collect();
                    
                    for (idx, (name, color)) in tags.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.colored_label(
                                egui::Color32::from_rgb(color[0], color[1], color[2]),
                                name
                            );
                            if ui.button("Assign").clicked() {
                                self.assign_tag_to_note(idx);
                            }
                        });
                    }
                    
                    if ui.button("Close").clicked() {
                        self.show_tag_dialog = false;
                    }
                });
        }
    }
    
    fn render_encryption_dialog(&mut self, ctx: &egui::Context) {
        if self.show_encryption_dialog {
            egui::Window::new("🔐 Encrypt/Decrypt Note")
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label("Password:");
                    ui.add(egui::TextEdit::singleline(&mut self.encryption_password).password(true));
                    ui.label("Confirm:");
                    ui.add(egui::TextEdit::singleline(&mut self.confirm_password).password(true));
                    
                    ui.horizontal(|ui| {
                        if ui.button("Apply").clicked() {
                            self.toggle_encryption();
                        }
                        if ui.button("Cancel").clicked() {
                            self.encryption_password.clear();
                            self.confirm_password.clear();
                            self.show_encryption_dialog = false;
                        }
                    });
                });
        }
    }
    
    fn render_export_dialog(&mut self, ctx: &egui::Context) {
        if self.show_export_dialog {
            egui::Window::new("📄 Export")
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label("Export format:");
                    ui.radio_value(&mut self.export_format, ExportFormat::PDF, "PDF");
                    ui.radio_value(&mut self.export_format, ExportFormat::Markdown, "Markdown");
                    ui.radio_value(&mut self.export_format, ExportFormat::PlainText, "Plain Text");
                    
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        if ui.button("Export Note").clicked() {
                            self.export_note_to_pdf();
                            self.show_export_dialog = false;
                        }
                        if ui.button("Export Folder").clicked() {
                            self.export_folder_to_pdf();
                            self.show_export_dialog = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_export_dialog = false;
                        }
                    });
                });
        }
    }
    
    fn render_statistics_dialog(&mut self, ctx: &egui::Context) {
        if self.show_statistics {
            let stats = self.calculate_statistics();
            egui::Window::new("📊 Statistics")
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label(format!("📁 Total Folders: {}", stats.total_folders));
                    ui.label(format!("📝 Total Notes: {}", stats.total_notes));
                    ui.label(format!("📝 Total Words: {}", stats.total_words));
                    ui.label(format!("📝 Total Characters: {}", stats.total_chars));
                    ui.label(format!("🔒 Encrypted Notes: {}", stats.encrypted_count));
                    ui.label(format!("🏷 Total Tags: {}", stats.total_tags));
                    ui.label(format!("⭐ Favorites: {}", stats.favorite_count));
                    
                    if ui.button("Close").clicked() {
                        self.show_statistics = false;
                    }
                });
        }
    }
    
    fn render_version_history_dialog(&mut self, ctx: &egui::Context) {
        if self.show_version_history {
            egui::Window::new("📚 Version History - Timeline")
                .collapsible(false)
                .resizable(true)
                .default_width(600.0)
                .show(ctx, |ui| {
                    if self.note_versions.is_empty() {
                        ui.label("No version history available for this note.");
                        if ui.button("Close").clicked() {
                            self.show_version_history = false;
                        }
                        return;
                    }
                    
                    ui.heading("🕐 Version Timeline");
                    ui.label("Drag the slider to navigate through versions");
                    ui.separator();
                    
                    // Timeline slider
                    ui.horizontal(|ui| {
                        ui.label("Oldest");
                        
                        let slider_response = ui.add(
                            egui::Slider::new(&mut self.version_timeline_position, 0.0..=1.0)
                                .show_value(false)
                                .text("")
                        );
                        
                        ui.label("Newest");
                        
                        // Calculate which version we're at based on slider
                        let version_count = self.note_versions.len();
                        let version_index = ((1.0 - self.version_timeline_position) * (version_count - 1) as f32).round() as usize;
                        let version_index = version_index.min(version_count.saturating_sub(1));
                        
                        // If slider changed, update selected version
                        if slider_response.changed() {
                            self.selected_version = Some(version_index);
                        }
                    });
                    
                    ui.separator();
                    
                    // Show current version info
                    let version_count = self.note_versions.len();
                    let current_index = ((1.0 - self.version_timeline_position) * (version_count - 1) as f32).round() as usize;
                    let current_index = current_index.min(version_count.saturating_sub(1));
                    
                    if let Some(version) = self.note_versions.get(current_index) {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label("📅 Date:");
                                ui.strong(&version.timestamp);
                            });
                            ui.horizontal(|ui| {
                                ui.label("💬 Message:");
                                ui.label(&version.message);
                            });
                            ui.horizontal(|ui| {
                                ui.label("👤 Author:");
                                ui.label(&version.author);
                            });
                            ui.horizontal(|ui| {
                                ui.label("📍 Position:");
                                ui.label(format!("{} of {} versions", current_index + 1, version_count));
                            });
                        });
                        
                        ui.add_space(10.0);
                        
                        // Action buttons
                        ui.horizontal(|ui| {
                            if ui.button("👁 Preview This Version").clicked() {
                                // Preview the version content (would need to implement restore_version to return content)
                                self.selected_version = Some(current_index);
                            }
                            
                            if ui.button("↩ Restore This Version").clicked() {
                                self.restore_version(current_index);
                                self.show_version_history = false;
                            }
                            
                            if ui.button("❌ Close").clicked() {
                                self.show_version_history = false;
                            }
                        });
                    }
                    
                    ui.separator();
                    
                    // Version list (for reference)
                    ui.label("📋 All Versions:");
                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            let versions = self.note_versions.clone();
                            
                            for (idx, version) in versions.iter().enumerate().rev() {
                                let is_current = idx == current_index;
                                let bg_color = if is_current {
                                    egui::Color32::from_rgb(100, 150, 255)
                                } else {
                                    egui::Color32::from_gray(60)
                                };
                                
                                ui.horizontal(|ui| {
                                    if is_current {
                                        ui.label("👉");
                                    } else {
                                        ui.label("  ");
                                    }
                                    
                                    egui::Frame::none()
                                        .fill(bg_color)
                                        .inner_margin(4.0)
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.label(&version.timestamp);
                                                ui.separator();
                                                ui.label(&version.message);
                                                
                                                if ui.small_button("↩").clicked() {
                                                    self.version_timeline_position = 1.0 - (idx as f32 / (version_count - 1) as f32);
                                                    self.selected_version = Some(idx);
                                                }
                                            });
                                        });
                                });
                                ui.add_space(2.0);
                            }
                        });
                });
        }
    }
}
