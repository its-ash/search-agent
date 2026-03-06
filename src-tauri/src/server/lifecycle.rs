use std::sync::Arc;
use std::time::Duration;

use parking_lot::Mutex;
use tokio::process::Child;
use tokio::time::sleep;
use tracing::{error, info};

use crate::{config::AppConfig, storage::sqlite::SqliteStore};

use super::{health, process};

#[derive(Debug, Clone)]
pub struct ServerStatusSnapshot {
    pub status: String,
    pub ownership: String,
    pub endpoint: String,
    pub pid: Option<u32>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ownership {
    None,
    External,
    SelfManaged,
}

#[derive(Debug)]
struct Inner {
    status: String,
    ownership: Ownership,
    pid: Option<u32>,
    child: Option<Child>,
    message: Option<String>,
}

pub struct ServerManager {
    config: AppConfig,
    sqlite: Arc<SqliteStore>,
    inner: Mutex<Inner>,
}

impl ServerManager {
    pub fn new(config: AppConfig, sqlite: Arc<SqliteStore>) -> Self {
        Self {
            config,
            sqlite,
            inner: Mutex::new(Inner {
                status: "stopped".to_string(),
                ownership: Ownership::None,
                pid: None,
                child: None,
                message: None,
            }),
        }
    }

    pub async fn initialize(&self) -> anyhow::Result<()> {
        {
            let mut g = self.inner.lock();
            g.status = "starting".to_string();
            g.message = Some("Checking local llama server".to_string());
        }

        if health::check(&self.config.llama_endpoint).await {
            info!("reusing already running local llama server");
            self.update_state("running", Ownership::External, None, Some("external server detected"));
            self.sqlite
                .set_server_state("running", "external", None, &self.config.llama_endpoint, None)
                .await?;
            return Ok(());
        }

        let mut child = match process::spawn_server(&self.config).await {
            Ok(child) => child,
            Err(err) => {
                self.update_state(
                    "error",
                    Ownership::None,
                    None,
                    Some(&format!("failed to spawn llama server: {err}")),
                );
                let _ = self
                    .sqlite
                    .set_server_state(
                        "error",
                        "none",
                        None,
                        &self.config.llama_endpoint,
                        Some("llama-server command unavailable"),
                    )
                    .await;
                return Err(err);
            }
        };
        let pid = child.id();
        self.update_state("starting", Ownership::SelfManaged, pid, Some("spawned local llama server"));

        let mut ready = false;
        for _ in 0..60 {
            if health::check(&self.config.llama_endpoint).await {
                ready = true;
                break;
            }
            if let Some(status) = child.try_wait()? {
                self.update_state(
                    "error",
                    Ownership::SelfManaged,
                    pid,
                    Some(&format!("server exited early: {status}")),
                );
                break;
            }
            sleep(Duration::from_secs(1)).await;
        }

        if !ready {
            self.update_state("error", Ownership::SelfManaged, pid, Some("server startup timeout"));
            return Err(anyhow::anyhow!("llama server failed to become ready"));
        }

        {
            let mut g = self.inner.lock();
            g.status = "running".to_string();
            g.message = Some("local server ready".to_string());
            g.child = Some(child);
        }

        self.sqlite
            .set_server_state("running", "self", pid.map(i64::from), &self.config.llama_endpoint, None)
            .await?;

        Ok(())
    }

    pub async fn restart(&self) -> anyhow::Result<()> {
        self.shutdown_if_owned().await?;
        self.initialize().await
    }

    pub async fn shutdown_if_owned(&self) -> anyhow::Result<()> {
        let child_opt: Option<Child> = {
            let mut g = self.inner.lock();
            if g.ownership != Ownership::SelfManaged {
                return Ok(());
            }
            g.status = "stopping".to_string();
            g.child.take()
        };

        if let Some(mut child) = child_opt {
            let _ = child.start_kill();
            for _ in 0..8 {
                if child.try_wait()?.is_some() {
                    break;
                }
                sleep(Duration::from_secs(1)).await;
            }
            if child.try_wait()?.is_none() {
                error!("forcing kill for owned llama server process");
                let _ = child.kill().await;
            }
        }

        self.update_state("stopped", Ownership::None, None, Some("server stopped"));
        self.sqlite
            .set_server_state("stopped", "none", None, &self.config.llama_endpoint, None)
            .await?;
        Ok(())
    }

    pub async fn status_snapshot(&self) -> ServerStatusSnapshot {
        let mut maybe_crashed = false;
        {
            let mut g = self.inner.lock();
            if g.ownership == Ownership::SelfManaged {
                if let Some(child) = g.child.as_mut() {
                    if let Ok(Some(status)) = child.try_wait() {
                        g.status = "error".to_string();
                        g.message = Some(format!("server crashed: {status}"));
                        g.child = None;
                        maybe_crashed = true;
                    }
                }
            }
        }

        if maybe_crashed {
            let _ = self
                .sqlite
                .set_server_state("error", "self", None, &self.config.llama_endpoint, Some("crash detected"))
                .await;
        }

        let g = self.inner.lock();
        ServerStatusSnapshot {
            status: g.status.clone(),
            ownership: match g.ownership {
                Ownership::None => "none".to_string(),
                Ownership::External => "external".to_string(),
                Ownership::SelfManaged => "self".to_string(),
            },
            endpoint: self.config.llama_endpoint.clone(),
            pid: g.pid,
            message: g.message.clone(),
        }
    }

    pub fn endpoint(&self) -> &str {
        &self.config.llama_endpoint
    }

    fn update_state(&self, status: &str, ownership: Ownership, pid: Option<u32>, msg: Option<&str>) {
        let mut g = self.inner.lock();
        g.status = status.to_string();
        g.ownership = ownership;
        g.pid = pid;
        g.message = msg.map(ToString::to_string);
    }
}

pub fn should_shutdown_on_exit(ownership: &str) -> bool {
    ownership == "self"
}

#[cfg(test)]
mod tests {
    use super::{should_shutdown_on_exit, Ownership};

    #[test]
    fn ownership_values_are_distinct() {
        assert_ne!(Ownership::External, Ownership::SelfManaged);
    }

    #[test]
    fn shutdown_only_for_self() {
        assert!(should_shutdown_on_exit("self"));
        assert!(!should_shutdown_on_exit("external"));
    }
}
