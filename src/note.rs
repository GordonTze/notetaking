use chrono::Utc;
use serde::{Deserialize, Serialize};
use crate::tags::NoteTags;
use crate::encryption::EncryptedData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
    pub file_path: String,
    
    // New features
    pub tags: NoteTags,
    pub is_encrypted: bool,
    pub encrypted_data: Option<EncryptedData>,
    pub linked_notes: Vec<(usize, usize)>, // (folder_idx, note_idx)
    pub embedded_images: Vec<String>, // Image paths
}

impl Note {
    pub fn new(title: String, file_path: String) -> Self {
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Self {
            title,
            content: String::new(),
            created_at: now.clone(),
            updated_at: now,
            file_path,
            tags: NoteTags::new(),
            is_encrypted: false,
            encrypted_data: None,
            linked_notes: Vec::new(),
            embedded_images: Vec::new(),
        }
    }
    
    pub fn update_timestamp(&mut self) {
        self.updated_at = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    }
    
    pub fn from_file(file_path: String, title: String, content: String, metadata: NoteMetadata) -> Self {
        Self {
            title,
            content,
            created_at: metadata.created_at,
            updated_at: metadata.updated_at,
            file_path,
            tags: metadata.tags,
            is_encrypted: metadata.is_encrypted,
            encrypted_data: metadata.encrypted_data,
            linked_notes: metadata.linked_notes,
            embedded_images: metadata.embedded_images,
        }
    }
    
    pub fn add_tag(&mut self, tag_index: usize) {
        self.tags.add_tag(tag_index);
    }
    
    pub fn remove_tag(&mut self, tag_index: usize) {
        self.tags.remove_tag(tag_index);
    }
    
    pub fn add_image(&mut self, image_path: String) {
        self.embedded_images.push(image_path);
    }
    
    pub fn link_to(&mut self, target: (usize, usize)) {
        if !self.linked_notes.contains(&target) {
            self.linked_notes.push(target);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteMetadata {
    pub created_at: String,
    pub updated_at: String,
    pub tags: NoteTags,
    pub is_encrypted: bool,
    pub encrypted_data: Option<EncryptedData>,
    pub linked_notes: Vec<(usize, usize)>,
    pub embedded_images: Vec<String>,
}

impl NoteMetadata {
    pub fn new() -> Self {
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Self {
            created_at: now.clone(),
            updated_at: now,
            tags: NoteTags::new(),
            is_encrypted: false,
            encrypted_data: None,
            linked_notes: Vec::new(),
            embedded_images: Vec::new(),
        }
    }
    
    pub fn from_note(note: &Note) -> Self {
        Self {
            created_at: note.created_at.clone(),
            updated_at: note.updated_at.clone(),
            tags: note.tags.clone(),
            is_encrypted: note.is_encrypted,
            encrypted_data: note.encrypted_data.clone(),
            linked_notes: note.linked_notes.clone(),
            embedded_images: note.embedded_images.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub name: String,
    pub notes: Vec<Note>,
    pub path: String,
}

impl Folder {
    pub fn new(name: String, path: String) -> Self {
        Self {
            name,
            notes: Vec::new(),
            path,
        }
    }
    
    pub fn add_note(&mut self, note: Note) {
        self.notes.push(note);
    }
}

