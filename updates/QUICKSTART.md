# Quick Start Guide

## First Time Setup

### 1. Build the Application
```bash
chmod +x build.sh
./build.sh
```

Or manually:
```bash
cargo build --release
```

### 2. Run the Application
```bash
cargo run --release
```

Or use the compiled binary:
```bash
./target/release/notetaking-app
```

## Common Workflows

### Creating Your First Note

1. **Launch the application**
   ```bash
   ./target/release/notetaking-app
   ```

2. **Create a folder**
   - Click "📁 New Folder" button
   - Enter folder name (e.g., "Work", "Personal", "Projects")
   - Click "Create"

3. **Create a note**
   - Select the folder you just created from the sidebar
   - Click "📝 New Note" button
   - Enter note title (e.g., "Meeting Notes", "Todo List")
   - Click "Create"

4. **Edit the note**
   - Click on the note in the sidebar
   - Click "✏ Edit" button
   - Type your content (supports Markdown)
   - Click "💾 Save" when done

### Organizing Notes

#### Folder Structure Example
```
Work/
  - Project A Notes
  - Meeting Notes
  - Todo List
  
Personal/
  - Shopping List
  - Book Ideas
  - Journal Entries
  
Learning/
  - Rust Tutorial Notes
  - Design Patterns
  - Code Snippets
```

### Using Markdown

The app supports Markdown formatting:

```markdown
# Heading 1
## Heading 2
### Heading 3

**Bold text**
*Italic text*

- Bullet point 1
- Bullet point 2

1. Numbered item 1
2. Numbered item 2

`code inline`

```
code block
```

[Link text](https://example.com)
```

### Searching Notes

1. **Quick search**
   - Type in the search bar at the top
   - Results appear instantly as you type
   - Click on any result to open that note

2. **Search tips**
   - Fuzzy matching means you don't need exact spelling
   - Search looks in both note titles and content
   - Works across all folders

### Syncing to Cloud

The app doesn't automatically sync, but provides a manual sync feature:

1. **Create sync backup**
   - Click "☁ Sync to Cloud" button
   - A folder `notes_data_cloud_sync` is created
   
2. **Upload to cloud**
   - Open your cloud storage (Google Drive, Dropbox, etc.)
   - Upload the entire `notes_data_cloud_sync` folder
   - You now have a cloud backup!

3. **Restore from cloud**
   - Download your cloud backup
   - Replace the `notes_data` folder with the downloaded folder
   - Restart the application

## Keyboard Shortcuts

- **Text editing**: Standard shortcuts work (Ctrl+C, Ctrl+V, Ctrl+Z, etc.)
- **Save note**: Click "💾 Save" button
- **Cancel edit**: Click "❌ Cancel" button

## Tips & Tricks

### 1. Use Descriptive Titles
Good: "Project Alpha - Sprint Planning 2025-01-15"
Bad: "Notes"

### 2. Organize by Context
Create folders for different areas of your life:
- Work
- Personal
- Learning
- Projects
- Ideas

### 3. Use Markdown for Structure
- Use headings to organize long notes
- Use lists for todos and bullet points
- Use code blocks for snippets

### 4. Regular Syncing
- Sync to cloud weekly (or after important notes)
- Keeps your notes backed up
- Allows access from multiple devices

### 5. Search Everything
- Don't worry about perfect organization
- Use search to find anything quickly
- Fuzzy search is forgiving of typos

## Advanced Usage

### Direct File Access
Your notes are stored as plain text files in `notes_data/`:

```bash
ls notes_data/
# Shows all folders

cat notes_data/Work/meeting-notes.md
# Shows note content

vim notes_data/Work/meeting-notes.md
# Edit directly with your favorite editor
```

### Backup Strategy
```bash
# Manual backup
cp -r notes_data notes_data_backup_$(date +%Y%m%d)

# Automated backup (add to crontab)
0 0 * * 0 cp -r ~/notes_data ~/notes_backup_$(date +\%Y\%m\%d)
```

### Syncing with Git
```bash
cd notes_data
git init
git add .
git commit -m "Initial notes"
git remote add origin YOUR_REPO_URL
git push -u origin main
```

### Import Existing Notes
```bash
# Place markdown files in folder structure
mkdir -p notes_data/ImportedNotes
cp ~/my-old-notes/*.md notes_data/ImportedNotes/
# Restart app to see imported notes
```

## Troubleshooting

### App Won't Start
```bash
# Check Rust installation
cargo --version
rustc --version

# Rebuild
cargo clean
cargo build --release
```

### Notes Not Showing
```bash
# Check directory structure
ls -la notes_data/

# Check permissions
chmod -R u+rw notes_data/
```

### Search Not Working
- Restart the application
- Verify notes have content
- Check that search query is not empty

## Example Note Template

Here's a template for meeting notes:

```markdown
# Meeting: [Topic]
Date: 2025-01-15
Attendees: [Names]

## Agenda
1. [Topic 1]
2. [Topic 2]
3. [Topic 3]

## Discussion
- [Key point 1]
- [Key point 2]
- [Decision made]

## Action Items
- [ ] [Task 1] - [Person] - [Due date]
- [ ] [Task 2] - [Person] - [Due date]

## Next Steps
[What happens next]

## Notes
[Additional context]
```

## Getting Help

- Check README.md for detailed documentation
- Open an issue for bugs or feature requests
- Review the code in `src/` for technical details

## Next Steps

Now that you're familiar with the basics:
1. Create your folder structure
2. Add some notes
3. Try the search feature
4. Set up regular cloud syncing
5. Explore Markdown formatting

Happy note-taking! 📝
