#[cfg(test)]
mod tests {
    use std::error::Error;
    use thirtyfour::{prelude::ElementQueryable, By};
    use undetected_chromedriver::chrome_driver;

    #[tokio::test]
    async fn test_headless_detection() -> Result<(), Box<dyn Error>> {
        let driver = chrome_driver().await?;
        driver
            .goto("https://arh.antoinevastel.com/bots/areyouheadless")
            .await?;
        let is_headless = driver.query(By::XPath(r#"//*[@id="res"]/p"#));
        assert_eq!(
            is_headless.first().await.unwrap().text().await?,
            "You are not Chrome headless"
        );
        driver.quit().await?;

        Ok(())
    }
}
