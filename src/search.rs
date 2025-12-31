use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

use crate::note::Folder;

pub struct FuzzySearch {
    matcher: SkimMatcherV2,
}

impl FuzzySearch {
    pub fn new() -> Self {
        Self {
            matcher: SkimMatcherV2::default(),
        }
    }
    
    pub fn search(&self, folders: &[Folder], query: &str) -> Vec<(usize, usize)> {
        let mut results = Vec::new();
        
        for (folder_idx, folder) in folders.iter().enumerate() {
            for (note_idx, note) in folder.notes.iter().enumerate() {
                // Search in title
                let title_score = self.matcher.fuzzy_match(&note.title, query);
                
                // Search in content
                let content_score = self.matcher.fuzzy_match(&note.content, query);
                
                // If either matches, add to results
                if title_score.is_some() || content_score.is_some() {
                    results.push((folder_idx, note_idx));
                }
            }
        }
        
        results
    }
}
