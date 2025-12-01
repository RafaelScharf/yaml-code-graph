// crates/ycg_core/src/file_filter.rs

use crate::model::FileFilterConfig;
use crate::scip_proto;
use anyhow::{Context, Result};
use std::path::Path;

/// File filter that applies include/exclude patterns and gitignore rules
pub struct FileFilter {
    include_patterns: Vec<glob::Pattern>,
    exclude_patterns: Vec<glob::Pattern>,
    gitignore_matcher: Option<GitignoreMatcher>,
}

impl FileFilter {
    /// Create a new FileFilter from configuration
    pub fn new(config: &FileFilterConfig, project_root: &Path) -> Result<Self> {
        // Compile include patterns
        let include_patterns = config
            .include_patterns
            .iter()
            .map(|pattern| {
                glob::Pattern::new(pattern)
                    .with_context(|| format!("Invalid include pattern: {}", pattern))
            })
            .collect::<Result<Vec<_>>>()?;

        // Compile exclude patterns
        let exclude_patterns = config
            .exclude_patterns
            .iter()
            .map(|pattern| {
                glob::Pattern::new(pattern)
                    .with_context(|| format!("Invalid exclude pattern: {}", pattern))
            })
            .collect::<Result<Vec<_>>>()?;

        // Initialize gitignore matcher if enabled
        let gitignore_matcher = if config.use_gitignore {
            GitignoreMatcher::new(project_root).ok()
        } else {
            None
        };

        Ok(FileFilter {
            include_patterns,
            exclude_patterns,
            gitignore_matcher,
        })
    }

    /// Check if a file should be processed based on filtering rules
    ///
    /// Rules (in order):
    /// 1. If include patterns exist, file must match at least one
    /// 2. If file matches any exclude pattern, it's excluded
    /// 3. If gitignore is enabled and file is ignored, it's excluded
    /// 4. Otherwise, file is included
    pub fn should_process(&self, file_path: &Path) -> bool {
        let path_str = file_path.to_string_lossy();

        // Rule 1: Apply include patterns first (if any exist)
        if !self.include_patterns.is_empty() {
            let matches_include = self
                .include_patterns
                .iter()
                .any(|pattern| pattern.matches(&path_str));

            if !matches_include {
                return false;
            }
        }

        // Rule 2: Apply exclude patterns (exclude wins over include)
        if self
            .exclude_patterns
            .iter()
            .any(|pattern| pattern.matches(&path_str))
        {
            return false;
        }

        // Rule 3: Check gitignore
        if let Some(ref matcher) = self.gitignore_matcher {
            if matcher.is_ignored(file_path) {
                return false;
            }
        }

        // Default: include the file
        true
    }

    /// Filter a list of SCIP documents based on filtering rules
    pub fn filter_documents(
        &self,
        documents: Vec<scip_proto::Document>,
    ) -> Vec<scip_proto::Document> {
        documents
            .into_iter()
            .filter(|doc| {
                let path = Path::new(&doc.relative_path);
                self.should_process(path)
            })
            .collect()
    }
}

/// Wrapper around gitignore crate for matching ignored files
struct GitignoreMatcher {
    gitignore: ignore::gitignore::Gitignore,
}

impl GitignoreMatcher {
    /// Create a new GitignoreMatcher by loading .gitignore from project root
    fn new(project_root: &Path) -> Result<Self> {
        let gitignore_path = project_root.join(".gitignore");

        let mut builder = ignore::gitignore::GitignoreBuilder::new(project_root);
        builder.add(&gitignore_path);

        // Build returns Result, but we don't fail if .gitignore doesn't exist
        let gitignore = match builder.build() {
            Ok(gi) => gi,
            Err(e) => {
                eprintln!("Warning: Error loading .gitignore: {}", e);
                // Return an empty gitignore matcher
                ignore::gitignore::GitignoreBuilder::new(project_root).build()?
            }
        };

        Ok(GitignoreMatcher { gitignore })
    }

    /// Check if a file path is ignored by gitignore rules
    fn is_ignored(&self, path: &Path) -> bool {
        // Check if the path matches any gitignore pattern
        // We assume path is relative to project root
        let is_dir = false; // We're checking file paths from SCIP documents
        self.gitignore.matched(path, is_dir).is_ignore()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_no_patterns_includes_all() {
        let config = FileFilterConfig {
            include_patterns: vec![],
            exclude_patterns: vec![],
            use_gitignore: false,
        };

        let temp_dir = TempDir::new().unwrap();
        let filter = FileFilter::new(&config, temp_dir.path()).unwrap();

        assert!(filter.should_process(Path::new("src/main.rs")));
        assert!(filter.should_process(Path::new("tests/test.rs")));
        assert!(filter.should_process(Path::new("README.md")));
    }

    #[test]
    fn test_include_pattern_filters() {
        let config = FileFilterConfig {
            include_patterns: vec!["**/*.rs".to_string()],
            exclude_patterns: vec![],
            use_gitignore: false,
        };

        let temp_dir = TempDir::new().unwrap();
        let filter = FileFilter::new(&config, temp_dir.path()).unwrap();

        assert!(filter.should_process(Path::new("src/main.rs")));
        assert!(filter.should_process(Path::new("tests/test.rs")));
        assert!(!filter.should_process(Path::new("README.md")));
        assert!(!filter.should_process(Path::new("package.json")));
    }

    #[test]
    fn test_exclude_pattern_filters() {
        let config = FileFilterConfig {
            include_patterns: vec![],
            exclude_patterns: vec!["**/target/**".to_string(), "**/node_modules/**".to_string()],
            use_gitignore: false,
        };

        let temp_dir = TempDir::new().unwrap();
        let filter = FileFilter::new(&config, temp_dir.path()).unwrap();

        assert!(filter.should_process(Path::new("src/main.rs")));
        assert!(!filter.should_process(Path::new("target/debug/main")));
        assert!(!filter.should_process(Path::new("node_modules/package/index.js")));
    }

    #[test]
    fn test_exclude_wins_over_include() {
        let config = FileFilterConfig {
            include_patterns: vec!["**/*.rs".to_string()],
            exclude_patterns: vec!["**/target/**".to_string()],
            use_gitignore: false,
        };

        let temp_dir = TempDir::new().unwrap();
        let filter = FileFilter::new(&config, temp_dir.path()).unwrap();

        assert!(filter.should_process(Path::new("src/main.rs")));
        assert!(!filter.should_process(Path::new("target/debug/main.rs")));
    }

    #[test]
    fn test_gitignore_integration() {
        let temp_dir = TempDir::new().unwrap();
        let gitignore_path = temp_dir.path().join(".gitignore");

        // Create a .gitignore file with patterns
        fs::write(&gitignore_path, "*.log\nnode_modules/\n").unwrap();

        let config = FileFilterConfig {
            include_patterns: vec![],
            exclude_patterns: vec![],
            use_gitignore: true,
        };

        let filter = FileFilter::new(&config, temp_dir.path()).unwrap();

        // Test that normal files are included
        assert!(filter.should_process(Path::new("src/main.rs")));

        // Test that gitignored file patterns are excluded
        assert!(!filter.should_process(Path::new("debug.log")));
        assert!(!filter.should_process(Path::new("app.log")));
    }

    #[test]
    fn test_filter_documents() {
        let config = FileFilterConfig {
            include_patterns: vec!["**/*.rs".to_string()],
            exclude_patterns: vec![],
            use_gitignore: false,
        };

        let temp_dir = TempDir::new().unwrap();
        let filter = FileFilter::new(&config, temp_dir.path()).unwrap();

        let documents = vec![
            scip_proto::Document {
                relative_path: "src/main.rs".to_string(),
                ..Default::default()
            },
            scip_proto::Document {
                relative_path: "README.md".to_string(),
                ..Default::default()
            },
            scip_proto::Document {
                relative_path: "tests/test.rs".to_string(),
                ..Default::default()
            },
        ];

        let filtered = filter.filter_documents(documents);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].relative_path, "src/main.rs");
        assert_eq!(filtered[1].relative_path, "tests/test.rs");
    }
}
