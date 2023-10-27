#[cfg(test)]
mod tests {
    use std::error::Error;
    use undetected_chromedriver::chrome_driver;

    #[tokio::test]
    async fn test_chrome() -> Result<(), Box<dyn Error>> {
        let driver = chrome_driver().await?;
        assert!(driver.title().await.is_ok());
        driver.quit().await?;

        Ok(())
    }
}
