use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteLink {
    pub source_folder: usize,
    pub source_note: usize,
    pub target_folder: usize,
    pub target_note: usize,
    pub link_text: String,
}

#[derive(Debug, Clone, Default)]
pub struct LinkManager {
    // Map from note ID to outgoing links
    pub outgoing_links: HashMap<(usize, usize), Vec<(usize, usize)>>,
    // Map from note ID to incoming links (backlinks)
    pub incoming_links: HashMap<(usize, usize), Vec<(usize, usize)>>,
}

impl LinkManager {
    pub fn new() -> Self {
        Self {
            outgoing_links: HashMap::new(),
            incoming_links: HashMap::new(),
        }
    }
    
    pub fn add_link(
        &mut self,
        source: (usize, usize),
        target: (usize, usize),
    ) {
        // Add to outgoing links
        self.outgoing_links
            .entry(source)
            .or_insert_with(Vec::new)
            .push(target);
        
        // Add to incoming links (backlinks)
        self.incoming_links
            .entry(target)
            .or_insert_with(Vec::new)
            .push(source);
    }
    
    pub fn remove_link(
        &mut self,
        source: (usize, usize),
        target: (usize, usize),
    ) {
        if let Some(links) = self.outgoing_links.get_mut(&source) {
            links.retain(|&t| t != target);
        }
        
        if let Some(backlinks) = self.incoming_links.get_mut(&target) {
            backlinks.retain(|&s| s != source);
        }
    }
    
    pub fn get_outgoing_links(&self, note: (usize, usize)) -> Vec<(usize, usize)> {
        self.outgoing_links
            .get(&note)
            .cloned()
            .unwrap_or_default()
    }
    
    pub fn get_backlinks(&self, note: (usize, usize)) -> Vec<(usize, usize)> {
        self.incoming_links
            .get(&note)
            .cloned()
            .unwrap_or_default()
    }
    
    pub fn scan_note_for_links(&mut self, content: &str, source: (usize, usize)) -> Vec<String> {
        // Extract [[Note Name]] style links
        let mut link_names = Vec::new();
        let mut chars = content.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '[' {
                if chars.peek() == Some(&'[') {
                    chars.next(); // consume second '['
                    
                    // Extract link text
                    let mut link_text = String::new();
                    let mut closed = false;
                    
                    while let Some(c) = chars.next() {
                        if c == ']' {
                            if chars.peek() == Some(&']') {
                                chars.next(); // consume second ']'
                                closed = true;
                                break;
                            }
                        }
                        link_text.push(c);
                    }
                    
                    if closed && !link_text.is_empty() {
                        link_names.push(link_text);
                    }
                }
            }
        }
        
        link_names
    }
    
    pub fn rebuild_links_for_note(
        &mut self,
        source: (usize, usize),
        content: &str,
        note_name_to_id: &HashMap<String, (usize, usize)>,
    ) {
        // Clear existing outgoing links for this note
        if let Some(old_targets) = self.outgoing_links.remove(&source) {
            // Remove backlinks
            for target in old_targets {
                if let Some(backlinks) = self.incoming_links.get_mut(&target) {
                    backlinks.retain(|&s| s != source);
                }
            }
        }
        
        // Scan for new links
        let link_names = self.scan_note_for_links(content, source);
        
        // Add new links
        for link_name in link_names {
            if let Some(&target) = note_name_to_id.get(&link_name) {
                self.add_link(source, target);
            }
        }
    }
    
    pub fn get_link_count(&self, note: (usize, usize)) -> (usize, usize) {
        let outgoing = self.get_outgoing_links(note).len();
        let incoming = self.get_backlinks(note).len();
        (outgoing, incoming)
    }
}

pub fn format_wiki_link(note_title: &str) -> String {
    format!("[[{}]]", note_title)
}

pub fn insert_wiki_link_at_cursor(content: &mut String, cursor_pos: usize, note_title: &str) {
    let link = format_wiki_link(note_title);
    content.insert_str(cursor_pos, &link);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_link_extraction() {
        let mut manager = LinkManager::new();
        let content = "Check out [[Note A]] and [[Note B]] for more info.";
        
        let links = manager.scan_note_for_links(content, (0, 0));
        
        assert_eq!(links.len(), 2);
        assert_eq!(links[0], "Note A");
        assert_eq!(links[1], "Note B");
    }
    
    #[test]
    fn test_backlinks() {
        let mut manager = LinkManager::new();
        
        manager.add_link((0, 0), (1, 1));
        manager.add_link((0, 1), (1, 1));
        
        let backlinks = manager.get_backlinks((1, 1));
        assert_eq!(backlinks.len(), 2);
        assert!(backlinks.contains(&(0, 0)));
        assert!(backlinks.contains(&(0, 1)));
    }
}
