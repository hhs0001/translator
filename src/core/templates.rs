use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::paths;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptTemplate {
    pub id: String,
    pub name: String,
    pub content: String,
    #[serde(default)]
    pub created_at: u64,
    #[serde(default)]
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct TemplatesData {
    templates: Vec<PromptTemplate>,
}

pub fn load_all() -> Result<Vec<PromptTemplate>, String> {
    paths::migrate_old_settings();
    let path = paths::templates_path()?;
    load_from(&path)
}

pub fn save_all(templates: &[PromptTemplate]) -> Result<(), String> {
    let path = paths::templates_path()?;
    save_to(&path, templates)
}

pub fn add(name: String, content: String) -> Result<PromptTemplate, String> {
    let mut templates = load_all()?;
    let now = now_millis()?;

    let template = PromptTemplate {
        id: format!("{}-{}", now, rand_id()),
        name,
        content,
        created_at: now,
        updated_at: now,
    };

    templates.push(template.clone());
    save_all(&templates)?;
    Ok(template)
}

pub fn update(
    template_id: &str,
    name: Option<String>,
    content: Option<String>,
) -> Result<PromptTemplate, String> {
    let mut templates = load_all()?;
    let now = now_millis()?;

    let template = templates
        .iter_mut()
        .find(|t| t.id == template_id)
        .ok_or_else(|| "Template not found".to_string())?;

    if let Some(n) = name {
        template.name = n;
    }
    if let Some(c) = content {
        template.content = c;
    }
    template.updated_at = now;

    let updated = template.clone();
    save_all(&templates)?;
    Ok(updated)
}

pub fn delete(template_id: &str) -> Result<(), String> {
    let mut templates = load_all()?;
    templates.retain(|t| t.id != template_id);
    save_all(&templates)
}

fn load_from(path: &std::path::Path) -> Result<Vec<PromptTemplate>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content =
        fs::read_to_string(path).map_err(|e| format!("Failed to read templates: {}", e))?;
    let data: TemplatesData =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse templates: {}", e))?;
    Ok(data.templates)
}

fn save_to(path: &std::path::Path, templates: &[PromptTemplate]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create app data dir: {}", e))?;
    }

    let data = TemplatesData {
        templates: templates.to_vec(),
    };
    let content = serde_json::to_string_pretty(&data)
        .map_err(|e| format!("Failed to serialize templates: {}", e))?;
    fs::write(path, content).map_err(|e| format!("Failed to write templates: {}", e))
}

fn now_millis() -> Result<u64, String> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("Time error: {}", e))?
        .as_millis() as u64)
}

fn rand_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    format!("{:x}", nanos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camel_case_json_compatibility() {
        let json = r#"{
            "templates": [
                {
                    "id": "1",
                    "name": "PT",
                    "content": "traduza",
                    "createdAt": 1,
                    "updatedAt": 2
                }
            ]
        }"#;
        let data: TemplatesData = serde_json::from_str(json).unwrap();
        assert_eq!(data.templates.len(), 1);
        assert_eq!(data.templates[0].created_at, 1);
        assert_eq!(data.templates[0].updated_at, 2);
    }

    #[test]
    fn missing_file_returns_empty() {
        let path = std::env::temp_dir().join(format!(
            "translator-templates-missing-{}.json",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let templates = load_from(&path).unwrap();
        assert!(templates.is_empty());
    }

    #[test]
    fn save_load_roundtrip() {
        let dir = std::env::temp_dir().join(format!(
            "translator-templates-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("templates.json");

        let templates = vec![PromptTemplate {
            id: "a".into(),
            name: "N".into(),
            content: "C".into(),
            created_at: 10,
            updated_at: 20,
        }];
        save_to(&path, &templates).unwrap();
        let loaded = load_from(&path).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "N");
        assert_eq!(loaded[0].content, "C");

        let _ = fs::remove_dir_all(&dir);
    }
}
