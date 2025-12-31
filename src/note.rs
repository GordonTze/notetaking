use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
    pub file_path: String,
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
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteMetadata {
    pub created_at: String,
    pub updated_at: String,
}

impl NoteMetadata {
    pub fn new() -> Self {
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Self {
            created_at: now.clone(),
            updated_at: now,
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
