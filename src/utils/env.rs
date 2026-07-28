pub struct EnvVarGuard {
    name: String,
    old_value: Option<String>,
}

impl EnvVarGuard {
    pub fn set(name: &str, value: &str) -> Self {
        let old_value = std::env::var(name).ok();
        unsafe { std::env::set_var(name, value); }
        Self {
            name: name.to_string(),
            old_value,
        }
    }

    pub fn remove(name: &str) -> Self {
        let old_value = std::env::var(name).ok();
        unsafe { std::env::remove_var(name); }
        Self {
            name: name.to_string(),
            old_value,
        }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        match &self.old_value {
            Some(val) => unsafe { std::env::set_var(&self.name, val) },
            None => unsafe { std::env::remove_var(&self.name) },
        }
    }
}