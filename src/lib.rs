use rand::Rng;
use reqwest::Client;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::os::unix::fs::PermissionsExt;
use std::{
    env::consts::ARCH, env::consts::OS, error::Error, fs, io, path::Path, process::Command,
    time::Duration,
};
use thirtyfour::{
    prelude::ElementWaitable, By, ChromeCapabilities, DesiredCapabilities, WebDriver,
};
use tracing::{error, info};
use zip::ZipArchive;

fn get_chrome_caps() -> Result<ChromeCapabilities, Box<dyn Error>> {
    let mut caps = DesiredCapabilities::chrome();

    caps.set_no_sandbox()?;
    caps.set_disable_dev_shm_usage()?;
    caps.add_chrome_arg("--disable-blink-features=AutomationControlled")?;
    caps.add_chrome_arg("window-size=1920,1080")?;
    caps.add_chrome_arg("user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/102.0.0.0 Safari/537.36")?;
    caps.add_chrome_arg("disable-infobars")?;
    caps.add_chrome_option("excludeSwitches", ["enable-automation"])?;

    Ok(caps)
}

/// Fetches a new ChromeDriver executable and patches it to prevent detection.
/// Returns a WebDriver instance.
pub async fn chrome_driver() -> Result<WebDriver, Box<dyn Error>> {
    if Path::new("chromedriver").exists() || Path::new("chromedriver.exe").exists() {
        info!("ChromeDriver already exists!");
    } else {
        info!("ChromeDriver does not exist! Fetching...");

        fetch_chromedriver(Client::new()).await?;
    }

    let patched_bin = match OS {
        "linux" => "chromedriver_PATCHED",
        "macos" => "chromedriver_PATCHED",
        "windows" => "chromedriver_PATCHED.exe",
        _ => panic!("Unsupported OS!"),
    };

    match !Path::new(patched_bin).exists() {
        true => {
            info!("Starting ChromeDriver executable patch...");

            let chromedriver_bin = if cfg!(windows) {
                "chromedriver.exe"
            } else {
                "chromedriver"
            };

            patch(&chromedriver_bin, &patched_bin)?;
        }
        false => {
            info!("Detected patched chromedriver executable!");
        }
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        let mut perms = fs::metadata(patched_bin)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(patched_bin, perms)?;
    }

    info!("Starting chromedriver...");
    let port: usize = rand::thread_rng().gen_range(2000..5000);

    Command::new(format!("./{}", patched_bin))
        .arg(format!("--port={}", port))
        .spawn()?;

    let mut driver = None;
    let mut attempt = 0;

    let url = format!("http://localhost:{}", port);
    let caps = get_chrome_caps()?;

    while driver.is_none() && attempt < 20 {
        attempt += 1;
        match WebDriver::new(&url, caps.clone()).await {
            Ok(d) => driver = Some(d),
            Err(_) => tokio::time::sleep(Duration::from_millis(250)).await,
        }
    }

    Ok(driver.ok_or("failed connecting to driver")?)
}

async fn fetch_chromedriver(client: Client) -> Result<(), Box<dyn Error>> {
    let chrome_version = get_chrome_version(OS).await?;

    let arch = match (OS, ARCH) {
        ("linux", "x86_64") => "linux64",
        ("windows", "x86") => "win32",
        ("windows", "x86_64") => "win64",
        ("macos", "x86_64") => "mac-x64",
        ("macos", "aarch64") => "mac-arm64",
        _ => panic!("Unsupported OS!"),
    };

    let download_url = if chrome_version.as_str() > "114" {
        // Fetch the correct version
        let url = "https://googlechromelabs.github.io/chrome-for-testing/latest-versions-per-milestone.json";
        let body = client.get(url).send().await?.bytes().await?;

        let json = serde_json::from_slice::<serde_json::Value>(&body)?;
        let version = json["milestones"][chrome_version]["version"]
            .as_str()
            .ok_or("cannot find version in latest-versions-per-milestone.json")?;

        // Fetch the chromedriver binary
        format!("https://edgedl.me.gvt1.com/edgedl/chrome/chrome-for-testing/{version}/{arch}/chromedriver-{arch}.zip")
    } else {
        let url =
            format!("https://chromedriver.storage.googleapis.com/LATEST_RELEASE_{chrome_version}");
        let body = client.get(url).send().await?.text().await?;

        format!("https://chromedriver.storage.googleapis.com/{body}/chromedriver_{arch}.zip")
    };

    let body = client.get(&download_url).send().await?.bytes().await?;

    let mut archive = ZipArchive::new(io::Cursor::new(body))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = file.mangled_name();

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            let outpath_relative = outpath.file_name().ok_or("cannot find zip file name")?;
            let mut outfile = fs::File::create(outpath_relative)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(())
}

async fn get_chrome_version(os: &str) -> Result<String, Box<dyn Error>> {
    info!("Getting installed Chrome version...");

    let command = match os {
        "linux" => Command::new("/usr/bin/google-chrome")
            .arg("--version")
            .output()?,
        "macos" => Command::new("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome")
            .arg("--version")
            .output()?,
        "windows" => Command::new("powershell")
            .arg("-c")
            .arg("(Get-Item 'C:/Program Files/Google/Chrome/Application/chrome.exe').VersionInfo")
            .output()?,
        _ => panic!("Unsupported OS!"),
    };

    let output = String::from_utf8(command.stdout)?;

    let version = output
        .lines()
        .flat_map(|line| line.chars().filter(|&ch| ch.is_ascii_digit()))
        .take(3)
        .collect::<String>();

    info!("Currently installed Chrome version: {}", version);
    Ok(version)
}

/// Patch chrome driver
pub fn patch<P: AsRef<Path> + std::marker::Copy>(
    source: P,
    target: P,
) -> Result<(), Box<dyn Error>> {
    let f = fs::read(source)?;

    let mut new_chromedriver_bytes = f.clone();
    let mut total_cdc = String::new();
    let mut cdc_pos_list = Vec::new();
    let mut is_cdc_present = false;
    let mut patch_ct = 0;

    for i in 0..f.len() - 3 {
        if "cdc_"
            == format!(
                "{}{}{}{}",
                f[i] as char,
                f[i + 1] as char,
                f[i + 2] as char,
                f[i + 3] as char
            )
            .as_str()
        {
            let range = i + 4..i + 22;
            for x in range {
                total_cdc.push_str(&(f[x] as char).to_string());
            }

            is_cdc_present = true;
            cdc_pos_list.push(i);
            total_cdc = String::from("");
        }
    }

    match is_cdc_present {
        true => info!("Found cdcs!"),
        false => info!("No cdcs were found!"),
    }

    let get_random_char = || -> char {
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
            .chars()
            .collect::<Vec<char>>()[rand::thread_rng().gen_range(0..48)]
    };

    for i in cdc_pos_list {
        let range = i + 4..i + 22;
        for x in range {
            new_chromedriver_bytes[x] = get_random_char() as u8;
        }
        patch_ct += 1;
    }

    info!("Patched {patch_ct} cdcs!");

    info!("Starting to write to binary file...");

    let _ = fs::File::create(target)?;

    match fs::write(target, new_chromedriver_bytes) {
        Ok(_) => {
            info!("Successfully wrote patched executable to 'chromedriver_PATCHED'!")
        }
        Err(err) => error!("Error when writing patch to file! Error: {err}"),
    };

    Ok(())
}

#[async_trait::async_trait]
pub trait ChromeDriver {
    async fn web_driver() -> Result<WebDriver, Box<dyn Error>>;
    async fn bypass_cloudflare(&self, url: &str) -> Result<(), Box<dyn Error>>;
    async fn goto(&self, url: &str) -> Result<(), Box<dyn Error>>;
}

#[async_trait::async_trait]
impl ChromeDriver for WebDriver {
    async fn web_driver() -> Result<WebDriver, Box<dyn Error>> {
        chrome_driver().await
    }

    /// Special goto handling for cloudflare
    async fn goto(&self, url: &str) -> Result<(), Box<dyn Error>> {
        self.execute(&format!(r#"window.open("{}", "_blank");"#, url), vec![])
            .await?;

        tokio::time::sleep(Duration::from_secs(3)).await;

        let first_window = self
            .windows()
            .await?
            .first()
            .ok_or("Unable to get first windows")?
            .clone();

        self.switch_to_window(first_window).await?;
        self.close_window().await?;

        let first_window = self
            .windows()
            .await?
            .last()
            .ok_or("Unable to get last windows")?
            .clone();

        self.switch_to_window(first_window).await?;

        Ok(())
    }

    async fn bypass_cloudflare(&self, url: &str) -> Result<(), Box<dyn Error>> {
        self.goto(url).await?;

        self.enter_frame(0).await?;

        let button = self.find(By::Css("#challenge-stage")).await?;

        button.wait_until().clickable().await?;

        tokio::time::sleep(Duration::from_secs(2)).await;

        button.click().await?;
        Ok(())
    }
}
