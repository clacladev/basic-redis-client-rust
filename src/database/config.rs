use super::{Database, SETTINGS_DBFILENAME_ID, SETTINGS_DIR_ID};
use crate::cli::CliParam;

impl Database {
    pub fn config_setup(&mut self, cli_params: &[CliParam]) {
        cli_params.iter().for_each(|param| match param {
            CliParam::Dir(dir) => {
                self.config.insert(SETTINGS_DIR_ID.to_string(), dir.clone());
            }
            CliParam::DbFilename(dbfilename) => {
                self.config
                    .insert(SETTINGS_DBFILENAME_ID.to_string(), dbfilename.clone());
            }
        });
    }

    pub fn config_get(&self, key: &str) -> Option<String> {
        self.config.get(key).cloned()
    }
}
