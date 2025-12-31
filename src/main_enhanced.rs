use eframe::egui;
use std::sync::Arc;
use std::sync::Mutex;
use std::path::PathBuf;

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

use note::Folder;
use storage::Storage;
use search::FuzzySearch;
use theme::{Theme, ThemeManager};
use encryption::Encryption;
use tags::TagManager;
use links::LinkManager;
use version_control::VersionControl;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([1000.0, 700.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Notetaking App - Enhanced",
        options,
        Box::new(|cc| {
            Ok(Box::new(NoteTakingApp::new(cc)))
        }),
    )
}

struct NoteTakingApp {
    storage: Arc<Mutex<Storage>>,
    search: FuzzySearch,
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
    search_results: Vec<(usize, usize)>,
    
    // Folder management
    new_folder_name: String,
    show_new_folder_dialog: bool,
    
    // Note management
    new_note_title: String,
    show_new_note_dialog: bool,
    
    // UI flags
    is_editing: bool,
    sidebar_open: bool,
    show_markdown_preview: bool,
    
    // Theme settings
    show_theme_selector: bool,
    
    // Tag management
    show_tag_manager: bool,
    new_tag_name: String,
    selected_tag_filter: Option<usize>,
    
    // Encryption
    show_encryption_dialog: bool,
    encryption_password: String,
    confirm_password: String,
    
    // Export
    show_export_dialog: bool,
    export_format: ExportFormat,
    
    // Links and backlinks
    show_links_panel: bool,
    
    // Version history
    show_version_history: bool,
    note_versions: Vec<version_control::Version>,
    
    // Image embedding
    show_image_dialog: bool,
    image_path: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ExportFormat {
    PDF,
    Markdown,
    PlainText,
}

impl NoteTakingApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
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
        
        // Apply default theme
        theme_manager.current_theme.apply_to_egui(&cc.egui_ctx);
        
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
            is_editing: false,
            sidebar_open: true,
            show_markdown_preview: false,
            show_theme_selector: false,
            show_tag_manager: false,
            new_tag_name: String::new(),
            selected_tag_filter: None,
            show_encryption_dialog: false,
            encryption_password: String::new(),
            confirm_password: String::new(),
            show_export_dialog: false,
            export_format: ExportFormat::PDF,
            show_links_panel: false,
            show_version_history: false,
            note_versions: Vec::new(),
            show_image_dialog: false,
            image_path: String::new(),
        }
    }
    
    fn save_current_note(&mut self) {
        if let (Some(folder_idx), Some(note_idx)) = (self.selected_folder, self.selected_note) {
            let mut storage = self.storage.lock().unwrap();
            if let Some(folder) = storage.folders.get_mut(folder_idx) {
                if let Some(note) = folder.notes.get_mut(note_idx) {
                    note.content = self.current_note_content.clone();
                    note.update_timestamp();
                    
                    // Update links
                    let note_name_map = self.build_note_name_map(&storage);
                    self.link_manager.rebuild_links_for_note(
                        (folder_idx, note_idx),
                        &note.content,
                        &note_name_map,
                    );
                    
                    storage.save_note(folder_idx, note_idx).ok();
                    
                    // Commit to version control
                    if let Some(ref vc) = self.version_control {
                        let file_path = PathBuf::from(&note.file_path);
                        vc.commit_note(&file_path, &format!("Updated: {}", note.title)).ok();
                    }
                }
            }
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
        
        // Filter by tag if selected
        if let Some(tag_idx) = self.selected_tag_filter {
            self.search_results = storage.folders.iter().enumerate()
                .flat_map(|(folder_idx, folder)| {
                    folder.notes.iter().enumerate()
                        .filter(|(_, note)| note.tags.has_tag(tag_idx))
                        .map(move |(note_idx, _)| (folder_idx, note_idx))
                })
                .collect();
        } else {
            self.search_results = self.search.search(&storage.folders, &self.search_query);
        }
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
                        println!("✓ Note created: {} in folder {}", self.new_note_title, folder_idx);
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
    
    fn export_current_note(&mut self) {
        if let (Some(folder_idx), Some(note_idx)) = (self.selected_folder, self.selected_note) {
            let storage = self.storage.lock().unwrap();
            if let Some(folder) = storage.folders.get(folder_idx) {
                if let Some(note) = folder.notes.get(note_idx) {
                    match self.export_format {
                        ExportFormat::PDF => {
                            let output_path = PathBuf::from(format!("{}.pdf", note.title));
                            match pdf_export::PdfExporter::export_note(&note.title, &note.content, &output_path) {
                                Ok(_) => println!("✓ Exported to PDF: {:?}", output_path),
                                Err(e) => eprintln!("✗ PDF export failed: {}", e),
                            }
                        }
                        ExportFormat::Markdown => {
                            let output_path = PathBuf::from(format!("{}.md", note.title));
                            std::fs::write(&output_path, &note.content).ok();
                            println!("✓ Exported to Markdown: {:?}", output_path);
                        }
                        ExportFormat::PlainText => {
                            let output_path = PathBuf::from(format!("{}.txt", note.title));
                            std::fs::write(&output_path, &note.content).ok();
                            println!("✓ Exported to text: {:?}", output_path);
                        }
                    }
                }
            }
        }
    }
    
    fn toggle_note_encryption(&mut self) {
        if self.encryption_password != self.confirm_password {
            eprintln!("Passwords don't match!");
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
                                self.current_note_content = note.content.clone();
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
    
    fn load_note_versions(&mut self) {
        if let (Some(folder_idx), Some(note_idx)) = (self.selected_folder, self.selected_note) {
            let storage = self.storage.lock().unwrap();
            if let Some(folder) = storage.folders.get(folder_idx) {
                if let Some(note) = folder.notes.get(note_idx) {
                    if let Some(ref vc) = self.version_control {
                        let file_path = PathBuf::from(&note.file_path);
                        match vc.get_file_history(&file_path) {
                            Ok(versions) => self.note_versions = versions,
                            Err(e) => eprintln!("Failed to load versions: {}", e),
                        }
                    }
                }
            }
        }
    }
}

impl eframe::App for NoteTakingApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply current theme
        self.theme_manager.current_theme.apply_to_egui(ctx);
        
        self.render_top_panel(ctx);
        
        if self.sidebar_open {
            self.render_sidebar(ctx);
        }
        
        self.render_central_panel(ctx);
        
        // Dialogs
        self.render_dialogs(ctx);
    }
}

// Continue in next part...
