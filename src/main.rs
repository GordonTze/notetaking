use eframe::egui;
use std::sync::Arc;
use std::sync::Mutex;

mod note;
mod storage;
mod search;

use note::Folder;
use storage::Storage;
use search::FuzzySearch;

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
    is_editing: bool,
    sidebar_open: bool,
}

impl NoteTakingApp {
    fn new() -> Self {
        let storage = Storage::new("./notes_data".to_string());
        let search = FuzzySearch::new();
        
        Self {
            storage: Arc::new(Mutex::new(storage)),
            search,
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
        }
    }
    
    fn save_current_note(&mut self) {
        if let (Some(folder_idx), Some(note_idx)) = (self.selected_folder, self.selected_note) {
            let mut storage = self.storage.lock().unwrap();
            if let Some(folder) = storage.folders.get_mut(folder_idx) {
                if let Some(note) = folder.notes.get_mut(note_idx) {
                    note.content = self.current_note_content.clone();
                    note.update_timestamp();
                    storage.save_note(folder_idx, note_idx).ok();
                }
            }
        }
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
}

impl eframe::App for NoteTakingApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top panel with search and controls
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Toggle sidebar button
                if ui.button(if self.sidebar_open { "◀ Close" } else { "▶ Open" }).clicked() {
                    self.sidebar_open = !self.sidebar_open;
                }
                
                ui.separator();
                
                // Search bar
                ui.label("🔍");
                let search_response = ui.text_edit_singleline(&mut self.search_query);
                if search_response.changed() {
                    self.perform_search();
                }
                
                ui.separator();
                
                // Sync button
                if ui.button("☁ Sync to Cloud").clicked() {
                    self.sync_to_cloud();
                }
                
                ui.separator();
                
                // New folder button
                if ui.button("📁 New Folder").clicked() {
                    self.show_new_folder_dialog = true;
                }
                
                // New note button - always visible
                ui.add_enabled_ui(self.selected_folder.is_some(), |ui| {
                    if ui.button("📝 New Note").clicked() {
                        self.show_new_note_dialog = true;
                    }
                });
                
                // Show hint if no folder selected
                if self.selected_folder.is_none() {
                    ui.label("(Select a folder first)");
                }
            });
        });
        
        // Sidebar with folders and notes
        if self.sidebar_open {
            egui::SidePanel::left("sidebar")
                .resizable(true)
                .default_width(250.0)
                .show(ctx, |ui| {
                    ui.heading("Folders & Notes");
                    ui.separator();
                    
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        // Show search results if searching
                        if !self.search_query.is_empty() && !self.search_results.is_empty() {
                            ui.label("Search Results:");
                            ui.separator();
                            
                            // Collect display data first to avoid borrow issues
                            let search_display: Vec<_> = {
                                let storage = self.storage.lock().unwrap();
                                self.search_results.iter().filter_map(|(folder_idx, note_idx)| {
                                    storage.folders.get(*folder_idx).and_then(|folder| {
                                        folder.notes.get(*note_idx).map(|note| {
                                            (
                                                *folder_idx,
                                                *note_idx,
                                                format!("📄 {} (in {})", note.title, folder.name),
                                                note.content.clone()
                                            )
                                        })
                                    })
                                }).collect()
                            };
                            
                            for (folder_idx, note_idx, label, content) in search_display {
                                if ui.selectable_label(
                                    self.selected_folder == Some(folder_idx) && 
                                    self.selected_note == Some(note_idx),
                                    label
                                ).clicked() {
                                    self.save_current_note();
                                    self.selected_folder = Some(folder_idx);
                                    self.selected_note = Some(note_idx);
                                    self.current_note_content = content;
                                    self.is_editing = false;
                                }
                            }
                        } else {
                            // Show folder tree - collect data first to avoid borrow issues
                            let folders_display: Vec<_> = {
                                let storage = self.storage.lock().unwrap();
                                storage.folders.iter().enumerate().map(|(folder_idx, folder)| {
                                    let notes: Vec<_> = folder.notes.iter().enumerate()
                                        .map(|(note_idx, note)| (note_idx, note.title.clone(), note.content.clone()))
                                        .collect();
                                    (folder_idx, folder.name.clone(), notes)
                                }).collect()
                            };
                            
                            for (folder_idx, folder_name, notes) in folders_display {
                                // Show if this folder is selected
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
                                    for (note_idx, title, content) in notes {
                                        if ui.selectable_label(
                                            self.selected_folder == Some(folder_idx) && 
                                            self.selected_note == Some(note_idx),
                                            &title
                                        ).clicked() {
                                            self.save_current_note();
                                            self.selected_folder = Some(folder_idx);
                                            self.selected_note = Some(note_idx);
                                            self.current_note_content = content;
                                            self.is_editing = false;
                                        }
                                    }
                                });
                                
                                // Click on folder name selects the folder
                                if header_response.header_response.clicked() {
                                    self.selected_folder = Some(folder_idx);
                                    self.selected_note = None;
                                    self.current_note_content.clear();
                                    println!("✓ Folder selected: {} (index {})", folder_name, folder_idx);
                                }
                            }
                        }
                    });
                });
        }
        
        // Main editor panel
        egui::CentralPanel::default().show(ctx, |ui| {
            if let (Some(folder_idx), Some(note_idx)) = (self.selected_folder, self.selected_note) {
                // Collect note data first to avoid borrow conflicts
                let note_data = {
                    let storage = self.storage.lock().unwrap();
                    storage.folders.get(folder_idx).and_then(|folder| {
                        folder.notes.get(note_idx).map(|note| {
                            (note.title.clone(), note.created_at.clone(), note.updated_at.clone())
                        })
                    })
                };
                
                if let Some((title, created_at, updated_at)) = note_data {
                    ui.horizontal(|ui| {
                        ui.heading(&title);
                        ui.separator();
                        
                        if self.is_editing {
                            if ui.button("💾 Save").clicked() {
                                self.save_current_note();
                                self.is_editing = false;
                            }
                            if ui.button("❌ Cancel").clicked() {
                                // Reload content from storage
                                let storage = self.storage.lock().unwrap();
                                if let Some(folder) = storage.folders.get(folder_idx) {
                                    if let Some(note) = folder.notes.get(note_idx) {
                                        self.current_note_content = note.content.clone();
                                    }
                                }
                                self.is_editing = false;
                            }
                        } else {
                            if ui.button("✏ Edit").clicked() {
                                self.is_editing = true;
                            }
                        }
                    });
                    
                    ui.separator();
                    
                    ui.label(format!("Created: {}", created_at));
                    ui.label(format!("Updated: {}", updated_at));
                    
                    ui.separator();
                    
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        if self.is_editing {
                            ui.add(
                                egui::TextEdit::multiline(&mut self.current_note_content)
                                    .desired_width(f32::INFINITY)
                                    .desired_rows(30)
                                    .font(egui::TextStyle::Monospace)
                            );
                        } else {
                            // Render markdown
                            ui.add(
                                egui::TextEdit::multiline(&mut self.current_note_content.as_str())
                                    .desired_width(f32::INFINITY)
                                    .desired_rows(30)
                                    .interactive(false)
                            );
                        }
                    });
                }
            } else {
                ui.vertical_centered(|ui| {
                    ui.add_space(200.0);
                    ui.heading("Welcome to Notetaking App");
                    ui.label("Select a note from the sidebar or create a new one");
                });
            }
        });
        
        // New folder dialog
        if self.show_new_folder_dialog {
            egui::Window::new("Create New Folder")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Folder name:");
                    ui.text_edit_singleline(&mut self.new_folder_name);
                    
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
        
        // New note dialog
        if self.show_new_note_dialog {
            egui::Window::new("Create New Note")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Note title:");
                    let response = ui.text_edit_singleline(&mut self.new_note_title);
                    
                    // Auto-focus the text field
                    if response.changed() {
                        // Do nothing, just to capture the response
                    }
                    
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
                            ui.label(format!("Creating in folder: {}", folder.name));
                        }
                    }
                });
        }
    }
}
