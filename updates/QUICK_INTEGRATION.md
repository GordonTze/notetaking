# Quick Start: Adding Enhanced Features

This guide shows how to quickly add the most useful features to your existing app.

## Option 1: Add Just Dark Mode (Easiest - 5 minutes)

### Step 1: Add module declaration

At the top of `src/main.rs`, add:
```rust
mod theme;
```

### Step 2: Add theme manager to app state

In the `NoteTakingApp` struct:
```rust
struct NoteTakingApp {
    // ... existing fields ...
    theme_manager: ThemeManager,
}
```

### Step 3: Initialize in `new()`

```rust
fn new(cc: &eframe::CreationContext<'_>) -> Self {
    let theme_manager = ThemeManager::new();
    theme_manager.current_theme.apply_to_egui(&cc.egui_ctx);
    
    Self {
        // ... existing fields ...
        theme_manager,
    }
}
```

### Step 4: Add toggle button in top panel

```rust
if ui.button("🌙 Dark Mode").clicked() {
    self.theme_manager.toggle_dark_mode();
}
```

### Step 5: Apply theme each frame

In `update()` method:
```rust
self.theme_manager.current_theme.apply_to_egui(ctx);
```

**Done!** You now have dark mode toggle.

---

## Option 2: Add Tags (Medium - 10 minutes)

### Step 1: Add modules

```rust
mod theme;
mod tags;
```

### Step 2: Update app state

```rust
struct NoteTakingApp {
    // ... existing ...
    tag_manager: TagManager,
    new_tag_name: String,
    show_tag_dialog: bool,
}
```

### Step 3: Initialize

```rust
fn new(cc: &eframe::CreationContext<'_>) -> Self {
    Self {
        // ...
        tag_manager: TagManager::new(),
        new_tag_name: String::new(),
        show_tag_dialog: false,
    }
}
```

### Step 4: Add button

```rust
if ui.button("🏷️ Tags").clicked() {
    self.show_tag_dialog = true;
}
```

### Step 5: Add dialog

```rust
if self.show_tag_dialog {
    egui::Window::new("Tags")
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.new_tag_name);
                if ui.button("Add").clicked() {
                    self.tag_manager.add_tag(self.new_tag_name.clone());
                    self.new_tag_name.clear();
                }
            });
            
            // Show all tags
            for tag in self.tag_manager.all_tags() {
                ui.colored_label(
                    egui::Color32::from_rgb(tag.color[0], tag.color[1], tag.color[2]),
                    &tag.name
                );
            }
        });
}
```

**Done!** You can now create and view tags.

---

## Option 3: Add PDF Export (Medium - 10 minutes)

### Step 1: Add module

```rust
mod pdf_export;
```

### Step 2: Add export method

```rust
fn export_current_note_to_pdf(&self) {
    if let (Some(folder_idx), Some(note_idx)) = (self.selected_folder, self.selected_note) {
        let storage = self.storage.lock().unwrap();
        if let Some(folder) = storage.folders.get(folder_idx) {
            if let Some(note) = folder.notes.get(note_idx) {
                let output_path = std::path::PathBuf::from(format!("{}.pdf", note.title));
                match pdf_export::PdfExporter::export_note(&note.title, &note.content, &output_path) {
                    Ok(_) => println!("✓ Exported to: {:?}", output_path),
                    Err(e) => eprintln!("✗ Export failed: {}", e),
                }
            }
        }
    }
}
```

### Step 3: Add button

```rust
if ui.button("📄 Export PDF").clicked() {
    self.export_current_note_to_pdf();
}
```

**Done!** You can now export notes to PDF.

---

## Option 4: Add Encryption (Advanced - 15 minutes)

### Step 1: Add module

```rust
mod encryption;
```

### Step 2: Update app state

```rust
struct NoteTakingApp {
    // ...
    encryption: Encryption,
    show_encryption_dialog: bool,
    encryption_password: String,
}
```

### Step 3: Initialize

```rust
fn new(_cc: &eframe::CreationContext<'_>) -> Self {
    Self {
        // ...
        encryption: Encryption::new(),
        show_encryption_dialog: false,
        encryption_password: String::new(),
    }
}
```

### Step 4: Add encrypt/decrypt method

```rust
fn toggle_encryption(&mut self) {
    if let (Some(folder_idx), Some(note_idx)) = (self.selected_folder, self.selected_note) {
        let mut storage = self.storage.lock().unwrap();
        if let Some(folder) = storage.folders.get_mut(folder_idx) {
            if let Some(note) = folder.notes.get_mut(note_idx) {
                if note.is_encrypted {
                    // Decrypt
                    if let Some(ref enc_data) = note.encrypted_data {
                        match self.encryption.decrypt(enc_data, &self.encryption_password) {
                            Ok(decrypted) => {
                                note.content = decrypted;
                                note.is_encrypted = false;
                                note.encrypted_data = None;
                                self.current_note_content = note.content.clone();
                                println!("✓ Decrypted");
                            }
                            Err(e) => eprintln!("✗ Decryption failed: {}", e),
                        }
                    }
                } else {
                    // Encrypt
                    match self.encryption.encrypt(&note.content, &self.encryption_password) {
                        Ok(encrypted) => {
                            note.encrypted_data = Some(encrypted);
                            note.is_encrypted = true;
                            note.content = "[ENCRYPTED]".to_string();
                            self.current_note_content = note.content.clone();
                            println!("✓ Encrypted");
                        }
                        Err(e) => eprintln!("✗ Encryption failed: {}", e),
                    }
                }
                storage.save_note(folder_idx, note_idx).ok();
            }
        }
    }
    self.encryption_password.clear();
    self.show_encryption_dialog = false;
}
```

### Step 5: Add UI

```rust
// Button
if ui.button("🔐 Encrypt").clicked() {
    self.show_encryption_dialog = true;
}

// Dialog
if self.show_encryption_dialog {
    egui::Window::new("Encrypt Note")
        .show(ctx, |ui| {
            ui.label("Password:");
            ui.add(egui::TextEdit::singleline(&mut self.encryption_password)
                .password(true));
            
            if ui.button("Encrypt/Decrypt").clicked() {
                self.toggle_encryption();
            }
        });
}
```

**Done!** You can now encrypt/decrypt notes.

---

## Full Integration: All Features

For a complete integration with all features:

1. **Copy all module files** to `src/`
2. **Update Cargo.toml** with new dependencies
3. **Replace main.rs** with enhanced version
4. **Run** `cargo build --release`

### Quick Commands:

```bash
# 1. Copy new modules (already done)
ls src/*.rs

# 2. Dependencies already updated in Cargo.toml

# 3. Build
cargo build --release

# 4. Run
cargo run --release
```

---

## Feature Priority Recommendation

If adding features incrementally, recommended order:

1. ✅ **Dark Mode** (5 min) - Best UX improvement
2. ✅ **Tags** (10 min) - Most useful for organization
3. ✅ **PDF Export** (10 min) - Most requested feature
4. ✅ **Encryption** (15 min) - Security conscious users
5. ⏭️ **Images** (20 min) - Visual note takers
6. ⏭️ **Links** (20 min) - Power users
7. ⏭️ **Version History** (30 min) - Advanced users

---

## Testing Each Feature

### Dark Mode
1. Click "🌙 Dark Mode" button
2. Interface should darken
3. Click again to return to light

### Tags
1. Click "🏷️ Tags"
2. Type "Important" and click "Add"
3. Tag appears in dialog

### PDF Export
1. Select a note
2. Click "📄 Export PDF"
3. Check for .pdf file in directory

### Encryption
1. Select a note
2. Click "🔐 Encrypt"
3. Enter password
4. Note content becomes "[ENCRYPTED]"
5. Click "🔐 Encrypt" again
6. Enter same password
7. Note content is restored

---

## Minimal Working Example

Here's a minimal main.rs with just dark mode:

```rust
use eframe::egui;
use std::sync::Arc;
use std::sync::Mutex;

mod note;
mod storage;
mod search;
mod theme;

use note::Folder;
use storage::Storage;
use search::FuzzySearch;
use theme::ThemeManager;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Notetaking App",
        options,
        Box::new(|cc| Ok(Box::new(NoteTakingApp::new(cc)))),
    )
}

struct NoteTakingApp {
    storage: Arc<Mutex<Storage>>,
    theme_manager: ThemeManager,
    // ... rest of existing fields ...
}

impl NoteTakingApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let theme_manager = ThemeManager::new();
        theme_manager.current_theme.apply_to_egui(&cc.egui_ctx);
        
        Self {
            storage: Arc::new(Mutex::new(Storage::new("./notes_data".to_string()))),
            theme_manager,
            // ... rest of fields ...
        }
    }
}

impl eframe::App for NoteTakingApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme every frame
        self.theme_manager.current_theme.apply_to_egui(ctx);
        
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("🌙 Toggle Dark Mode").clicked() {
                    self.theme_manager.toggle_dark_mode();
                }
                // ... rest of top panel ...
            });
        });
        
        // ... rest of UI ...
    }
}
```

---

## Common Issues

**"Cannot find module 'theme'"**
→ Make sure `theme.rs` is in `src/` directory

**"Unresolved import"**
→ Check `use` statements match module names

**"Field not found"**
→ Add missing fields to struct initialization

**Compilation takes forever**
→ Normal for first build with new dependencies

**"feature not enabled"**
→ Check Cargo.toml has correct feature flags

---

## Next Steps

Once you've added the basic features:

1. Customize themes in `theme.rs`
2. Add more tag colors
3. Enhance PDF formatting
4. Add auto-save for encrypted notes
5. Implement cloud sync

See `ENHANCED_FEATURES.md` for detailed documentation on each feature.

---

**Quick Reference**:
- 🌙 Dark Mode: `mod theme;` + toggle button
- 🏷️ Tags: `mod tags;` + dialog
- 📄 PDF: `mod pdf_export;` + export method
- 🔐 Encrypt: `mod encryption;` + password dialog
- All features: Copy all modules + build

Happy coding! 🚀
