use std::path::Path;

use cu::pre::*;

use crate::home;

#[derive(Default)]
pub struct ShellProfile {}

impl ShellProfile {
    pub fn save(&self) -> cu::Result<()> {
        let init_dir = home::init_dir();
        cu::check!(
            self.save_bash(&init_dir),
            "failed to save bash init profile"
        )?;
        cu::check!(
            self.save_pwsh(&init_dir),
            "failed to save pwsh init profile"
        )?;
        Ok(())
    }
    fn save_bash(&self, init_dir: &Path) -> cu::Result<()> {
        let init_bash = init_dir.join("init.bash");
        let home = init_dir.parent_abs()?;
        let content = format!(
            r#"# init/init.bash
# this file is managed by the tool, do not edit manually
export SHAFT_HOME="{}"
export PATH="$SHAFT_HOME:$SHAFT_HOME/bin:$PATH"
# ===
    "#,
            home.as_utf8()?
        );

        // TODO: shell configs from package

        cu::fs::write(init_bash, content)?;
        Ok(())
    }
    fn save_pwsh(&self, init_dir: &Path) -> cu::Result<()> {
        let init_pwsh = init_dir.join("init.pwsh");
        let content = r#"# init/init.pwsh
# this file is managed by the tool, do not edit manually
# ===
    "#;

        // TODO: shell configs from package

        cu::fs::write(init_pwsh, content)?;
        Ok(())
    }
}
