const PARAM_PREFIX: &str = "--";
const PARAM_DIR: &str = "--dir";
const PARAM_DBFILENAME: &str = "--dbfilename";

#[derive(Debug)]
pub enum CliParam {
    Dir(String),
    DbFilename(String),
}

impl CliParam {
    fn get_param_value(string: &str) -> Option<String> {
        let string = string.trim();
        if string.is_empty() || string.starts_with(PARAM_PREFIX) {
            return None;
        }
        Some(string.to_string())
    }

    pub fn from(strings: &[String]) -> Vec<Self> {
        let mut params: Vec<Self> = Vec::new();
        if strings.is_empty() {
            return params;
        }

        let mut strings_next_index = 1;
        match strings[0].to_lowercase().as_str() {
            PARAM_DIR => {
                if strings.len() >= 2 {
                    if let Some(value) = Self::get_param_value(&strings[1]) {
                        params.push(CliParam::Dir(value));
                        strings_next_index += 1;
                    }
                }
            }
            PARAM_DBFILENAME => {
                if strings.len() >= 2 {
                    if let Some(value) = Self::get_param_value(&strings[1]) {
                        params.push(CliParam::DbFilename(value));
                        strings_next_index += 1;
                    }
                }
            }
            _ => {}
        }

        // Tries to get other params from the remaining strings
        params.extend(Self::from(&strings[strings_next_index..]));

        params
    }
}
