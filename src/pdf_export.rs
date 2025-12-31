use printpdf::*;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

pub struct PdfExporter;

impl PdfExporter {
    pub fn export_note(
        title: &str,
        content: &str,
        output_path: &Path,
    ) -> Result<(), String> {
        // Create PDF document
        let (doc, page1, layer1) = PdfDocument::new(
            title,
            Mm(210.0),  // A4 width
            Mm(297.0),  // A4 height
            "Layer 1"
        );
        
        let current_layer = doc.get_page(page1).get_layer(layer1);
        
        // Load font
        let font = doc.add_builtin_font(BuiltinFont::Helvetica)
            .map_err(|e| format!("Font error: {}", e))?;
        
        // Title
        current_layer.use_text(title, 24.0, Mm(20.0), Mm(270.0), &font);
        
        // Content - split into lines
        let mut y_position = 250.0;
        let line_height = 5.0;
        
        for line in content.lines() {
            if y_position < 20.0 {
                // Need new page
                break; // Simplified version - full version would add pages
            }
            
            current_layer.use_text(line, 12.0, Mm(20.0), Mm(y_position), &font);
            y_position -= line_height;
        }
        
        // Save
        let file = File::create(output_path)
            .map_err(|e| format!("File creation error: {}", e))?;
        
        doc.save(&mut BufWriter::new(file))
            .map_err(|e| format!("PDF save error: {}", e))?;
        
        Ok(())
    }
    
    pub fn export_multiple_notes(
        notes: &[(String, String)], // (title, content)
        output_path: &Path,
    ) -> Result<(), String> {
        let (doc, page1, layer1) = PdfDocument::new(
            "Notes Collection",
            Mm(210.0),
            Mm(297.0),
            "Layer 1"
        );
        
        let font = doc.add_builtin_font(BuiltinFont::Helvetica)
            .map_err(|e| format!("Font error: {}", e))?;
        
        let mut current_page = page1;
        let mut y_position = 270.0;
        
        for (title, content) in notes {
            let current_layer = doc.get_page(current_page).get_layer(layer1);
            
            // Check if we need a new page
            if y_position < 30.0 {
                let (new_page, _) = doc.add_page(Mm(210.0), Mm(297.0), "Layer 1");
                current_page = new_page;
                y_position = 270.0;
            }
            
            // Write title
            current_layer.use_text(title, 16.0, Mm(20.0), Mm(y_position), &font);
            y_position -= 10.0;
            
            // Write content (simplified)
            for line in content.lines().take(20) {
                if y_position < 20.0 {
                    break;
                }
                current_layer.use_text(line, 10.0, Mm(20.0), Mm(y_position), &font);
                y_position -= 5.0;
            }
            
            y_position -= 10.0; // Space between notes
        }
        
        let file = File::create(output_path)
            .map_err(|e| format!("File creation error: {}", e))?;
        
        doc.save(&mut BufWriter::new(file))
            .map_err(|e| format!("PDF save error: {}", e))?;
        
        Ok(())
    }
}
