// JSON Translation Mapping Loader
// Author: Bureum Lee
// License: MIT

// ⚠️ 중요: 이 파일은 빌드 타임(Host)에서만 실행됩니다. (Not Kernel Code)
// 커널 모듈(lib.rs)에서는 절대 이 모듈을 직접 mod로 불러오면 안 됩니다!

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// ═══════════════════════════════════════════════════════════════════════════
// JSON Schema Definitions
// ═══════════════════════════════════════════════════════════════════════════
#[derive(Debug, Serialize, Deserialize)]
pub struct TranslationMap {
    pub language: String,
    pub language_code: String,
    pub version: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub mappings: Vec<MappingEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MappingEntry {
    pub en: String,
    pub translated: String,
    #[serde(default)]
    pub context: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
// Runtime Translation Table Builder
// ═══════════════════════════════════════════════════════════════════════════
pub struct TranslationTableBuilder {
    entries: Vec<(String, String)>,
}

impl TranslationTableBuilder {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
    
    /// Load from JSON file
    pub fn load_json<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let map: TranslationMap = serde_json::from_str(&content)?;
        
        for entry in map.mappings {
            self.entries.push((entry.en, entry.translated));
        }
        
        Ok(())
    }
    
    /// Load from multiple JSON files (for multi-language support)
    pub fn load_directory<P: AsRef<Path>>(&mut self, dir: P) -> Result<(), Box<dyn std::error::Error>> {
        let dir_path = dir.as_ref();
        
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                self.load_json(&path)?;
            }
        }
        
        Ok(())
    }
    
    /// Generate Rust code for static compilation
    pub fn generate_rust_code(&self) -> String {
        let mut code = String::new();
        
        code.push_str("// Auto-generated translation table\n");
        code.push_str("// DO NOT EDIT MANUALLY\n\n");
        code.push_str("use crate::i18n::TranslationEntry;\n\n"); // 의존성 경로 추가
        code.push_str("pub const I18N_TABLE_KO: &[TranslationEntry] = &[\n");
        
        for (original, translated) in &self.entries {
            code.push_str(&format!(
                "    TranslationEntry::new(\"{}\", \"{}\"),\n",
                escape_rust_string(original),
                escape_rust_string(translated)
            ));
        }
        
        code.push_str("];\n");
        code
    }
    
    /// Build HashMap for runtime use (Optional, for host-side tools)
    pub fn build_hashmap(&self) -> HashMap<String, String> {
        self.entries.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}

fn escape_rust_string(s: &str) -> String {
    s.replace('\\', "\\\\")
     .replace('"', "\\\"")
     .replace('\n', "\\n")
     .replace('\r', "\\r")
     .replace('\t', "\\t")
}

// ═══════════════════════════════════════════════════════════════════════════
// Example JSON Template
// ═══════════════════════════════════════════════════════════════════════════
pub const EXAMPLE_JSON_KO: &str = r#"{
  "language": "한국어",
  "language_code": "ko_KR",
  "version": "1.0.0",
  "author": "Bureum Lee",
  "description": "Korean translation for Linux kernel messages",
  "mappings": [
    {
      "en": "Ready!",
      "translated": "준비 완료!",
      "category": "status"
    },
    {
      "en": "Error",
      "translated": "오류",
      "category": "error"
    },
    {
      "en": "Permission denied",
      "translated": "권한이 거부되었습니다",
      "context": "Access control error",
      "category": "permission"
    }
  ]
}"#;

// ═══════════════════════════════════════════════════════════════════════════
// Build Script Helper (for build.rs)
// ═══════════════════════════════════════════════════════════════════════════
/// Generate static Rust code from JSON mapping files
/// This function is meant to be called from build.rs
pub fn generate_translation_table_from_json(
    json_dir: &str,
    output_file: &str
) -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = TranslationTableBuilder::new();
    
    // 디렉토리가 없으면 기본 예제 생성 (User Friendly)
    if !Path::new(json_dir).exists() {
        fs::create_dir_all(json_dir)?;
        let default_path = Path::new(json_dir).join("ko.json");
        fs::write(default_path, EXAMPLE_JSON_KO)?;
    }
    
    builder.load_directory(json_dir)?;
    
    let rust_code = builder.generate_rust_code();
    fs::write(output_file, rust_code)?;
    
    println!("Generated translation table: {}", output_file);
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// Testing
// ═══════════════════════════════════════════════════════════════════════════
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_json_parsing() {
        let map: TranslationMap = serde_json::from_str(EXAMPLE_JSON_KO).unwrap();
        assert_eq!(map.language, "한국어");
        assert_eq!(map.language_code, "ko_KR");
        assert!(map.mappings.len() > 0);
    }
    
    #[test]
    fn test_table_builder() {
        let map: TranslationMap = serde_json::from_str(EXAMPLE_JSON_KO).unwrap();
        let mut builder = TranslationTableBuilder::new();
        
        for entry in map.mappings {
            builder.entries.push((entry.en, entry.translated));
        }
        
        let hashmap = builder.build_hashmap();
        assert_eq!(hashmap.get("Error"), Some(&"오류".to_string()));
    }
}
