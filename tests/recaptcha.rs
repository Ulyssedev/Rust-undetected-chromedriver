#[cfg(test)]
mod tests {
    use std::error::Error;
    use thirtyfour::{
        prelude::{ElementQueryable, ElementWaitable},
        By,
    };
    use undetected_chromedriver::chrome_driver;

    async fn get_score(driver: &thirtyfour::WebDriver) -> Result<f32, Box<dyn Error>> {
        driver
            .goto("https://recaptcha-demo.appspot.com/recaptcha-v3-request-scores.php")
            .await?;

        let button = driver
            .query(By::XPath(r#"//*[@id="recaptcha-steps"]/li[2]/button[2]"#))
            .first()
            .await?;

        button.wait_until().clickable().await?;
        button.click().await?;

        let response = driver
            .query(By::XPath(r#"//*[@id="recaptcha-steps"]/li[5]/pre"#))
            .first()
            .await?;

        response.wait_until().displayed().await?;

        println!("reponse: {}", response.text().await?);

        let response_text = response.text().await?;
        let score = response_text
            .lines()
            .find(|line| line.contains("\"score\":"))
            .and_then(|line| {
                let start_index = line.find(':')?;
                let end_index = line.find(',')?;
                line.get(start_index + 1..end_index)
            })
            .and_then(|score_str| score_str.trim().parse::<f32>().ok());

        Ok(score.ok_or("cannot find score")?)
    }

    #[tokio::test]
    async fn recaptcha() -> Result<(), Box<dyn Error>> {
        let driver = chrome_driver().await?;
        let score = get_score(&driver).await;
        assert!(score.unwrap_or(0.0) >= 0.7);
        driver.quit().await?;

        Ok(())
    }
}
