use once_cell::sync::Lazy;
use serde_json::Value;

static EN: Lazy<Value> =
    Lazy::new(|| serde_json::from_str(include_str!("../assets/i18n/en.json")).unwrap());
static PT_BR: Lazy<Value> =
    Lazy::new(|| serde_json::from_str(include_str!("../assets/i18n/pt-BR.json")).unwrap());

const FALLBACK_EN: &[(&str, &str)] = &[
    ("navbar.title", "SubTranslator"),
    ("navbar.translation", "Translation"),
    ("navbar.settings", "Settings"),
    ("navbar.logs", "Logs"),
    (
        "translation.placeholder",
        "Translation workspace will appear here.",
    ),
    ("settings.placeholder", "Settings will appear here."),
];

const FALLBACK_PT: &[(&str, &str)] = &[
    ("navbar.title", "SubTranslator"),
    ("navbar.translation", "Traduzir"),
    ("navbar.settings", "Configurar"),
    ("navbar.logs", "Logs"),
    (
        "translation.placeholder",
        "A área de tradução aparecerá aqui.",
    ),
    ("settings.placeholder", "As configurações aparecerão aqui."),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Language {
    #[default]
    English,
    PortugueseBrazil,
}

impl Language {
    pub fn code(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::PortugueseBrazil => "pt-BR",
        }
    }

    pub fn from_code(code: &str) -> Self {
        match code {
            "pt-BR" | "pt_BR" | "pt" => Self::PortugueseBrazil,
            _ => Self::English,
        }
    }
}

pub fn t(lang: Language, key: &str) -> String {
    if let Some(value) = lookup(bundle(lang), key) {
        return value;
    }
    if let Some(value) = lookup(&EN, key) {
        return value;
    }
    fallback(lang, key).unwrap_or_else(|| key.to_string())
}

/// Interpolate `{{name}}` placeholders from the JSON catalogs.
pub fn tf(lang: Language, key: &str, vars: &[(&str, &str)]) -> String {
    let mut value = t(lang, key);
    for (name, replacement) in vars {
        value = value.replace(&format!("{{{{{name}}}}}"), replacement);
    }
    value
}

fn bundle(lang: Language) -> &'static Value {
    match lang {
        Language::English => &EN,
        Language::PortugueseBrazil => &PT_BR,
    }
}

fn lookup(root: &Value, key: &str) -> Option<String> {
    let mut current = root;
    for part in key.split('.') {
        current = current.get(part)?;
    }
    match current {
        Value::String(s) => Some(s.clone()),
        _ => None,
    }
}

fn fallback(lang: Language, key: &str) -> Option<String> {
    let table = match lang {
        Language::English => FALLBACK_EN,
        Language::PortugueseBrazil => FALLBACK_PT,
    };
    table
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| (*v).to_string())
        .or_else(|| {
            FALLBACK_EN
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| (*v).to_string())
        })
}
