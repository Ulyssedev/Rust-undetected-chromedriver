#[cfg(test)]
mod tests {
    use thirtyfour::prelude::ElementQueryable;
    use thirtyfour::By;
    use undetected_chromedriver::Chrome;
    use thirtyfour::WebDriver;

    #[tokio::test]
    async fn test_cloudflare() {
        let driver: WebDriver = Chrome::new().await;
        driver.bypass_cloudflare("https://nowsecure.nl").await.unwrap();
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        println!("{}", driver.source().await.unwrap());
        let passed = driver.query(By::XPath("/html/body/div[2]/div/main/p[1]"));
        assert_eq!(passed.first().await.unwrap().text().await.unwrap(), "you passed!");
        driver.quit().await.unwrap();
    }
}