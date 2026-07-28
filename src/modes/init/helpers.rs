use std::fs::File;
use std::io::Write;

pub fn write_file(name: &str, content: &str) -> std::io::Result<()> {
    let nami_dir = crate::utils::get_nami_dir();
    let dest_path = nami_dir.join(name);
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = File::create(dest_path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

pub fn write_file_if_not_exists(name: &str, content: &str) -> std::io::Result<bool> {
    let nami_dir = crate::utils::get_nami_dir();
    let dest_path = nami_dir.join(name);
    if dest_path.exists() {
        return Ok(false);
    }
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = File::create(dest_path)?;
    file.write_all(content.as_bytes())?;
    Ok(true)
}

pub fn merge_toml(existing: &mut toml::Value, incoming: &toml::Value) {
    match (existing, incoming) {
        (toml::Value::Table(ext_table), toml::Value::Table(inc_table)) => {
            for (key, val) in inc_table {
                if let Some(ext_val) = ext_table.get_mut(key) {
                    merge_toml(ext_val, val);
                } else {
                    ext_table.insert(key.clone(), val.clone());
                }
            }
        }
        (ext_val, inc_val) => {
            *ext_val = inc_val.clone();
        }
    }
}
