#[cfg(test)]
mod tests {
    use thirtyfour::prelude::ElementQueryable;
    use thirtyfour::By;
    use undetected_chromedriver::chrome;

    #[tokio::test]
    async fn test_headless_detection() {
        let driver = chrome().await.unwrap();
        driver
            .goto("https://arh.antoinevastel.com/bots/areyouheadless")
            .await
            .unwrap();
        let is_headless = driver.query(By::XPath(r#"//*[@id="res"]/p"#));
        assert_eq!(
            is_headless.first().await.unwrap().text().await.unwrap(),
            "You are not Chrome headless"
        );
        driver.quit().await.unwrap();
    }
}
