//! Custom-mission pack metadata.
//!
//! Mirrors the `details.json` files written next to each mod zip under
//! `datadirs/mods/<slug>/details.json`. The on-disk format is produced by
//! the rhmods.com scraper and is consumed here so the game can list and
//! install community missions.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Top-level metadata for one custom mission mod.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModDetails {
    pub slug: String,
    pub title: String,
    pub page_url: String,
    pub author: String,
    pub map: String,
    /// Free-form date string as displayed on rhmods.com (e.g. `"Feb 12, 2026"`).
    pub uploaded: String,
    /// `"Vanilla"` and/or `"Spellforge"`.
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub likes: u32,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub images: Vec<String>,
    #[serde(default)]
    pub versions: Vec<ModVersion>,
}

/// One uploaded version of the mod. Each version is mirrored as a separate
/// zip next to the `details.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModVersion {
    pub date_uploaded: String,
    #[serde(default)]
    pub version_notes: String,
    pub download_url: String,
    /// Filename of the mirrored zip, relative to the mod's directory.
    pub local_file: String,
}

impl ModDetails {
    pub fn requires_spellforge(&self) -> bool {
        self.tags
            .iter()
            .any(|t| t.eq_ignore_ascii_case("Spellforge"))
    }

    pub fn load(path: &Path) -> Result<Self, ModDetailsError> {
        let bytes = fs::read(path).map_err(|e| ModDetailsError::Io(path.to_path_buf(), e))?;
        serde_json::from_slice(&bytes).map_err(|e| ModDetailsError::Parse(path.to_path_buf(), e))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ModDetailsError {
    #[error("reading {0}: {1}")]
    Io(std::path::PathBuf, #[source] std::io::Error),
    #[error("parsing {0}: {1}")]
    Parse(std::path::PathBuf, #[source] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_vanilla() {
        let json = r#"{
          "slug": "derby-attack-siege",
          "title": "Derby Attack Siege",
          "page_url": "https://rhmods.com/missions/derby-attack-siege/",
          "author": "Nescafe",
          "map": "Derby",
          "uploaded": "Jan 7, 2025",
          "tags": ["Vanilla"],
          "likes": 1,
          "description": "I've created my own version of Derby siege",
          "images": ["https://example.com/img.png"],
          "versions": [{
            "date_uploaded": "Jan 7, 2025",
            "version_notes": "",
            "download_url": "https://example.com/x.zip",
            "local_file": "2025-01-07.zip"
          }]
        }"#;
        let d: ModDetails = serde_json::from_str(json).unwrap();
        assert_eq!(d.slug, "derby-attack-siege");
        assert!(!d.requires_spellforge());
        assert_eq!(d.versions[0].local_file, "2025-01-07.zip");
    }

    #[test]
    fn parse_spellforge() {
        let json = r#"{
          "slug": "meet-the-spy",
          "title": "Meet the Spy",
          "page_url": "https://rhmods.com/missions/meet-the-spy/",
          "author": "CraignRush",
          "map": "York",
          "uploaded": "Feb 12, 2026",
          "tags": ["Spellforge"],
          "likes": 2,
          "description": "...",
          "images": [],
          "versions": []
        }"#;
        let d: ModDetails = serde_json::from_str(json).unwrap();
        assert!(d.requires_spellforge());
    }
}
