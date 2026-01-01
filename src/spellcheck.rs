use std::collections::HashSet;

pub struct SpellChecker {
    dictionary: HashSet<String>,
    enabled: bool,
}

impl SpellChecker {
    pub fn new() -> Self {
        let mut dictionary = HashSet::new();
        
        // Basic English dictionary words
        let words = vec![
            // Same words as autocomplete plus more
            "the", "be", "to", "of", "and", "a", "in", "that", "have", "I",
            "it", "for", "not", "on", "with", "he", "as", "you", "do", "at",
            "this", "but", "his", "by", "from", "they", "we", "say", "her", "she",
            "or", "an", "will", "my", "one", "all", "would", "there", "their",
            "what", "so", "up", "out", "if", "about", "who", "get", "which", "go",
            "me", "when", "make", "can", "like", "time", "no", "just", "him", "know",
            "take", "people", "into", "year", "your", "good", "some", "could", "them",
            "see", "other", "than", "then", "now", "look", "only", "come", "its", "over",
            "think", "also", "back", "after", "use", "two", "how", "our", "work",
            "first", "well", "way", "even", "new", "want", "because", "any", "these",
            "give", "day", "most", "us", "is", "was", "are", "been", "has", "had",
            "were", "said", "did", "having", "may", "should", "could", "would",
            
            // Common nouns
            "person", "people", "man", "woman", "child", "children", "family",
            "world", "life", "hand", "part", "place", "case", "point", "week",
            "company", "number", "group", "problem", "fact", "home", "house",
            "note", "notes", "document", "file", "project", "task", "meeting",
            "idea", "plan", "goal", "room", "office", "work", "business",
            
            // Common verbs
            "create", "created", "creating", "write", "wrote", "written", "writing",
            "read", "reading", "update", "updated", "updating", "delete", "deleted",
            "find", "found", "finding", "search", "searched", "searching",
            "need", "needed", "needing", "want", "wanted", "wanting",
            "start", "started", "starting", "finish", "finished", "finishing",
            "complete", "completed", "completing", "save", "saved", "saving",
            "open", "opened", "opening", "close", "closed", "closing",
            
            // Common adjectives
            "important", "good", "great", "new", "old", "first", "last", "long",
            "little", "own", "other", "right", "big", "high", "different", "small",
            "large", "next", "early", "young", "few", "public", "bad", "same",
            "able", "current", "recent", "previous", "possible", "available",
            
            // Technology
            "computer", "software", "hardware", "internet", "website", "email",
            "password", "username", "login", "logout", "download", "upload",
            "file", "folder", "directory", "document", "text", "image", "video",
            "application", "program", "system", "network", "server", "database",
            
            // Time
            "today", "tomorrow", "yesterday", "week", "month", "year", "day",
            "morning", "afternoon", "evening", "night", "hour", "minute", "second",
            "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday",
            "January", "February", "March", "April", "May", "June", "July",
            "August", "September", "October", "November", "December",
            
            // Numbers
            "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten",
            "first", "second", "third", "fourth", "fifth",
            
            // Markdown/formatting
            "bold", "italic", "header", "list", "link", "image", "code", "quote",
            "table", "section", "paragraph", "line", "format", "style",
        ];
        
        for word in words {
            dictionary.insert(word.to_string());
        }
        
        Self {
            dictionary,
            enabled: true,
        }
    }
    
    pub fn is_correct(&self, word: &str) -> bool {
        if !self.enabled {
            return true;
        }
        
        // Ignore empty strings, numbers, and single characters
        if word.is_empty() || word.len() == 1 {
            return true;
        }
        
        // Ignore words with special characters (might be code, URLs, etc.)
        if word.contains("://") || word.contains('@') || word.contains('#') {
            return true;
        }
        
        // Check if word is in dictionary (case-insensitive)
        self.dictionary.contains(&word.to_lowercase())
    }
    
    pub fn check_text(&self, text: &str) -> Vec<(usize, usize, String)> {
        if !self.enabled {
            return Vec::new();
        }
        
        let mut misspelled = Vec::new();
        let mut current_pos = 0;
        
        for word in text.split_whitespace() {
            // Find the actual position in the text
            if let Some(pos) = text[current_pos..].find(word) {
                let word_start = current_pos + pos;
                let word_end = word_start + word.len();
                
                // Clean word from punctuation
                let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric());
                
                if !clean_word.is_empty() && !self.is_correct(clean_word) {
                    misspelled.push((word_start, word_end, clean_word.to_string()));
                }
                
                current_pos = word_end;
            }
        }
        
        misspelled
    }
    
    pub fn add_to_dictionary(&mut self, word: String) {
        self.dictionary.insert(word.to_lowercase());
    }
    
    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }
    
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    // Simple suggestion using Levenshtein-like approach
    pub fn suggest(&self, word: &str) -> Vec<String> {
        if word.is_empty() {
            return Vec::new();
        }
        
        let word_lower = word.to_lowercase();
        let mut suggestions: Vec<(usize, String)> = Vec::new();
        
        for dict_word in &self.dictionary {
            // Calculate simple edit distance
            let distance = self.simple_distance(&word_lower, dict_word);
            if distance <= 2 {
                suggestions.push((distance, dict_word.clone()));
            }
        }
        
        // Sort by distance
        suggestions.sort_by_key(|(d, _)| *d);
        suggestions.truncate(5);
        
        suggestions.into_iter().map(|(_, w)| w).collect()
    }
    
    fn simple_distance(&self, s1: &str, s2: &str) -> usize {
        // Simple character difference count (not true Levenshtein)
        let len_diff = (s1.len() as i32 - s2.len() as i32).abs() as usize;
        let mut char_diff = 0;
        
        for (c1, c2) in s1.chars().zip(s2.chars()) {
            if c1 != c2 {
                char_diff += 1;
            }
        }
        
        len_diff + char_diff
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_correct_words() {
        let checker = SpellChecker::new();
        assert!(checker.is_correct("the"));
        assert!(checker.is_correct("hello"));
        assert!(!checker.is_correct("wrng"));
    }
    
    #[test]
    fn test_check_text() {
        let checker = SpellChecker::new();
        let errors = checker.check_text("This is a tst");
        assert!(!errors.is_empty());
    }
}
