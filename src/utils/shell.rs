use tokio::process::Command;

pub fn build_shell_command() -> (&'static str, &'static str) {
    #[cfg(target_os = "windows")]
    {
        ("cmd.exe", "/C")
    }
    #[cfg(not(target_os = "windows"))]
    {
        ("sh", "-c")
    }
}

pub fn spawn_shell_command(command: &str, current_dir: Option<&std::path::Path>) -> std::result::Result<Command, String> {
    let (shell, flag) = build_shell_command();
    let mut cmd = Command::new(shell);
    cmd.arg(flag).arg(command);
    if let Some(dir) = current_dir {
        cmd.current_dir(dir);
    }
    Ok(cmd)
}