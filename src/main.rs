use eframe::egui;
use std::sync::Arc;
use std::sync::Mutex;

mod encryption;
mod images;
mod links;
mod note;
mod pdf_export;
mod search;
mod storage;
mod tags;
mod theme;
mod version_control;

use encryption::Encryption;
use links::LinkManager;
use search::FuzzySearch;
use std::path::PathBuf;
use storage::Storage;
use tags::TagManager;
use theme::ThemeManager;
use version_control::VersionControl;

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

    // Images
    show_image_dialog: bool,

    // Statistics
    show_statistics: bool,

    // Settings
    show_settings: bool,
    auto_save_enabled: bool,
    auto_save_interval: f32,
    last_save_time: std::time::Instant,

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
            show_image_dialog: false,
            show_statistics: false,
            show_settings: false,
            auto_save_enabled: true,
            auto_save_interval: 30.0,
            last_save_time: std::time::Instant::now(),
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
                let path = storage
                    .folders
                    .get(folder_idx)
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
                    let title = storage
                        .folders
                        .get(folder_idx)
                        .and_then(|f| f.notes.get(note_idx))
                        .map(|n| n.title.clone())
                        .unwrap_or_default();
                    drop(storage);
                    vc.commit_note(&file_path, &format!("Updated: {}", title))
                        .ok();
                }
            }

            self.last_save_time = std::time::Instant::now();
            println!("✓ Note saved and versioned");
        }
    }

    fn build_note_name_map(
        &self,
        storage: &Storage,
    ) -> std::collections::HashMap<String, (usize, usize)> {
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
                    Ok(_) => {
                        println!(
                            "✓ Note created: {} in folder {}",
                            self.new_note_title, folder_idx
                        );
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
                            match self
                                .encryption
                                .decrypt(encrypted_data, &self.encryption_password)
                            {
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
                        match self
                            .encryption
                            .encrypt(&note.content, &self.encryption_password)
                        {
                            Ok(encrypted_data) => {
                                note.encrypted_data = Some(encrypted_data);
                                note.is_encrypted = true;
                                note.content = "[ENCRYPTED]".to_string();
                                self.current_note_content =
                                    "[ENCRYPTED - Enter password to decrypt]".to_string();
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
                    match pdf_export::PdfExporter::export_note(
                        &note.title,
                        &note.content,
                        &output_path,
                    ) {
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
                let notes: Vec<(String, String)> = folder
                    .notes
                    .iter()
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

        let total_words: usize = storage
            .folders
            .iter()
            .flat_map(|f| &f.notes)
            .map(|n| n.content.split_whitespace().count())
            .sum();

        let total_chars: usize = storage
            .folders
            .iter()
            .flat_map(|f| &f.notes)
            .map(|n| n.content.len())
            .sum();

        let encrypted_count: usize = storage
            .folders
            .iter()
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

        // Top panel with all feature buttons
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // Sidebar toggle
                if ui
                    .button(if self.sidebar_open { "◀" } else { "▶" })
                    .clicked()
                {
                    self.sidebar_open = !self.sidebar_open;
                }

                ui.separator();

                // Search
                ui.label("🔍");
                let search_response = ui.add(
                    egui::TextEdit::singleline(&mut self.search_query).hint_text("Search notes..."),
                );
                if search_response.changed() {
                    self.perform_search();
                }

                ui.separator();

                // File menu
                ui.menu_button("📁 File", |ui| {
                    if ui.button("📁 New Folder").clicked() {
                        self.show_new_folder_dialog = true;
                        ui.close_menu();
                    }
                    if ui.button("📝 New Note (Ctrl+N)").clicked() && self.selected_folder.is_some()
                    {
                        self.show_new_note_dialog = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("💾 Save (Ctrl+S)").clicked() {
                        self.save_current_note();
                        ui.close_menu();
                    }
                    if ui.button("☁ Sync to Cloud").clicked() {
                        self.sync_to_cloud();
                        ui.close_menu();
                    }
                });

                // Edit menu
                ui.menu_button("✏ Edit", |ui| {
                    if ui.button("🔗 Insert Link").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("🖼 Insert Image").clicked() {
                        self.show_image_dialog = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui
                        .button(if self.is_favorite() {
                            "⭐ Unfavorite"
                        } else {
                            "☆ Favorite"
                        })
                        .clicked()
                    {
                        self.toggle_favorite();
                        ui.close_menu();
                    }
                });

                // View menu
                ui.menu_button("👁 View", |ui| {
                    if ui
                        .button(if self.show_markdown_preview {
                            "📝 Edit Mode (Ctrl+P)"
                        } else {
                            "👁 Preview Mode (Ctrl+P)"
                        })
                        .clicked()
                    {
                        if self.selected_note.is_some() {
                            self.show_markdown_preview = !self.show_markdown_preview;
                        }
                        ui.close_menu();
                    }
                    if ui.button("🔗 Links Panel").clicked() {
                        self.show_links_panel = !self.show_links_panel;
                        ui.close_menu();
                    }
                    if ui.button("⭐ Favorites").clicked() {
                        self.show_favorites = !self.show_favorites;
                        ui.close_menu();
                    }
                    if ui.button("📊 Statistics").clicked() {
                        self.show_statistics = !self.show_statistics;
                        ui.close_menu();
                    }
                });

                // Tools menu
                ui.menu_button("🛠 Tools", |ui| {
                    if ui.button("🏷 Tag Manager").clicked() {
                        self.show_tag_dialog = true;
                        ui.close_menu();
                    }
                    if ui.button("🔐 Encrypt/Decrypt (Ctrl+E)").clicked() {
                        self.show_encryption_dialog = true;
                        ui.close_menu();
                    }
                    if ui.button("📄 Export...").clicked() {
                        self.show_export_dialog = true;
                        ui.close_menu();
                    }
                    if ui.button("📚 Version History").clicked() {
                        self.load_version_history();
                        self.show_version_history = true;
                        ui.close_menu();
                    }
                });

                // Settings menu
                ui.menu_button("⚙ Settings", |ui| {
                    if ui.button("🎨 Theme").clicked() {
                        self.show_theme_dialog = true;
                        ui.close_menu();
                    }
                    if ui
                        .button(if self.theme_manager.current_theme.is_dark {
                            "☀ Light Mode"
                        } else {
                            "🌙 Dark Mode"
                        })
                        .clicked()
                    {
                        self.toggle_dark_mode();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui
                        .checkbox(&mut self.auto_save_enabled, "Auto-save")
                        .changed()
                    {
                        println!("Auto-save: {}", self.auto_save_enabled);
                    }
                });

                // Current note indicator
                if let (Some(folder_idx), Some(note_idx)) =
                    (self.selected_folder, self.selected_note)
                {
                    let storage = self.storage.lock().unwrap();
                    if let Some(folder) = storage.folders.get(folder_idx) {
                        if let Some(note) = folder.notes.get(note_idx) {
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if note.is_encrypted {
                                        ui.label("🔒");
                                    }
                                    if self.is_favorite() {
                                        ui.label("⭐");
                                    }
                                    ui.label(format!("📝 {}", note.title));
                                },
                            );
                        }
                    }
                }
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
            .default_width(250.0)
            .show(ctx, |ui| {
                ui.heading("📚 Library");
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.render_folder_tree(ui);
                });
            });
    }

    fn render_folder_tree(&mut self, ui: &mut egui::Ui) {
        let folders_display: Vec<_> = {
            let storage = self.storage.lock().unwrap();
            storage
                .folders
                .iter()
                .enumerate()
                .map(|(folder_idx, folder)| {
                    let notes: Vec<_> = folder
                        .notes
                        .iter()
                        .enumerate()
                        .map(|(note_idx, note)| {
                            (
                                note_idx,
                                note.title.clone(),
                                note.content.clone(),
                                note.is_encrypted,
                            )
                        })
                        .collect();
                    (folder_idx, folder.name.clone(), notes)
                })
                .collect()
        };

        for (folder_idx, folder_name, notes) in folders_display {
            let is_selected = self.selected_folder == Some(folder_idx);
            let folder_label = if is_selected {
                format!("📁 {} ✓", folder_name)
            } else {
                format!("📁 {}", folder_name)
            };

            let header_response = ui.collapsing(&folder_label, |ui| {
                if notes.is_empty() {
                    ui.label("(No notes yet)");
                }
                for (note_idx, title, content, is_encrypted) in notes {
                    let label = if is_encrypted {
                        format!("🔒 {}", title)
                    } else {
                        title.clone()
                    };

                    if ui
                        .selectable_label(
                            self.selected_folder == Some(folder_idx)
                                && self.selected_note == Some(note_idx),
                            label,
                        )
                        .clicked()
                    {
                        self.save_current_note();
                        self.selected_folder = Some(folder_idx);
                        self.selected_note = Some(note_idx);
                        self.current_note_content = content;
                    }
                }
            });

            if header_response.header_response.clicked() {
                self.selected_folder = Some(folder_idx);
                self.selected_note = None;
                self.current_note_content.clear();
            }
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
                    (
                        note.title.clone(),
                        note.created_at.clone(),
                        note.updated_at.clone(),
                        note.is_encrypted,
                    )
                })
            })
        };

        if let Some((title, created_at, updated_at, is_encrypted)) = note_data {
            // Header with title and controls
            ui.horizontal(|ui| {
                ui.heading(&title);

                ui.add_space(10.0);

                // Status indicators
                if is_encrypted {
                    ui.label("🔒");
                }
                if self.is_favorite() {
                    ui.label("⭐");
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Preview toggle button
                    if ui
                        .button(if self.show_markdown_preview {
                            "📝 Edit"
                        } else {
                            "👁 Preview"
                        })
                        .clicked()
                    {
                        self.show_markdown_preview = !self.show_markdown_preview;
                        if !self.show_markdown_preview {
                            // Switching back to edit mode, save first
                            self.save_current_note();
                        }
                    }

                    // Save button (always visible for manual saves)
                    if ui.button("💾 Save").clicked() {
                        self.save_current_note();
                    }

                    // Auto-save indicator
                    if self.auto_save_enabled {
                        let elapsed = self.last_save_time.elapsed().as_secs_f32();
                        let time_until_save = self.auto_save_interval - elapsed;
                        if time_until_save > 0.0 {
                            ui.label(format!("⏱ Auto-save in {}s", time_until_save.ceil() as i32));
                        } else {
                            ui.label("✓ Auto-saved");
                        }
                    }
                });
            });

            ui.separator();

            // Metadata
            ui.horizontal(|ui| {
                ui.label(format!("Created: {}", created_at));
                ui.separator();
                ui.label(format!("Updated: {}", updated_at));
            });

            ui.separator();

            // Main editor area - seamlessly editable or preview
            egui::ScrollArea::vertical().show(ui, |ui| {
                if self.show_markdown_preview {
                    // Preview mode - read-only markdown rendering
                    ui.horizontal(|ui| {
                        ui.label("📖 Preview Mode");
                        ui.label("(Click 'Edit' to modify)");
                    });
                    ui.separator();

                    egui_commonmark::CommonMarkViewer::new().show(
                        ui,
                        &mut egui_commonmark::CommonMarkCache::default(),
                        &self.current_note_content,
                    );
                } else {
                    // Edit mode - always editable, auto-saves
                    ui.horizontal(|ui| {
                        ui.label("✏️ Edit Mode");
                        if self.auto_save_enabled {
                            ui.label("(Auto-saving every 30s)");
                        } else {
                            ui.label("(Remember to save manually)");
                        }
                    });
                    ui.separator();

                    let text_edit = egui::TextEdit::multiline(&mut self.current_note_content)
                        .desired_width(f32::INFINITY)
                        .desired_rows(30)
                        .font(egui::TextStyle::Monospace);

                    let response = ui.add(text_edit);

                    // Auto-focus on the editor when note is first selected
                    response.request_focus();

                    // Show helpful hints at the bottom
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("💡 Tips:");
                        ui.label("• Just start typing!");
                        ui.label("• Use Markdown: **bold**, *italic*, # headers");
                        ui.label("• Link notes: [[Note Name]]");
                        ui.label("• Toggle preview to see formatted text");
                    });
                }
            });
        }
    }

    fn render_welcome_screen(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);

            // Main heading
            ui.heading("📝 Enhanced Notetaking App");
            ui.add_space(20.0);

            // Quick start instructions
            ui.label("🚀 Quick Start:");
            ui.add_space(10.0);

            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 20.0;
                ui.label("1️⃣ Click 'File' → 'New Folder'");
                ui.label("2️⃣ Click the folder to select it");
                ui.label("3️⃣ Click 'File' → 'New Note'");
            });

            ui.add_space(20.0);
            ui.label("4️⃣ Click the note and start typing immediately!");

            ui.add_space(40.0);
            ui.separator();
            ui.add_space(20.0);

            // Feature highlights
            ui.label("✨ Key Features:");
            ui.add_space(10.0);

            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 15.0;
                ui.label("🎨 7 Themes");
                ui.label("🏷️ Smart Tags");
                ui.label("🔐 Encryption");
                ui.label("📄 PDF Export");
                ui.label("⭐ Favorites");
            });

            ui.add_space(10.0);

            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 15.0;
                ui.label("📚 Version History");
                ui.label("👁️ Markdown Preview");
                ui.label("💾 Auto-Save");
                ui.label("🔗 Note Linking");
                ui.label("📊 Statistics");
            });

            ui.add_space(40.0);
            ui.separator();
            ui.add_space(20.0);

            // Pro tips
            ui.label("💡 Pro Tips:");
            ui.add_space(10.0);

            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                ui.label("• No 'Edit' button needed - just click and type!");
                ui.label("• Auto-saves every 30 seconds");
                ui.label("• Use **bold** and *italic* in Markdown");
            });

            ui.add_space(10.0);

            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                ui.label("• Link notes with [[Note Name]]");
                ui.label("• Press '👁 Preview' to see formatted text");
                ui.label("• Change themes in Settings menu");
            });
        });
    }

    fn render_all_dialogs(&mut self, ctx: &egui::Context) {
        self.render_theme_dialog(ctx);
        self.render_tag_dialog(ctx);
        self.render_encryption_dialog(ctx);
        self.render_export_dialog(ctx);
        self.render_statistics_dialog(ctx);
        self.render_version_history_dialog(ctx);
    }

    fn render_theme_dialog(&mut self, ctx: &egui::Context) {
        if self.show_theme_dialog {
            egui::Window::new("🎨 Select Theme")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    for theme in self.theme_manager.available_themes.clone() {
                        if ui.button(&theme.name).clicked() {
                            self.theme_manager.set_theme(theme);
                            self.show_theme_dialog = false;
                        }
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
                    let tags: Vec<_> = self
                        .tag_manager
                        .all_tags()
                        .iter()
                        .map(|t| (t.name.clone(), t.color))
                        .collect();

                    for (idx, (name, color)) in tags.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.colored_label(
                                egui::Color32::from_rgb(color[0], color[1], color[2]),
                                name,
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
                    ui.add(
                        egui::TextEdit::singleline(&mut self.encryption_password).password(true),
                    );
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
                    ui.radio_value(
                        &mut self.export_format,
                        ExportFormat::PlainText,
                        "Plain Text",
                    );

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
            egui::Window::new("📚 Version History")
                .collapsible(false)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        // Clone versions to avoid borrow conflicts
                        let versions = self.note_versions.clone();

                        for (idx, version) in versions.iter().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(&version.timestamp);
                                ui.label(&version.message);
                                if ui.button("Restore").clicked() {
                                    self.restore_version(idx);
                                    self.show_version_history = false;
                                }
                            });
                            ui.separator();
                        }
                    });

                    if ui.button("Close").clicked() {
                        self.show_version_history = false;
                    }
                });
        }
    }
}
