use crate::Result;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// File sorter that organizes files by extension and HTML categories
pub struct FileSorter {
    source_dir: PathBuf,
    target_dir: PathBuf,
}

/// HTML file categories based on content or naming patterns
#[derive(Debug, Clone, PartialEq)]
pub enum HtmlCategory {
    Index,      // index.html, home.html
    Template,   // template files, layouts
    Component,  // component files
    Page,       // general pages
    Unknown,    // other HTML files
}

impl FileSorter {
    /// Create a new file sorter
    pub fn new<P: AsRef<Path>>(source_dir: P, target_dir: P) -> Self {
        Self {
            source_dir: source_dir.as_ref().to_path_buf(),
            target_dir: target_dir.as_ref().to_path_buf(),
        }
    }

    /// Sort files by extension and HTML categories
    pub fn sort_files(&self) -> Result<SortingReport> {
        let mut report = SortingReport::new();
        
        // Ensure target directory exists
        fs::create_dir_all(&self.target_dir)?;

        // Read all files from source directory
        let entries = fs::read_dir(&self.source_dir)?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                self.sort_single_file(&path, &mut report)?;
            }
        }

        Ok(report)
    }

    /// Sort a single file based on its extension and content
    fn sort_single_file(&self, file_path: &Path, report: &mut SortingReport) -> Result<()> {
        let file_name = file_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let extension = file_path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("no_extension")
            .to_lowercase();

        // Determine target folder based on extension
        let target_folder = if extension == "html" || extension == "htm" {
            // For HTML files, create subcategories
            let category = self.detect_html_category(file_path)?;
            format!("html/{}", category.folder_name())
        } else {
            // For other files, sort by extension
            extension.clone()
        };

        // Create target directory
        let target_dir = self.target_dir.join(&target_folder);
        fs::create_dir_all(&target_dir)?;

        // Move file to target directory
        let target_path = target_dir.join(file_name);
        fs::copy(file_path, &target_path)?;

        // Update report
        report.add_file_moved(extension, target_folder, file_name.to_string());

        println!("Moved {} -> {}", file_path.display(), target_path.display());

        Ok(())
    }

    /// Detect HTML file category based on filename and content
    fn detect_html_category(&self, file_path: &Path) -> Result<HtmlCategory> {
        let file_name = file_path.file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Check filename patterns first
        let category = match file_name.as_str() {
            "index" | "home" | "main" => HtmlCategory::Index,
            name if name.contains("template") || name.contains("layout") => HtmlCategory::Template,
            name if name.contains("component") || name.starts_with("comp_") => HtmlCategory::Component,
            _ => {
                // Check content for more sophisticated categorization
                self.detect_html_category_by_content(file_path)?
            }
        };

        Ok(category)
    }

    /// Detect HTML category by analyzing file content
    fn detect_html_category_by_content(&self, file_path: &Path) -> Result<HtmlCategory> {
        let content = fs::read_to_string(file_path).unwrap_or_default().to_lowercase();

        if content.contains("template") || content.contains("layout") {
            Ok(HtmlCategory::Template)
        } else if content.contains("component") || content.contains("widget") {
            Ok(HtmlCategory::Component)
        } else if content.contains("<html") && content.contains("<head") {
            Ok(HtmlCategory::Page)
        } else {
            Ok(HtmlCategory::Unknown)
        }
    }
}

impl HtmlCategory {
    /// Get the folder name for this category
    pub fn folder_name(&self) -> &'static str {
        match self {
            HtmlCategory::Index => "index",
            HtmlCategory::Template => "templates",
            HtmlCategory::Component => "components",
            HtmlCategory::Page => "pages",
            HtmlCategory::Unknown => "other",
        }
    }
}

/// Report of the sorting operation
#[derive(Debug, Default)]
pub struct SortingReport {
    pub files_moved: HashMap<String, Vec<String>>,
    pub total_files: usize,
}

impl SortingReport {
    pub fn new() -> Self {
        Self {
            files_moved: HashMap::new(),
            total_files: 0,
        }
    }

    pub fn add_file_moved(&mut self, extension: String, folder: String, filename: String) {
        let key = format!("{} -> {}", extension, folder);
        self.files_moved.entry(key)
            .or_default()
            .push(filename);
        self.total_files += 1;
    }

    pub fn print_summary(&self) {
        println!("\n=== File Sorting Summary ===");
        println!("Total files moved: {}", self.total_files);
        println!();

        for (category, files) in &self.files_moved {
            println!("{}: {} files", category, files.len());
            for file in files {
                println!("  - {}", file);
            }
            println!();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_html_category_detection() {
        assert_eq!(HtmlCategory::Index.folder_name(), "index");
        assert_eq!(HtmlCategory::Template.folder_name(), "templates");
        assert_eq!(HtmlCategory::Component.folder_name(), "components");
        assert_eq!(HtmlCategory::Page.folder_name(), "pages");
        assert_eq!(HtmlCategory::Unknown.folder_name(), "other");
    }

    #[test]
    fn test_file_sorter_creation() {
        let source = tempdir().unwrap();
        let target = tempdir().unwrap();
        
        let sorter = FileSorter::new(source.path(), target.path());
        assert_eq!(sorter.source_dir, source.path());
        assert_eq!(sorter.target_dir, target.path());
    }

    #[test]
    fn test_sorting_report() {
        let mut report = SortingReport::new();
        report.add_file_moved("txt".to_string(), "txt".to_string(), "test.txt".to_string());
        
        assert_eq!(report.total_files, 1);
        assert!(report.files_moved.contains_key("txt -> txt"));
    }
}