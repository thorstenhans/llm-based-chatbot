use serde::{Deserialize, Serialize};
use spin_sdk::variables;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct AppConfig {
    pub openai_base_url: String,
    pub openai_api_key: String,
    pub model_name: String,
}


impl AppConfig {
    pub async fn try_from_variables() ->anyhow::Result<Self> {
        let openai_base_url = variables::get("openai_base_url").await?;
        let openai_api_key = variables::get("openai_api_key").await?;
        let model_name = variables::get("openai_model").await?;

        Ok(AppConfig {
            openai_base_url,
            openai_api_key,
            model_name,
        })
    }
}