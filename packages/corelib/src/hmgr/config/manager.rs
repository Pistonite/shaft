use std::marker::PhantomData;
use std::path::Path;

use cu::pre::*;

use crate::hmgr::config::{self, ConfigTemplate};
use crate::jsexe;

pub struct ConfigDef<T> {
    /// Template content
    pub template_str: &'static str,
    /// JS to migrate previous config to latest
    /// length is equal to the latest version.
    /// empty script indicates compatible change - such as
    /// new config values being added
    pub migration_scripts: &'static [&'static str],

    _marker: PhantomData<T>,
}
impl<T> Clone for ConfigDef<T> {
    fn clone(&self) -> Self {
        Self {
            template_str: self.template_str,
            migration_scripts: self.migration_scripts,
            _marker: self._marker.clone(),
        }
    }
}
impl<T> Copy for ConfigDef<T> {}

impl<T> ConfigDef<T> {
    pub const fn new(
        template_str: &'static str,
        migration_scripts: &'static [&'static str],
    ) -> Self {
        Self {
            template_str,
            migration_scripts,
            _marker: PhantomData,
        }
    }
}

impl<T> ConfigDef<T>
where
    for<'de> T: Deserialize<'de>,
{
    pub fn load_default(self) -> cu::Result<T> {
        let template = toml::parse::<ConfigTemplate>(self.template_str)?;
        let content = config::serialize_config_template(&template, self.current_version());
        toml::parse::<T>(&content)
    }
    /// Load the configuration file, perform migration if needed
    #[inline(always)]
    pub fn load(self, path: impl AsRef<Path>) -> cu::Result<T> {
        self.load_impl(path.as_ref())
    }
    fn load_impl(self, path: &Path) -> cu::Result<T> {
        let mut file_content = if !path.exists() {
            let template = toml::parse::<ConfigTemplate>(self.template_str)?;
            let content = config::serialize_config_template(&template, self.current_version());
            cu::fs::write(path, &content)?;
            content
        } else {
            cu::fs::read_string(path)?
        };
        let version = config::peek_version(&file_content).unwrap_or(0);
        let current_version = self.current_version();
        match version.cmp(&current_version) {
            std::cmp::Ordering::Greater => {
                cu::warn!(
                    "version in '{}' is greater than current version",
                    path.display()
                );
            }
            std::cmp::Ordering::Equal => {
                cu::debug!("config is up to date: '{}'", path.display());
            }
            std::cmp::Ordering::Less => {
                let object = cu::check!(
                    toml::parse::<toml::Table>(&file_content),
                    "failed to parse config file as TOML: '{}'",
                    path.display()
                )?;
                let mut object_string = cu::check!(
                    json::stringify(&object),
                    "failed to serialize toml config to json"
                )?;
                cu::warn!(
                    "migrating config file '{}' from version {} -> {}",
                    path.display(),
                    version,
                    current_version
                );

                for (i, script) in self.migration_scripts.iter().enumerate().skip(version) {
                    let next = i + 1;
                    if script.is_empty() {
                        cu::debug!("migrate to v{next}: no change");
                        continue;
                    }
                    cu::debug!("migrating to v{next}: running script");
                    object_string = cu::check!(
                        jsexe::run_str(&object_string, script),
                        "migration to v{next} failed"
                    )?;
                }
                let mut new_config = cu::check!(
                    json::parse::<toml::Table>(&object_string),
                    "failed to parse migrated config"
                )?;
                let template = toml::parse::<ConfigTemplate>(self.template_str)?;
                let content =
                    config::serialize_config(&template, self.current_version(), &mut new_config);
                cu::fs::write(path, &content)?;

                let (unused_count, unused_repr) = config::serialize_leaf_key_values(&new_config);
                if unused_count > 0 {
                    cu::warn!("there were {unused_count} unused config keys:\n{unused_repr}");
                }
                file_content = content;
            }
        }
        cu::check!(
            toml::parse::<T>(&file_content),
            "failed to parse typed config object"
        )
    }
    pub const fn current_version(self) -> usize {
        self.migration_scripts.len()
    }
}
