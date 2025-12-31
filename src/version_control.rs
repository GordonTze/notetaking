use git2::{Repository, Signature, IndexAddOption, Oid};
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    pub commit_id: String,
    pub message: String,
    pub timestamp: String,
    pub author: String,
}

pub struct VersionControl {
    repo_path: PathBuf,
}

impl VersionControl {
    pub fn new(repo_path: PathBuf) -> Result<Self, String> {
        Ok(Self { repo_path })
    }
    
    pub fn init(&self) -> Result<(), String> {
        // Initialize git repository if it doesn't exist
        if !self.repo_path.join(".git").exists() {
            Repository::init(&self.repo_path)
                .map_err(|e| format!("Failed to init repo: {}", e))?;
        }
        Ok(())
    }
    
    pub fn commit_note(&self, file_path: &Path, message: &str) -> Result<String, String> {
        let repo = Repository::open(&self.repo_path)
            .map_err(|e| format!("Failed to open repo: {}", e))?;
        
        // Get the index
        let mut index = repo.index()
            .map_err(|e| format!("Failed to get index: {}", e))?;
        
        // Add file to index
        let relative_path = file_path
            .strip_prefix(&self.repo_path)
            .map_err(|e| format!("Path error: {}", e))?;
        
        index.add_path(relative_path)
            .map_err(|e| format!("Failed to add file: {}", e))?;
        
        index.write()
            .map_err(|e| format!("Failed to write index: {}", e))?;
        
        // Create tree
        let tree_id = index.write_tree()
            .map_err(|e| format!("Failed to write tree: {}", e))?;
        
        let tree = repo.find_tree(tree_id)
            .map_err(|e| format!("Failed to find tree: {}", e))?;
        
        // Get signature
        let signature = Signature::now("NoteApp User", "user@noteapp.local")
            .map_err(|e| format!("Failed to create signature: {}", e))?;
        
        // Get parent commit if exists
        let parent_commit = repo.head().ok()
            .and_then(|h| h.target())
            .and_then(|oid| repo.find_commit(oid).ok());
        
        // Create commit
        let commit_id = if let Some(parent) = parent_commit {
            repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                message,
                &tree,
                &[&parent],
            )
        } else {
            repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                message,
                &tree,
                &[],
            )
        }.map_err(|e| format!("Failed to commit: {}", e))?;
        
        Ok(commit_id.to_string())
    }
    
    pub fn get_file_history(&self, file_path: &Path) -> Result<Vec<Version>, String> {
        let repo = Repository::open(&self.repo_path)
            .map_err(|e| format!("Failed to open repo: {}", e))?;
        
        let mut revwalk = repo.revwalk()
            .map_err(|e| format!("Failed to create revwalk: {}", e))?;
        
        revwalk.push_head()
            .map_err(|e| format!("Failed to push head: {}", e))?;
        
        let relative_path = file_path
            .strip_prefix(&self.repo_path)
            .map_err(|e| format!("Path error: {}", e))?;
        
        let mut versions = Vec::new();
        
        for oid_result in revwalk {
            let oid = oid_result.map_err(|e| format!("Walk error: {}", e))?;
            let commit = repo.find_commit(oid)
                .map_err(|e| format!("Failed to find commit: {}", e))?;
            
            // Check if this commit affects our file
            let tree = commit.tree()
                .map_err(|e| format!("Failed to get tree: {}", e))?;
            
            if tree.get_path(relative_path).is_ok() {
                let timestamp = DateTime::from_timestamp(commit.time().seconds(), 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "Unknown".to_string());
                
                versions.push(Version {
                    commit_id: oid.to_string(),
                    message: commit.message().unwrap_or("No message").to_string(),
                    timestamp,
                    author: commit.author().name().unwrap_or("Unknown").to_string(),
                });
            }
        }
        
        Ok(versions)
    }
    
    pub fn restore_version(&self, file_path: &Path, commit_id: &str) -> Result<String, String> {
        let repo = Repository::open(&self.repo_path)
            .map_err(|e| format!("Failed to open repo: {}", e))?;
        
        let oid = Oid::from_str(commit_id)
            .map_err(|e| format!("Invalid commit ID: {}", e))?;
        
        let commit = repo.find_commit(oid)
            .map_err(|e| format!("Failed to find commit: {}", e))?;
        
        let tree = commit.tree()
            .map_err(|e| format!("Failed to get tree: {}", e))?;
        
        let relative_path = file_path
            .strip_prefix(&self.repo_path)
            .map_err(|e| format!("Path error: {}", e))?;
        
        let entry = tree.get_path(relative_path)
            .map_err(|e| format!("File not found in commit: {}", e))?;
        
        let object = entry.to_object(&repo)
            .map_err(|e| format!("Failed to get object: {}", e))?;
        
        let blob = object.as_blob()
            .ok_or("Not a blob")?;
        
        let content = std::str::from_utf8(blob.content())
            .map_err(|e| format!("Invalid UTF-8: {}", e))?;
        
        Ok(content.to_string())
    }
    
    pub fn get_diff(&self, commit_id1: &str, commit_id2: &str) -> Result<String, String> {
        let repo = Repository::open(&self.repo_path)
            .map_err(|e| format!("Failed to open repo: {}", e))?;
        
        let oid1 = Oid::from_str(commit_id1)
            .map_err(|e| format!("Invalid commit ID 1: {}", e))?;
        let oid2 = Oid::from_str(commit_id2)
            .map_err(|e| format!("Invalid commit ID 2: {}", e))?;
        
        let commit1 = repo.find_commit(oid1)
            .map_err(|e| format!("Failed to find commit 1: {}", e))?;
        let commit2 = repo.find_commit(oid2)
            .map_err(|e| format!("Failed to find commit 2: {}", e))?;
        
        let tree1 = commit1.tree()
            .map_err(|e| format!("Failed to get tree 1: {}", e))?;
        let tree2 = commit2.tree()
            .map_err(|e| format!("Failed to get tree 2: {}", e))?;
        
        let diff = repo.diff_tree_to_tree(Some(&tree1), Some(&tree2), None)
            .map_err(|e| format!("Failed to create diff: {}", e))?;
        
        // Convert diff to string (simplified)
        let stats = diff.stats()
            .map_err(|e| format!("Failed to get stats: {}", e))?;
        
        Ok(format!(
            "Files changed: {}, Insertions: {}, Deletions: {}",
            stats.files_changed(),
            stats.insertions(),
            stats.deletions()
        ))
    }
}
