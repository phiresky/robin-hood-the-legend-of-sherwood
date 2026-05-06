//! Font manager — loads and caches fonts by name.
//!
//! Stores a flat registry of `FontEntry` records keyed by name, with group
//! and id metadata for each entry.

/// A single registered font.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FontEntry {
    pub name: String,
    pub font_group: u32,
    pub font_id: u32,
}

/// Registry of named fonts.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct FontManager {
    fonts: Vec<FontEntry>,
}

impl FontManager {
    /// Register a new font entry. If a font with the same name already exists
    /// it is replaced.
    pub fn register_font(&mut self, name: impl Into<String>, font_group: u32, font_id: u32) {
        let name = name.into();
        if let Some(existing) = self.fonts.iter_mut().find(|f| f.name == name) {
            existing.font_group = font_group;
            existing.font_id = font_id;
        } else {
            self.fonts.push(FontEntry {
                name,
                font_group,
                font_id,
            });
        }
    }

    /// Look up a font id by name.
    pub fn get_font_id(&self, name: &str) -> Option<u32> {
        self.fonts
            .iter()
            .find(|f| f.name == name)
            .map(|f| f.font_id)
    }

    /// Look up a font group by name.
    pub fn get_font_group(&self, name: &str) -> Option<u32> {
        self.fonts
            .iter()
            .find(|f| f.name == name)
            .map(|f| f.font_group)
    }

    /// Number of registered fonts.
    pub fn count(&self) -> usize {
        self.fonts.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_lookup() {
        let mut mgr = FontManager::default();
        mgr.register_font("small", 1, 10);
        mgr.register_font("large", 2, 20);

        assert_eq!(mgr.get_font_id("small"), Some(10));
        assert_eq!(mgr.get_font_group("small"), Some(1));
        assert_eq!(mgr.get_font_id("large"), Some(20));
        assert_eq!(mgr.get_font_group("large"), Some(2));
        assert_eq!(mgr.count(), 2);
    }

    #[test]
    fn missing_font_returns_none() {
        let mgr = FontManager::default();
        assert_eq!(mgr.get_font_id("nope"), None);
        assert_eq!(mgr.get_font_group("nope"), None);
        assert_eq!(mgr.count(), 0);
    }

    #[test]
    fn register_replaces_existing() {
        let mut mgr = FontManager::default();
        mgr.register_font("dup", 1, 10);
        mgr.register_font("dup", 3, 30);

        assert_eq!(mgr.count(), 1);
        assert_eq!(mgr.get_font_id("dup"), Some(30));
        assert_eq!(mgr.get_font_group("dup"), Some(3));
    }

    #[test]
    fn serde_roundtrip() {
        let mut mgr = FontManager::default();
        mgr.register_font("serif", 1, 42);
        let json = serde_json::to_string(&mgr).unwrap();
        let restored: FontManager = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.get_font_id("serif"), Some(42));
        assert_eq!(restored.count(), 1);
    }

    #[test]
    fn default_is_empty() {
        let mgr = FontManager::default();
        assert_eq!(mgr.count(), 0);
    }
}
