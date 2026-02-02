use cu::pre::*;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ConfigTemplate {
    #[serde(rename = "section")]
    pub sections: Vec<ConfigSection>,
}

/// One section of the config, which is in the format of:
/// ```toml
/// # comment
/// # ...
/// # comment
/// [key]
/// # child_comment
/// child_key = child_value
/// ...
/// ```
/// There will be one line between each section when converting
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ConfigSection {
    /// Comment of the section, may be multiline
    #[serde(default, deserialize_with = "trim_string")]
    pub comment: String,
    /// Content of the section (key and children)
    #[serde(flatten)]
    pub content: Option<ConfigSectionContent>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ConfigSectionContent {
    /// Key of the section (the part inside [])
    pub key: Vec<String>,
    /// Entries in the section
    #[serde(default)]
    pub children: Vec<ConfigEntry>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ConfigEntry {
    /// Comment attached to this entry
    #[serde(default, deserialize_with = "trim_string")]
    pub comment: String,
    /// Content of the entry
    #[serde(flatten)]
    pub content: Option<ConfigEntryContent>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ConfigEntryContent {
    /// Key of the child entry
    #[serde(deserialize_with = "entry_key")]
    pub key: Vec<String>,
    /// Default value of the entry
    pub value: toml::Value,
}

fn trim_string<'de, D>(deser: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deser)?;
    Ok(s.trim().to_owned())
}

fn entry_key<'de, D>(deser: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = Vec::<String>::deserialize(deser)?;
    if s.is_empty() {
        return Err(serde::de::Error::invalid_length(
            0,
            &"non-empty array representing key path of a child entry",
        ));
    }
    Ok(s)
}
