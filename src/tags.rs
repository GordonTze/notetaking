use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    pub color: [u8; 3],
}

impl Tag {
    pub fn new(name: String) -> Self {
        Self {
            name,
            color: Self::random_color(),
        }
    }
    
    pub fn with_color(name: String, color: [u8; 3]) -> Self {
        Self { name, color }
    }
    
    fn random_color() -> [u8; 3] {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        [rng.gen_range(50..200), rng.gen_range(50..200), rng.gen_range(50..200)]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TagManager {
    pub tags: Vec<Tag>,
}

impl TagManager {
    pub fn new() -> Self {
        Self {
            tags: Vec::new(),
        }
    }
    
    pub fn add_tag(&mut self, name: String) -> usize {
        // Check if tag already exists
        if let Some(idx) = self.tags.iter().position(|t| t.name == name) {
            return idx;
        }
        
        let tag = Tag::new(name);
        self.tags.push(tag);
        self.tags.len() - 1
    }
    
    pub fn get_tag(&self, name: &str) -> Option<&Tag> {
        self.tags.iter().find(|t| t.name == name)
    }
    
    pub fn get_tag_by_index(&self, index: usize) -> Option<&Tag> {
        self.tags.get(index)
    }
    
    pub fn remove_tag(&mut self, name: &str) {
        self.tags.retain(|t| t.name != name);
    }
    
    pub fn all_tags(&self) -> &[Tag] {
        &self.tags
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NoteTags {
    pub tag_indices: HashSet<usize>,
}

impl NoteTags {
    pub fn new() -> Self {
        Self {
            tag_indices: HashSet::new(),
        }
    }
    
    pub fn add_tag(&mut self, tag_index: usize) {
        self.tag_indices.insert(tag_index);
    }
    
    pub fn remove_tag(&mut self, tag_index: usize) {
        self.tag_indices.remove(&tag_index);
    }
    
    pub fn has_tag(&self, tag_index: usize) -> bool {
        self.tag_indices.contains(&tag_index)
    }
    
    pub fn get_tags<'a>(&self, manager: &'a TagManager) -> Vec<&'a Tag> {
        self.tag_indices
            .iter()
            .filter_map(|idx| manager.get_tag_by_index(*idx))
            .collect()
    }
    
    pub fn clear(&mut self) {
        self.tag_indices.clear();
    }
}

pub fn filter_notes_by_tag(notes: &[(usize, usize, NoteTags)], tag_index: usize) -> Vec<(usize, usize)> {
    notes
        .iter()
        .filter(|(_, _, tags)| tags.has_tag(tag_index))
        .map(|(folder_idx, note_idx, _)| (*folder_idx, *note_idx))
        .collect()
}
