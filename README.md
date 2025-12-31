# Notetaking App

A fast, efficient, and organized notetaking application built in Rust with a simple desktop UI.

## Features

### Core Features
- **⚡ Performance**: Notes appear instantly with fast loading and editing
- **📁 Organization**: Hierarchical folder structure with nested notes
- **✏️ Editing**: Plain text and Markdown support
- **🔍 Search**: Fuzzy search across all notes and folders

### Data Model & Structure
- **Format**: Plain text files with Markdown support (`.md` files)
- **Metadata**: Automatic tracking of created and updated timestamps
- **Storage**: Local file system with metadata stored separately

### Storage & Sync
- **Local Storage**: Notes stored in `./notes_data` directory
- **Folder Structure**: Organized by folders on the file system
- **Cloud Sync**: Manual sync to cloud (creates backup folder for upload)

### User Interface
- **Simple UI**: Clean and minimal interface
- **Sidebar Toggle**: Open/close button to show/hide folders and notes
- **Editor**: Full-screen editing with save/cancel options
- **Search Bar**: Real-time fuzzy search

## Installation

### Prerequisites
- Rust toolchain (rustc + cargo)
- Operating System: Linux, macOS, or Windows

### Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build the Application
```bash
cd notetaking-app
cargo build --release
```

The compiled binary will be in `target/release/notetaking-app`

## Usage

### Running the Application
```bash
cargo run --release
```

Or run the compiled binary:
```bash
./target/release/notetaking-app
```

### Creating Folders and Notes
1. Click "📁 New Folder" to create a new folder
2. Select a folder from the sidebar
3. Click "📝 New Note" to create a new note in the selected folder
4. Edit the note content and click "💾 Save"

### Searching Notes
1. Type in the search bar at the top
2. Results appear instantly in the sidebar
3. Click on any result to view the note

### Syncing to Cloud
1. Click "☁ Sync to Cloud" button
2. A backup folder `notes_data_cloud_sync` will be created
3. Manually upload this folder to your cloud storage service (Google Drive, Dropbox, etc.)

### Keyboard Navigation
- The UI is designed for efficient mouse/trackpad navigation
- Text editing supports standard keyboard shortcuts

## File Structure

```
notes_data/
├── Folder1/
│   ├── note1.md
│   ├── note1.meta
│   ├── note2.md
│   └── note2.meta
├── Folder2/
│   ├── note3.md
│   └── note3.meta
└── ...
```

### File Formats
- `.md` files: Markdown-formatted note content
- `.meta` files: JSON metadata (timestamps)

### Metadata Format
```json
{
  "created_at": "2025-01-15 10:30:00",
  "updated_at": "2025-01-15 14:45:00"
}
```

## Architecture

### Modules

#### `main.rs`
- Main application entry point
- UI rendering with egui/eframe
- Event handling and state management

#### `note.rs`
- Data structures for Notes and Folders
- Timestamp management
- Serialization/deserialization

#### `storage.rs`
- File system operations
- Loading and saving notes
- Folder management
- Cloud sync export

#### `search.rs`
- Fuzzy search implementation
- Title and content matching
- Result ranking

## Dependencies

- **eframe/egui**: Cross-platform GUI framework
- **chrono**: Date and time handling
- **serde/serde_json**: Serialization
- **walkdir**: Directory traversal
- **fuzzy-matcher**: Fuzzy search algorithm
- **rfd**: File dialogs (for future features)

## Performance

- Notes load instantly on selection
- Search is real-time with fuzzy matching
- No database overhead - direct file system access
- Minimal memory footprint

## Future Enhancements

Potential features for future versions:
- [ ] Rich text formatting preview
- [ ] Image embedding
- [ ] Tags and labels
- [ ] Export to PDF
- [ ] Encryption for sensitive notes
- [ ] Automatic cloud sync (Google Drive, Dropbox API)
- [ ] Note linking and backlinks
- [ ] Version history
- [ ] Dark mode toggle
- [ ] Customizable themes

## Troubleshooting

### Notes not appearing
- Check that `notes_data` directory exists
- Verify file permissions
- Ensure `.md` files are properly formatted

### Search not working
- Verify search query is not empty
- Check that notes contain searchable content
- Restart application if search seems stuck

### Sync issues
- Ensure write permissions in application directory
- Check disk space for sync folder creation
- Manually verify `notes_data_cloud_sync` folder contents

## License

This project is available for personal and commercial use.

## Contributing

Contributions welcome! Please feel free to submit issues or pull requests.

## Contact

For questions or feedback, please open an issue on the project repository.
