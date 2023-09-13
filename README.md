<h1 align="center">
  <img alt="rust undetected chrome driver logo" src=".github/images/logo.png" width="160px"/><br/>
  Rust undetected chromedriver
</h1>

A rust implementation of ultrafunkamsterdam's [undetected-chromedriver](https://github.com/ultrafunkamsterdam/undetected-chromedriver) library based on [thirtyfour](https://github.com/stevepryde/thirtyfour)

## Installation

To use this library, you will need to have Rust and Cargo installed on your system. You can then add the following line to your `Cargo.toml` file:

```toml
[dependencies]
undetected-chromedriver = "0.1.2"
```

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

### Headless mode

You can run the chromedriver in headless mode by using `xvfb-run`. This will require you to have `xvfb` installed on your system.

### Docker

A docker image is provided with chrome and xvfb installed. You can use it as follows:

```Dockerfile
FROM rust:latest as builder
COPY ./src ./src
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
RUN cargo build --release

FROM ghcr.io/ulyssedev/rust-undetected-chromedriver:latest
COPY --from=builder /target/release/binary /home/apps/binary
CMD ["/home/apps/binary"]
```