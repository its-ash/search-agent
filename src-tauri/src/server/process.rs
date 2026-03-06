use tokio::process::{Child, Command};

use crate::config::AppConfig;

pub async fn spawn_server(config: &AppConfig) -> anyhow::Result<Child> {
    let mut cmd = Command::new(&config.llama_command);
    cmd.args(&config.llama_args);
    cmd.kill_on_drop(false);
    let child = cmd.spawn()?;
    Ok(child)
}
