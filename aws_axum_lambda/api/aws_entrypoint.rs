use aws_axum_lambda::setup_server;
use lambda_http::{run, Error};
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing::Level;
mod aws_layer;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_max_level(Level::DEBUG)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .json()
        .init();

    // 日志
    info!("===========app===========");
    // 设置 Axum 应用
    let app = setup_server().await.layer(TraceLayer::new_for_http());
    info!("服务器路由设置完成");
    // 使用我们的 AWS Lambda 层
    let app = tower::ServiceBuilder::new()
        .layer(aws_layer::LambdaLayer::default().trim_stage())
        .service(app);

    // 使用 lambda_http 运行，它会自动处理 API Gateway 事件
    run(app).await?;
    Ok(())
}
