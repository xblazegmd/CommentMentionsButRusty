use std::collections::HashMap;
use std::error::Error;

use reqwest::Client;

#[cfg(target_os = "macos")]
use std::process::Command;

#[cfg(target_os = "windows")]
use winrt_notification::{Duration, Sound, Toast};

#[cfg(target_os = "linux")]
use notify_rust::Notification;

pub fn format_kv(string: String, sep: &str) -> HashMap<String, String> {
    let pieces: Vec<&str> = string.split(sep).collect();
    let mut ret = HashMap::new();

    for chunk in pieces.chunks_exact(2) {
        ret.insert(chunk[0].to_string(), chunk[1].to_string());
    }

    ret
}

pub async fn request_gd_servers(endpoint: &str, params: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
    let client = Client::new();
    let res = client.post(format!("https://www.boomlings.com/database/{}", endpoint))
        .header("User-Agent", "")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(params.to_string())
        .send().await?.text().await?;
    Ok(res)
}

pub async fn get_special_id() -> Result<i32, Box<dyn Error + Send + Sync>> {
    let res = request_gd_servers("getGJLevels21.php", "type=21&secret=Wmfd2893gb7").await?;

    let no_meta = res.split("#").next().ok_or("what even happened here bruh")?.to_string();

    let daily = no_meta.split("|").next().ok_or("what even happened here as well bruh")?.to_string();
    let daily_id = format_kv(daily, ":");
    let int_daily_id: i32 = daily_id.get("1").ok_or("wrong key you idiot")?.parse()?;

    Ok(int_daily_id)
}

#[cfg(target_os = "macos")]
#[allow(dead_code)]
pub fn notify(title: String, msg: String) {
    Command::new("osascript")
        .arg("-e")
        .arg(
            format!(
                r#"Display notification "{}" with title "{}" "#,
                msg,
                title
            )
        )
        .status().expect("Could not show notification");
}

#[cfg(target_os = "windows")]
pub fn notify(title: &str, msg: &str) {
    Toast::new(Toast::POWERSHELL_APP_ID)
        .title(title)
        .text1(msg)
        .sound(Some(Sound::SMS))
        .duration(Duration::Short)
        .show().expect("Could not show notification");
}

#[cfg(target_os = "linux")]
pub fn notify(title: &str, msg: &str) {
    Notification::new()
        .summary(title)
        .body(msg)
        .icon("rust")
        .show().expect("Could not show notification");
}