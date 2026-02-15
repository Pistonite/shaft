use std::path::{Path, PathBuf};

/// A parsed INI file that preserves comments and structure when saving.
pub struct IniFile {
    path: PathBuf,
    sections: Vec<IniSection>,
}

/// A section in an INI file. Lines before the first `[header]` are stored
/// in a section with `name: None`.
pub struct IniSection {
    /// Section name, or `None` for lines before the first header.
    name: Option<String>,
    lines: Vec<Line>,
}

enum Line {
    /// A comment or blank line (preserved as-is)
    Other(String),
    /// A key=value pair
    KeyValue { key: String, value: String },
}

impl IniFile {
    /// Open and parse an INI file. Returns an error if the file does not exist.
    #[inline(always)]
    pub fn open(path: impl AsRef<Path>) -> cu::Result<Self> {
        Self::open_impl(path.as_ref())
    }

    fn open_impl(path: &Path) -> cu::Result<Self> {
        let content = cu::fs::read_string(path)?;
        let mut sections = Vec::new();
        // start with an unnamed section for any lines before the first header
        let mut current = IniSection {
            name: None,
            lines: Vec::new(),
        };

        for raw in content.lines() {
            let trimmed = raw.trim();
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                let name = trimmed[1..trimmed.len() - 1].to_string();
                sections.push(current);
                current = IniSection {
                    name: Some(name),
                    lines: Vec::new(),
                };
            } else if let Some(eq) = trimmed.find('=') {
                let is_comment = trimmed.starts_with('#') || trimmed.starts_with(';');
                if is_comment {
                    current.lines.push(Line::Other(raw.to_string()));
                } else {
                    let key = trimmed[..eq].trim_end().to_string();
                    let value = trimmed[eq + 1..].trim_start().to_string();
                    current.lines.push(Line::KeyValue { key, value });
                }
            } else {
                current.lines.push(Line::Other(raw.to_string()));
            }
        }
        sections.push(current);

        Ok(Self {
            path: path.to_path_buf(),
            sections,
        })
    }

    pub fn set_path(&mut self, path: impl AsRef<Path>) {
        self.path = path.as_ref().to_path_buf();
    }

    /// Write the INI file back to disk.
    pub fn write(&self) -> cu::Result<()> {
        cu::fs::write(&self.path, self.to_string())
    }

    /// Get an immutable reference to a section. Returns `None` if the section does not exist.
    pub fn section(&self, name: &str) -> Option<&IniSection> {
        self.sections
            .iter()
            .find(|s| s.name.as_deref() == Some(name))
    }

    /// Get a mutable reference to a section, creating it if it does not exist.
    pub fn section_mut(&mut self, name: &str) -> &mut IniSection {
        let idx = self
            .sections
            .iter()
            .position(|s| s.name.as_deref() == Some(name));
        match idx {
            Some(i) => &mut self.sections[i],
            None => {
                self.sections.push(IniSection {
                    name: Some(name.to_string()),
                    lines: Vec::new(),
                });
                self.sections.last_mut().unwrap()
            }
        }
    }
}

impl std::fmt::Display for IniFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for section in &self.sections {
            if let Some(name) = &section.name {
                writeln!(f, "[{name}]")?;
            }
            for line in &section.lines {
                match line {
                    Line::Other(s) => writeln!(f, "{s}")?,
                    Line::KeyValue { key, value } => writeln!(f, "{key}={value}")?,
                }
            }
        }
        Ok(())
    }
}

impl IniSection {
    /// Get the value associated with a key in this section.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.lines.iter().find_map(|l| match l {
            Line::KeyValue { key: k, value } if k == key => Some(value.as_str()),
            _ => None,
        })
    }

    /// Set a key-value pair in this section.
    /// If the key already exists, its value is updated. Otherwise, a new entry is appended.
    pub fn set(&mut self, key: &str, value: impl Into<String>) {
        let value = value.into();
        for line in &mut self.lines {
            if let Line::KeyValue { key: k, value: v } = line {
                if k == key {
                    *v = value;
                    return;
                }
            }
        }
        self.lines.push(Line::KeyValue {
            key: key.to_string(),
            value,
        });
    }
}
