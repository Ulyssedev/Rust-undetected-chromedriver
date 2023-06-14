use rand::Rng;
use thirtyfour::{WebDriver, DesiredCapabilities};
use std::{process::Command};
#[cfg(target_os = "linux")]
use std::os::unix::fs::PermissionsExt;

/// Fetches a new ChromeDriver executable and patches it to prevent detection.
/// Returns a WebDriver instance.
pub async fn chrome() -> Result<WebDriver, Box<dyn std::error::Error>> {
    let os = std::env::consts::OS;
    if std::path::Path::new("chromedriver").exists() {
        println!("ChromeDriver already exists!");
    } else {
        println!("ChromeDriver does not exist! Fetching...");
        let client = reqwest::Client::new();
        fetch_chromedriver(&client).await.unwrap();
    }
    let chromedriver_executable = match os {
        "linux" => "chromedriver_PATCHED",
        "macos" => "chromedriver_PATCHED",
        "windows" => "chromedriver_PATCHED.exe",
        _ => panic!("Unsupported OS!"),
    };
    match !std::path::Path::new(chromedriver_executable).exists() {
        true => {
            println!("Starting ChromeDriver executable patch...");
            let file_name = if cfg!(windows) { "chromedriver.exe" } else { "chromedriver" };
            let f = std::fs::read(file_name).unwrap();
            let mut new_chromedriver_bytes = f.clone();
            let mut total_cdc = String::from("");
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
                    for x in i..i + 22 {
                        total_cdc.push_str(&(f[x] as char).to_string());
                    }
                    is_cdc_present = true;
                    cdc_pos_list.push(i);
                    total_cdc = String::from("");
                }
            }
            match is_cdc_present {
                true => println!("Found cdcs!"),
                false => println!("No cdcs were found!"),
            }
            let get_random_char = || -> char {
                "abefghijklmnopqrstuvwxyzABEFGHIJKLMNOPQRSTUVWXYZ"
                    .chars()
                    .collect::<Vec<char>>()[rand::thread_rng().gen_range(0..48)]
            };
            for i in cdc_pos_list {
                for x in i..i + 22 {
                    new_chromedriver_bytes[x] = get_random_char() as u8;
                }
                patch_ct += 1;
            }
            println!("Patched {} cdcs!", patch_ct);

            println!("Starting to write to binary file...");
            let _file = std::fs::File::create(chromedriver_executable).unwrap();
            match std::fs::write(chromedriver_executable, new_chromedriver_bytes) {
                Ok(_res) => println!(
                    "Successfully wrote patched executable to 'chromedriver_PATCHED'!",
                ),
                Err(err) => println!("Error when writing patch to file! Error: {}", err),
            };
        }
        false => {
            println!("Detected patched chromedriver executable!");
        }
    }
    #[cfg(target_os = "linux")]
    {
        let mut perms = std::fs::metadata(chromedriver_executable)
            .unwrap()
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(chromedriver_executable, perms).unwrap();
    }
    println!("Starting chromedriver...");
    let port: usize = rand::thread_rng().gen_range(2000..5000);
    Command::new(format!("./{}", chromedriver_executable))
        .arg(format!("--port={}", port))
        .spawn()
        .expect("Failed to start chromedriver!");


    let mut caps = DesiredCapabilities::chrome();
    caps.add_chrome_arg("--no-sandbox").unwrap();
    caps.add_chrome_arg("--disable-blink-features=AutomationControlled")
        .unwrap();
    caps.add_chrome_arg("window-size=1920,1080").unwrap();
    caps.add_chrome_arg("user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/102.0.0.0 Safari/537.36").unwrap();
    caps.add_chrome_arg("disable-infobars").unwrap();
    caps.add_chrome_option("excludeSwitches", ["enable-automation"])
        .unwrap();
    let driver = WebDriver::new(&format!("http://localhost:{}", port), caps)
        .await
        .unwrap();
    Ok(driver)
}

async fn fetch_chromedriver(client: &reqwest::Client) -> Result<(), Box<dyn std::error::Error>> {
    let os = std::env::consts::OS;
    let resp = client
        .get("https://chromedriver.storage.googleapis.com/LATEST_RELEASE")
        .send()
        .await?;
    let body = resp.text().await?;
    let url = match os {
        "linux" => format!(
            "https://chromedriver.storage.googleapis.com/{}/chromedriver_linux64.zip",
            body
        ),
        "windows" => format!(
            "https://chromedriver.storage.googleapis.com/{}/chromedriver_win32.zip",
            body
        ),
        "macos" => format!(
            "https://chromedriver.storage.googleapis.com/{}/chromedriver_mac64.zip",
            body
        ),
        _ => panic!("Unsupported OS!"),
    };
    let resp = client
        .get(url)
        .send()
        .await?;
    let body = resp.bytes().await?;

    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(body))?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = file.mangled_name();
        if (&*file.name()).ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(&p)?;
                }
            }
            let mut outfile = std::fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }
    Ok(())
}