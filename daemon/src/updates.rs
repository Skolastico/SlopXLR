use serde_json::{Value, json};
#[cfg(target_os = "windows")]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(target_os = "windows")]
static INSTALLING: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "windows")]
fn helper(mode: &str) -> anyhow::Result<std::process::Command> {
    use std::os::windows::process::CommandExt;
    let app = std::env::current_exe()?.parent().unwrap().to_path_buf();
    if !app.join("Update.ps1").is_file() || !app.join("release.json").is_file() {
        anyhow::bail!("Disponível apenas na versão instalada ou portátil distribuída.");
    }
    let system = std::path::PathBuf::from(std::env::var("SystemRoot")?);
    let mut command = std::process::Command::new(system.join("System32/WindowsPowerShell/v1.0/powershell.exe"));
    command.args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-File"])
        .arg(app.join("Update.ps1")).args(["-Mode", mode]).current_dir(app).creation_flags(0x08000000);
    Ok(command)
}

#[cfg(target_os = "windows")]
pub async fn check() -> Value {
    match tokio::task::spawn_blocking(|| -> anyhow::Result<Value> {
        let output = helper("Check")?.output()?;
        let text = String::from_utf8(output.stdout)?;
        let value: Value = serde_json::from_str(text.trim_start_matches('\u{feff}').trim())?;
        if !output.status.success() && value.get("Error").is_none() {
            anyhow::bail!("Could not check for updates");
        }
        Ok(value)
    }).await {
        Ok(Ok(value)) => value,
        Ok(Err(error)) => json!({"Error":error.to_string()}),
        Err(error) => json!({"Error":error.to_string()}),
    }
}
#[cfg(not(target_os = "windows"))]
pub async fn check() -> Value { json!({"available":false,"message":"Disponível apenas na versão instalada ou portátil distribuída."}) }

#[cfg(target_os = "windows")]
pub fn install(version: &str) -> anyhow::Result<()> {
    if version.len() > 40 || !version.bytes().all(|b| b.is_ascii_digit() || b"-.beta".contains(&b)) {
        anyhow::bail!("Invalid update version");
    }
    if INSTALLING.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
        anyhow::bail!("An update is already running");
    }
    let started = (|| -> anyhow::Result<std::process::Child> {
        let mut command = helper("Install")?;
        command.arg("-ExpectedVersion").arg(version);
        Ok(command.spawn()?)
    })();
    match started {
        Ok(mut child) => {
            std::thread::spawn(move || { let _ = child.wait(); INSTALLING.store(false, Ordering::SeqCst); });
            Ok(())
        },
        Err(error) => { INSTALLING.store(false, Ordering::SeqCst); Err(error) }
    }
}
#[cfg(not(target_os = "windows"))]
pub fn install(_: &str) -> anyhow::Result<()> { anyhow::bail!("Windows updates only") }
