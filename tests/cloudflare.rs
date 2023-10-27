#[cfg(test)]
mod tests {
    use std::{error::Error, time::Duration};
    use thirtyfour::{prelude::ElementQueryable, By, WebDriver};
    use undetected_chromedriver::ChromeDriver;

    #[tokio::test]
    async fn test_cloudflare() -> Result<(), Box<dyn Error>> {
        let driver: WebDriver = WebDriver::web_driver().await?;

        driver.bypass_cloudflare("https://nowsecure.nl").await?;

        tokio::time::sleep(Duration::from_secs(2)).await;

        println!("{}", driver.source().await?);
        let passed = driver.query(By::XPath("/html/body/div[2]/div/main/p[1]"));

        assert_eq!(passed.first().await?.text().await?, "you passed!");

        driver.quit().await?;

        Ok(())
    }
}
