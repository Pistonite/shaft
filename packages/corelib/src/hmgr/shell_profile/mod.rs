use std::path::Path;

use cu::pre::*;

use crate::hmgr;

#[derive(Default)]
pub struct ShellProfile {}

impl ShellProfile {
    pub fn save(&self) -> cu::Result<()> {
        let init_dir = hmgr::paths::init_root();
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
        let shaft_home = init_dir.parent_abs()?;
        let content =
            include_str!("./init_template.bash").replace("{{shaft_home}}", shaft_home.as_utf8()?);

        // TODO: shell configs from package

        cu::fs::write(init_bash, content)?;
        Ok(())
    }
    fn save_pwsh(&self, init_dir: &Path) -> cu::Result<()> {
        let init_pwsh = init_dir.join("init.pwsh");
        let content = include_str!("./init_template.pwsh");

        // TODO: shell configs from package

        cu::fs::write(init_pwsh, content)?;
        Ok(())
    }
}
