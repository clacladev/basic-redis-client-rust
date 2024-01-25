use std::collections::HashMap;

pub struct Database {
    pub data: HashMap<String, String>,
}

impl Database {
    pub fn new() -> Self {
        Database {
            data: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: String, value: String) -> anyhow::Result<()> {
        self.data.insert(key, value);
        Ok(())
    }

    pub fn get(&self, key: String) -> anyhow::Result<Option<String>> {
        Ok(self.data.get(&key).cloned())
    }
}
