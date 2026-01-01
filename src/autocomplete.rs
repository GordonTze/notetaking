use std::collections::HashSet;

pub struct Autocomplete {
    words: HashSet<String>,
    enabled: bool,
}

impl Autocomplete {
    pub fn new() -> Self {
        let mut words = HashSet::new();
        
        // Common English words for autocomplete
        let common_words = vec![
            // Articles & Pronouns
            "the", "a", "an", "this", "that", "these", "those",
            "I", "you", "he", "she", "it", "we", "they", "me", "him", "her", "us", "them",
            "my", "your", "his", "her", "its", "our", "their",
            
            // Common Verbs
            "is", "are", "was", "were", "be", "been", "being",
            "have", "has", "had", "do", "does", "did", "done",
            "can", "could", "will", "would", "shall", "should", "may", "might", "must",
            "make", "made", "making", "get", "got", "getting", "give", "gave", "given",
            "take", "took", "taken", "come", "came", "coming", "go", "went", "gone",
            "see", "saw", "seen", "know", "knew", "known", "think", "thought",
            "want", "wanted", "need", "needed", "use", "used", "using",
            "work", "worked", "working", "find", "found", "tell", "told",
            "become", "became", "feel", "felt", "try", "tried", "leave", "left",
            "call", "called", "ask", "asked", "keep", "kept", "show", "showed",
            "write", "wrote", "written", "read", "reading", "create", "created",
            
            // Common Nouns
            "time", "person", "people", "year", "years", "way", "ways", "day", "days",
            "thing", "things", "man", "men", "woman", "women", "child", "children",
            "world", "life", "hand", "hands", "part", "parts", "place", "places",
            "case", "cases", "point", "points", "government", "company", "companies",
            "number", "numbers", "group", "groups", "problem", "problems",
            "fact", "facts", "work", "works", "question", "questions",
            "home", "house", "houses", "room", "rooms", "office", "offices",
            "note", "notes", "document", "documents", "file", "files",
            "project", "projects", "task", "tasks", "meeting", "meetings",
            "idea", "ideas", "plan", "plans", "goal", "goals",
            
            // Common Adjectives
            "good", "better", "best", "new", "newer", "newest", "first", "last",
            "long", "longer", "longest", "great", "greater", "greatest",
            "little", "less", "least", "own", "other", "others", "old", "older",
            "right", "big", "bigger", "biggest", "high", "higher", "highest",
            "different", "small", "smaller", "smallest", "large", "larger", "largest",
            "next", "early", "earlier", "earliest", "young", "younger", "youngest",
            "important", "few", "fewer", "public", "bad", "worse", "worst",
            "same", "able", "recent", "current", "previous", "possible",
            
            // Prepositions & Conjunctions
            "of", "to", "in", "for", "on", "with", "at", "by", "from", "up", "about",
            "into", "through", "during", "before", "after", "above", "below",
            "between", "under", "since", "without", "and", "but", "or", "if",
            "because", "as", "until", "while", "so", "than", "when", "where",
            
            // Common Adverbs
            "not", "only", "just", "also", "very", "even", "back", "there", "down",
            "still", "now", "then", "here", "well", "out", "up", "over", "again",
            "more", "most", "never", "always", "often", "sometimes", "usually",
            "really", "actually", "probably", "perhaps", "maybe", "however",
            "therefore", "furthermore", "moreover", "nevertheless",
            
            // Business & Work
            "project", "management", "business", "market", "product", "service",
            "customer", "client", "team", "manager", "employee", "department",
            "budget", "finance", "revenue", "profit", "sales", "marketing",
            "strategy", "plan", "goal", "objective", "deadline", "schedule",
            "report", "presentation", "analysis", "research", "data", "information",
            
            // Technology
            "software", "hardware", "computer", "system", "application",
            "program", "code", "development", "website", "internet", "email",
            "network", "server", "database", "technology", "digital", "online",
            
            // Academic
            "study", "research", "analysis", "theory", "practice", "method",
            "approach", "concept", "model", "framework", "result", "conclusion",
            "evidence", "example", "process", "system", "structure", "function",
            
            // Time
            "today", "tomorrow", "yesterday", "week", "month", "morning",
            "afternoon", "evening", "night", "hour", "minute", "second",
            "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday",
            "January", "February", "March", "April", "May", "June", "July",
            "August", "September", "October", "November", "December",
        ];
        
        for word in common_words {
            words.insert(word.to_string());
        }
        
        Self {
            words,
            enabled: true,
        }
    }
    
    pub fn add_word(&mut self, word: String) {
        self.words.insert(word.to_lowercase());
    }
    
    pub fn add_words(&mut self, words: Vec<String>) {
        for word in words {
            self.add_word(word);
        }
    }
    
    pub fn get_suggestions(&self, prefix: &str) -> Vec<String> {
        if !self.enabled || prefix.is_empty() || prefix.len() < 2 {
            return Vec::new();
        }
        
        let prefix_lower = prefix.to_lowercase();
        let mut suggestions: Vec<String> = self.words
            .iter()
            .filter(|word| word.starts_with(&prefix_lower))
            .cloned()
            .collect();
        
        suggestions.sort();
        suggestions.truncate(10); // Limit to 10 suggestions
        suggestions
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
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_suggestions() {
        let autocomplete = Autocomplete::new();
        let suggestions = autocomplete.get_suggestions("pro");
        assert!(suggestions.contains(&"project".to_string()));
        assert!(suggestions.contains(&"problem".to_string()));
    }
    
    #[test]
    fn test_add_word() {
        let mut autocomplete = Autocomplete::new();
        autocomplete.add_word("custom".to_string());
        let suggestions = autocomplete.get_suggestions("cus");
        assert!(suggestions.contains(&"custom".to_string()));
    }
}
