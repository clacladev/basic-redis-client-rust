pub enum ValueType {
    String,
}

impl TryFrom<u8> for ValueType {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ValueType::String),
            _ => anyhow::bail!("-> Value type not supported. Value: {}", value),
        }
    }
}
