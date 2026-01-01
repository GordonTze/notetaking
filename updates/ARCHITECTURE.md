# Architecture Overview

## System Design

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     UI Layer (egui)                     │
│  ┌─────────────┬──────────────┬─────────────────────┐  │
│  │  Top Panel  │  Side Panel  │   Central Panel     │  │
│  │  (Search +  │  (Folders +  │   (Note Editor)     │  │
│  │   Actions)  │    Notes)    │                     │  │
│  └─────────────┴──────────────┴─────────────────────┘  │
└────────────────────┬────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────┐
│                Application State                        │
│  • Selected folder/note                                 │
│  • Current content buffer                               │
│  • Search state                                         │
│  • UI flags                                             │
└────────────────────┬────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────┐
│                  Business Logic                         │
│  ┌──────────────┬──────────────┬────────────────────┐  │
│  │   Storage    │    Search    │   Note/Folder      │  │
│  │   Manager    │    Engine    │   Management       │  │
│  └──────────────┴──────────────┴────────────────────┘  │
└────────────────────┬────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────┐
│                File System Layer                        │
│  notes_data/                                            │
│  ├── Folder1/                                           │
│  │   ├── note1.md                                       │
│  │   └── note1.meta                                     │
│  └── Folder2/                                           │
│      └── note2.md                                       │
└─────────────────────────────────────────────────────────┘
```

## Module Breakdown

### main.rs
**Purpose**: Application entry point and UI management

**Key Components**:
- `NoteTakingApp`: Main application state struct
- `eframe::App` implementation: UI rendering loop
- Event handlers: Button clicks, text input, note selection

**State Management**:
```rust
struct NoteTakingApp {
    storage: Arc<Mutex<Storage>>,  // Thread-safe storage
    search: FuzzySearch,            // Search engine
    
    // Selection state
    selected_folder: Option<usize>,
    selected_note: Option<usize>,
    current_note_content: String,   // Edit buffer
    
    // Search state
    search_query: String,
    search_results: Vec<(usize, usize)>,
    
    // UI state
    is_editing: bool,
    sidebar_open: bool,
    // ... dialog states
}
```

**UI Flow**:
1. Top panel: Search bar, action buttons
2. Side panel: Folder tree, note list, search results
3. Central panel: Note viewer/editor
4. Dialogs: New folder/note creation

### note.rs
**Purpose**: Data structures and models

**Structures**:

```rust
struct Note {
    title: String,
    content: String,
    created_at: String,
    updated_at: String,
    file_path: String,
}

struct Folder {
    name: String,
    notes: Vec<Note>,
    path: String,
}

struct NoteMetadata {
    created_at: String,
    updated_at: String,
}
```

**Responsibilities**:
- Note creation with timestamps
- Timestamp updates
- Serialization support (via serde)

### storage.rs
**Purpose**: File system operations and data persistence

**Key Functions**:

```rust
impl Storage {
    // Initialization
    fn new(base_path: String) -> Self
    fn load_all_notes(&mut self)
    
    // Read operations
    fn load_note(&self, path: &Path) -> io::Result<Note>
    
    // Write operations
    fn create_folder(&mut self, name: &str) -> io::Result<()>
    fn create_note(&mut self, folder_idx: usize, title: &str) -> io::Result<()>
    fn save_note(&mut self, folder_idx: usize, note_idx: usize) -> io::Result<()>
    
    // Sync
    fn export_to_cloud(&self) -> io::Result<String>
}
```

**File Operations**:
- Direct file system access (no database)
- Markdown files (.md) for content
- JSON metadata files (.meta) for timestamps
- Folder-based organization

**Directory Structure**:
```
notes_data/
├── Folder1/
│   ├── note1.md        # Note content
│   ├── note1.meta      # Note metadata (JSON)
│   ├── note2.md
│   └── note2.meta
└── Folder2/
    └── note3.md
```

### search.rs
**Purpose**: Fuzzy search implementation

**Algorithm**: Uses SkimMatcherV2 for fuzzy string matching

**Search Process**:
1. Iterate through all folders
2. For each note, search:
   - Note title
   - Note content
3. Return list of (folder_idx, note_idx) matches

```rust
pub struct FuzzySearch {
    matcher: SkimMatcherV2,
}

impl FuzzySearch {
    pub fn search(&self, folders: &[Folder], query: &str) 
        -> Vec<(usize, usize)>
}
```

**Features**:
- Real-time search (no indexing required)
- Fuzzy matching (tolerates typos)
- Searches both title and content

## Data Flow

### Creating a Note

```
User clicks "New Note"
    ↓
UI shows dialog
    ↓
User enters title
    ↓
UI calls app.create_note()
    ↓
app calls storage.create_note()
    ↓
Storage creates .md and .meta files
    ↓
Storage updates in-memory folder structure
    ↓
UI refreshes to show new note
```

### Editing a Note

```
User selects note from sidebar
    ↓
App loads note content into edit buffer
    ↓
User clicks "Edit" button
    ↓
TextEdit widget becomes editable
    ↓
User modifies content
    ↓
User clicks "Save"
    ↓
App calls storage.save_note()
    ↓
Storage writes content to .md file
    ↓
Storage updates .meta file with new timestamp
    ↓
UI returns to view mode
```

### Searching Notes

```
User types in search bar
    ↓
search_query state updates
    ↓
App calls search.search()
    ↓
Search iterates all notes
    ↓
Fuzzy matcher scores each note
    ↓
Results returned as indices
    ↓
UI displays results in sidebar
    ↓
User clicks result
    ↓
Note opens in editor
```

## Performance Characteristics

### Time Complexity

| Operation | Complexity | Notes |
|-----------|-----------|-------|
| Load all notes | O(n) | n = total notes, done once at startup |
| Create note | O(1) | File system write |
| Save note | O(1) | File system write |
| Search | O(n*m) | n = notes, m = avg content length |
| Open note | O(1) | Direct index access |

### Space Complexity

| Component | Memory Usage |
|-----------|--------------|
| Note metadata | ~200 bytes per note |
| Note content | Size of content (loaded on demand) |
| Folder structure | ~100 bytes per folder |
| Search index | None (real-time search) |

### Optimization Strategies

1. **Lazy Loading**: Only current note content is in memory
2. **No Database**: Direct file system access eliminates query overhead
3. **Efficient Search**: Fuzzy matcher is fast for reasonable note counts
4. **Minimal Copying**: Uses references where possible

## Concurrency Model

```rust
storage: Arc<Mutex<Storage>>
```

- **Arc**: Atomic reference counting for shared ownership
- **Mutex**: Ensures thread-safe access to storage
- Currently single-threaded UI, but prepared for async operations

## File Format Specification

### Note Content (.md)
```markdown
# Any valid Markdown content
## Headings
**Bold** and *italic*
- Lists
1. Numbered lists
`code`
```

### Note Metadata (.meta)
```json
{
  "created_at": "2025-01-15 10:30:00",
  "updated_at": "2025-01-15 14:45:00"
}
```

## Error Handling

### Strategy
- IO operations return `io::Result<T>`
- UI layer handles errors gracefully
- File system errors logged to console

### Recovery
- Missing metadata files: Create default metadata
- Corrupted files: Skip and log error
- Permission errors: Notify user via console

## Future Scalability

### For Large Note Collections (1000+ notes)

1. **Lazy Loading**
   - Load notes on-demand instead of at startup
   - Cache recently accessed notes

2. **Search Indexing**
   - Build inverted index for O(1) search
   - Update index incrementally

3. **Database Migration**
   - SQLite for metadata
   - Keep files for content
   - Improves search and organization

4. **Async I/O**
   - Non-blocking file operations
   - Background saving

### For Multiple Users

1. **Add User Context**
   ```rust
   struct Storage {
       user_id: String,
       base_path: String,
   }
   ```

2. **Isolation**
   - Separate folders per user
   - Permission-based access

3. **Sync Improvements**
   - Real-time sync via WebSockets
   - Conflict resolution
   - Version control

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_note_creation() { }
    
    #[test]
    fn test_search_fuzzy_match() { }
    
    #[test]
    fn test_save_and_load() { }
}
```

### Integration Tests
- Test full note lifecycle
- Test folder operations
- Test search across multiple notes

### UI Tests
- Manual testing required for egui
- Test user workflows end-to-end

## Security Considerations

### Current
- Local file system only
- No authentication
- No encryption

### Recommendations for Production
1. **Encryption at Rest**: Encrypt .md files
2. **Access Control**: User authentication
3. **Input Validation**: Sanitize file names
4. **Backup**: Automatic backup before sync

## Dependencies Rationale

| Crate | Purpose | Why Chosen |
|-------|---------|------------|
| eframe/egui | GUI framework | Cross-platform, immediate mode, pure Rust |
| chrono | Timestamps | De facto standard for date/time in Rust |
| serde/serde_json | Serialization | Standard serialization framework |
| walkdir | Directory traversal | Efficient recursive directory walking |
| fuzzy-matcher | Search | Fast fuzzy string matching |
| rfd | File dialogs | Native file dialogs (for future features) |

## Build Process

```bash
cargo build --release
```

**Optimization Flags** (in Cargo.toml):
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

**Result**: Single binary, no runtime dependencies (except system libs)

## Deployment

### Single Binary Distribution
```bash
target/release/notetaking-app
```

### Platform Support
- Linux: Native
- macOS: Native
- Windows: Native (with .exe extension)

### Installation
1. Copy binary to desired location
2. Run from terminal or create desktop shortcut
3. Data stored in `./notes_data` relative to binary

This architecture provides a solid foundation for a fast, reliable notetaking application while remaining simple enough to understand and modify.
