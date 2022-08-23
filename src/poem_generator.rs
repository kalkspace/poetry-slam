use std::error::Error;

const BACKEND_VAR: &str = "POETRY_SLAM_BACKEND";

pub async fn generate(training_data: &str) -> Result<String, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let base_url = std::env::var(BACKEND_VAR)
        .map_err(|e| format!("{} while trying to fetch {} variable", e, BACKEND_VAR))?;
    let res = client.post(base_url).json(training_data).send().await?;

    Ok(res.text().await?)
}
