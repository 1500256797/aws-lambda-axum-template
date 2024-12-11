use std::env;

use aws_sdk_dynamodb::Config;

#[derive(Clone, Debug)]
pub struct AppState {
    pub dynamo_config: Config,
}

impl AppState {
    pub async fn new() -> Self {
        let dynamo_config = create_local_client().await;
        AppState { dynamo_config }
    }
}

async fn create_local_client() -> Config {
    env::set_var("AWS_ACCESS_KEY_ID", "DEMO");
    env::set_var("AWS_SECRET_ACCESS_KEY", "DEMO");
    env::set_var("AWS_SESSION_TOKEN", "DEMO");
    env::set_var("AWS_DEFAULT_REGION", "eu-west-1");

    let sdk_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;

    aws_sdk_dynamodb::config::Builder::from(&sdk_config)
        .endpoint_url("http://localhost:8000")
        .build()
}
