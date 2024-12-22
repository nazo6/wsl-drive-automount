#![cfg_attr(feature = "silent", windows_subsystem = "windows")]

use std::{collections::HashMap, time::Duration};

use std::os::windows::process::CommandExt;

use futures::StreamExt as _;
use serde::Deserialize;
use wmi::{COMLibrary, FilterValue, WMIConnection};

#[derive(Deserialize, Debug)]
#[serde(rename = "__InstanceCreationEvent")]
#[serde(rename_all = "PascalCase")]
struct NewEvent {
    target_instance: LogicalDisk,
}

#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_LogicalDisk")]
#[serde(rename_all = "PascalCase")]
struct LogicalDisk {
    name: String,
}

const CREATE_NO_WINDOW: u32 = 0x08000000;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    futures::executor::block_on(main_inner())
}

async fn main_inner() -> Result<(), Box<dyn std::error::Error>> {
    let com_con = COMLibrary::new()?;
    let wmi_con = WMIConnection::new(com_con)?;

    let mut filters = HashMap::<String, FilterValue>::new();

    filters.insert(
        "TargetInstance".to_owned(),
        FilterValue::is_a::<LogicalDisk>()?,
    );

    let mut stream =
        wmi_con.async_filtered_notification::<NewEvent>(&filters, Some(Duration::from_secs(1)))?;

    while let Some(Ok(data)) = stream.next().await {
        let drive_path = data.target_instance.name;

        match execut_mount(&drive_path) {
            Ok(_) => println!("Drive {} mounted", drive_path),
            Err(e) => eprintln!("Failed to mount drive {}: {}", drive_path, e),
        }
    }

    Ok(())
}

fn execut_mount(drive_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let drive_letter = drive_path
        .get(0..1)
        .expect("Invalid drive path")
        .to_lowercase();

    let is_wsl_running = std::process::Command::new("wsl")
        .creation_flags(CREATE_NO_WINDOW)
        .arg("--list")
        .arg("--running")
        .spawn()?
        .wait()?
        // wsl --list --running returns 0 if there are running WSL instances
        .success();

    if is_wsl_running {
        let res = std::process::Command::new("wsl")
            .creation_flags(CREATE_NO_WINDOW)
            .args(["-u", "root", "-e", "mount", "-t", "drvfs", drive_path])
            .arg(format!("/mnt/{}", drive_letter))
            .spawn()?
            .wait()?;
        if !res.success() {
            return Err("Failed to mount drive".into());
        }
    } else {
        return Err("WSL is not running".into());
    }

    Ok(())
}
