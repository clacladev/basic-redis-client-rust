use crate::cli::CliParam;
use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

const SETTINGS_DIR_ID: &str = "dir";
const SETTINGS_DBFILENAME_ID: &str = "dbfilename";

#[derive(Debug)]
struct Item {
    value: String,
    expires_at: Option<u128>,
}

pub struct Database {
    data: HashMap<String, Item>,
    config: HashMap<String, String>,
}

// Init related
impl Database {
    pub fn new() -> Self {
        Database {
            data: HashMap::new(),
            config: HashMap::new(),
        }
    }
}

// Config related
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

// Data related
impl Database {
    pub fn set(
        &mut self,
        key: String,
        value: String,
        expires_at: Option<u128>,
    ) -> anyhow::Result<()> {
        self.data.insert(key, Item { value, expires_at });
        Ok(())
    }

    pub fn get(&mut self, key: String) -> anyhow::Result<Option<String>> {
        let Some(item) = self.data.get(&key) else {
            return Ok(None);
        };
        println!("-> Item: {:?}", item);

        let value = Some(item.value.clone());
        let Some(expires_at) = item.expires_at else {
            return Ok(value);
        };

        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
        if now <= expires_at {
            return Ok(value);
        }

        self.delete(key)?;
        Ok(None)
    }

    pub fn delete(&mut self, key: String) -> anyhow::Result<()> {
        self.data.remove(&key);
        Ok(())
    }
}
