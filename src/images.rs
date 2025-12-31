use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use image::DynamicImage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedImage {
    pub path: String,
    pub caption: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

impl EmbeddedImage {
    pub fn new(path: String) -> Self {
        Self {
            path,
            caption: String::new(),
            width: None,
            height: None,
        }
    }
    
    pub fn with_caption(mut self, caption: String) -> Self {
        self.caption = caption;
        self
    }
    
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImageManager {
    pub images: Vec<EmbeddedImage>,
}

impl ImageManager {
    pub fn new() -> Self {
        Self {
            images: Vec::new(),
        }
    }
    
    pub fn add_image(&mut self, path: String) -> usize {
        let image = EmbeddedImage::new(path);
        self.images.push(image);
        self.images.len() - 1
    }
    
    pub fn add_image_with_caption(&mut self, path: String, caption: String) -> usize {
        let image = EmbeddedImage::new(path).with_caption(caption);
        self.images.push(image);
        self.images.len() - 1
    }
    
    pub fn get_image(&self, index: usize) -> Option<&EmbeddedImage> {
        self.images.get(index)
    }
    
    pub fn remove_image(&mut self, index: usize) {
        if index < self.images.len() {
            self.images.remove(index);
        }
    }
    
    pub fn copy_image_to_note_folder(
        &self,
        source_path: &Path,
        note_folder: &Path,
    ) -> std::io::Result<PathBuf> {
        // Create images subdirectory
        let images_dir = note_folder.join("images");
        fs::create_dir_all(&images_dir)?;
        
        // Get filename
        let filename = source_path
            .file_name()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid filename"))?;
        
        let dest_path = images_dir.join(filename);
        
        // Copy file
        fs::copy(source_path, &dest_path)?;
        
        Ok(dest_path)
    }
    
    pub fn load_image(&self, path: &Path) -> Result<DynamicImage, String> {
        image::open(path).map_err(|e| format!("Failed to load image: {}", e))
    }
}

pub fn markdown_image_syntax(image: &EmbeddedImage) -> String {
    if image.caption.is_empty() {
        format!("![Image]({})", image.path)
    } else {
        format!("![{}]({})", image.caption, image.path)
    }
}

pub fn extract_images_from_markdown(content: &str) -> Vec<(String, String)> {
    // Simple regex-like extraction of ![alt](path) patterns
    let mut images = Vec::new();
    let mut chars = content.chars().peekable();
    
    while let Some(ch) = chars.next() {
        if ch == '!' {
            if chars.peek() == Some(&'[') {
                chars.next(); // consume '['
                
                // Extract alt text
                let mut alt = String::new();
                while let Some(&c) = chars.peek() {
                    if c == ']' {
                        chars.next();
                        break;
                    }
                    alt.push(chars.next().unwrap());
                }
                
                // Check for '('
                if chars.peek() == Some(&'(') {
                    chars.next(); // consume '('
                    
                    // Extract path
                    let mut path = String::new();
                    while let Some(&c) = chars.peek() {
                        if c == ')' {
                            chars.next();
                            break;
                        }
                        path.push(chars.next().unwrap());
                    }
                    
                    images.push((alt, path));
                }
            }
        }
    }
    
    images
}
