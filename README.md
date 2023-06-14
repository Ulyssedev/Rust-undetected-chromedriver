# Rust undetected chromedriver

A rust implementation of ultrafunkamsterdam's [undetected-chromedriver](https://github.com/ultrafunkamsterdam/undetected-chromedriver) library based on [thirtyfour](https://github.com/stevepryde/thirtyfour)

## Installation

To use this library, you will need to have Rust and Cargo installed on your system. You can then add the following line to your `Cargo.toml` file:

```toml
[dependencies]
undetected-chromedriver = "0.1.0"
```
*A propoer crates.io and docker release will be made available soon*

## Usage

Here's an example of how you can use the undetected chromedriver in your Rust project:

```rust
use undetected_chromedriver::chrome;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let driver = chrome().await?;

    driver.goto("https://www.rust-lang.org/").await?;

    let title = driver.title().await?;
    println!("Title: {}", title);

    driver.quit().await?;

    Ok(())
}
```
*Note: chrome needs to be installed on the system before using undetected chromedriver*