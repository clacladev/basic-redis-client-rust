pub fn set(key: &str, value: &str) -> anyhow::Result<()> {
    println!("-> Set key: {}, value: {}", key, value);
    Ok(())
}
