use std::fs;
use std::io;
use std::path::Path;

use crate::note::{Note, Folder, NoteMetadata};

pub struct Storage {
    base_path: String,
    pub folders: Vec<Folder>,
}

impl Storage {
    pub fn new(base_path: String) -> Self {
        // Create base directory if it doesn't exist
        fs::create_dir_all(&base_path).ok();
        
        let mut storage = Self {
            base_path,
            folders: Vec::new(),
        };
        
        // Load existing notes
        storage.load_all_notes();
        
        storage
    }
    
    fn load_all_notes(&mut self) {
        self.folders.clear();
        
        // Walk through directory structure
        let base = Path::new(&self.base_path);
        
        if !base.exists() {
            return;
        }
        
        // Get all immediate subdirectories (folders)
        if let Ok(entries) = fs::read_dir(base) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(folder_name) = path.file_name() {
                        let folder_name = folder_name.to_string_lossy().to_string();
                        let folder_path = path.to_string_lossy().to_string();
                        
                        let mut folder = Folder::new(folder_name, folder_path.clone());
                        
                        // Load notes from this folder
                        if let Ok(note_entries) = fs::read_dir(&path) {
                            for note_entry in note_entries.flatten() {
                                let note_path = note_entry.path();
                                if note_path.extension().and_then(|s| s.to_str()) == Some("md") {
                                    if let Ok(note) = self.load_note(&note_path) {
                                        folder.add_note(note);
                                    }
                                }
                            }
                        }
                        
                        self.folders.push(folder);
                    }
                }
            }
        }
    }
    
    fn load_note(&self, path: &Path) -> io::Result<Note> {
        let content = fs::read_to_string(path)?;
        let title = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
            .to_string();
        
        // Try to load metadata
        let metadata_path = path.with_extension("meta");
        let metadata = if metadata_path.exists() {
            fs::read_to_string(&metadata_path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_else(NoteMetadata::new)
        } else {
            NoteMetadata::new()
        };
        
        Ok(Note::from_file(
            path.to_string_lossy().to_string(),
            title,
            content,
            metadata,
        ))
    }
    
    pub fn create_folder(&mut self, name: &str) -> io::Result<()> {
        let folder_path = Path::new(&self.base_path).join(name);
        fs::create_dir_all(&folder_path)?;
        
        let folder = Folder::new(
            name.to_string(),
            folder_path.to_string_lossy().to_string(),
        );
        
        self.folders.push(folder);
        Ok(())
    }
    
    pub fn create_note(&mut self, folder_idx: usize, title: &str) -> io::Result<()> {
        if let Some(folder) = self.folders.get_mut(folder_idx) {
            let file_name = format!("{}.md", sanitize_filename(title));
            let file_path = Path::new(&folder.path).join(&file_name);
            
            // Create empty file
            fs::write(&file_path, "")?;
            
            // Create metadata file
            let metadata = NoteMetadata::new();
            let metadata_path = file_path.with_extension("meta");
            let metadata_json = serde_json::to_string_pretty(&metadata)?;
            fs::write(&metadata_path, metadata_json)?;
            
            let note = Note::new(title.to_string(), file_path.to_string_lossy().to_string());
            folder.add_note(note);
            
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "Folder not found"))
        }
    }
    
    pub fn save_note(&mut self, folder_idx: usize, note_idx: usize) -> io::Result<()> {
        if let Some(folder) = self.folders.get_mut(folder_idx) {
            if let Some(note) = folder.notes.get_mut(note_idx) {
                // Save content
                fs::write(&note.file_path, &note.content)?;
                
                // Save metadata
                let metadata = NoteMetadata {
                    created_at: note.created_at.clone(),
                    updated_at: note.updated_at.clone(),
                };
                let metadata_path = Path::new(&note.file_path).with_extension("meta");
                let metadata_json = serde_json::to_string_pretty(&metadata)?;
                fs::write(&metadata_path, metadata_json)?;
                
                return Ok(());
            }
        }
        Err(io::Error::new(io::ErrorKind::NotFound, "Note not found"))
    }
    
    pub fn export_to_cloud(&self) -> io::Result<String> {
        // This creates a backup/sync folder that user can manually upload to cloud
        let cloud_path = format!("{}_cloud_sync", self.base_path);
        
        // Remove old sync folder if exists
        if Path::new(&cloud_path).exists() {
            fs::remove_dir_all(&cloud_path)?;
        }
        
        // Create new sync folder
        fs::create_dir_all(&cloud_path)?;
        
        // Copy all folders and notes
        for folder in &self.folders {
            let folder_sync_path = Path::new(&cloud_path).join(&folder.name);
            fs::create_dir_all(&folder_sync_path)?;
            
            for note in &folder.notes {
                let note_path = Path::new(&note.file_path);
                let note_name = note_path.file_name().unwrap();
                let dest_path = folder_sync_path.join(note_name);
                
                fs::copy(&note.file_path, &dest_path)?;
                
                // Copy metadata
                let metadata_path = note_path.with_extension("meta");
                if metadata_path.exists() {
                    let dest_meta = dest_path.with_extension("meta");
                    fs::copy(&metadata_path, &dest_meta)?;
                }
            }
        }
        
        Ok(cloud_path)
    }
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim()
        .to_string()
}
