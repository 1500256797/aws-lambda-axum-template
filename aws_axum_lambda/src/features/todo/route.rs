use crate::{features::todo::model::Todo, state::AppState};
use crate::{features::todo::repo::TodoRepository, structs::ApiResponse};
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::instrument;

// define router
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/getTodos", get(get_todos))
        .route("/addTodo", post(add_todo))
}

#[instrument()]
pub async fn get_todos(State(state): State<AppState>) -> Json<ApiResponse<Vec<Todo>>> {
    tracing::info!("==========get_todos");
    let todo_repo = TodoRepository::new(state.dynamo_config).await;
    let todos = todo_repo.get_all().await;
    match todos {
        Ok(todos) => {
            println!("=======todos: {:?}", todos);
            let resp = ApiResponse::new(StatusCode::OK.into(), "success".to_string(), todos);
            Json(resp)
        }
        Err(e) => {
            tracing::error!("error: {:?}", e);
            Json(ApiResponse::new(
                StatusCode::BAD_REQUEST.into(),
                e.to_string(),
                vec![],
            ))
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AddTodoReq {
    pub title: String,
    pub description: String,
}

#[instrument()]
pub async fn add_todo(
    State(state): State<AppState>,
    Json(params): Json<AddTodoReq>,
) -> Json<ApiResponse<String>> {
    tracing::info!("add_todo");
    // Deserialize
    let mut item = Todo::new(params.title, params.description, Utc::now());
    item.generate_id();

    let todo_repo = TodoRepository::new(state.dynamo_config).await;
    match todo_repo.insert_todo(item.clone()).await {
        Ok(_) => Json(ApiResponse::new(
            StatusCode::OK.into(),
            "success".to_string(),
            format!("Todo inserted with ID: {}", item.id),
        )),
        Err(e) => {
            tracing::error!("error: {:?}", e);
            Json(ApiResponse::new(
                StatusCode::BAD_REQUEST.into(),
                e.to_string(),
                "".to_string(),
            ))
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use lambda_http::http::{Method, Uri};
//     use lambda_http::Body;
//     use std::env;

//     #[tokio::test]
//     async fn test_get_todo_list() {
//         let request = Request::default();

//         let config = create_local_client().await;

//         let response = get_todos(config, request).await;

//         assert!(response.is_ok());
//     }

//     #[tokio::test]
//     async fn test_get_todo() {
//         let request = Request::default();
//         let (mut parts, body) = request.into_parts();

//         parts.uri = Uri::from_static("https://test-todo/Prod/get-todo/1");
//         let request = Request::from_parts(parts, body);

//         let config = create_local_client().await;

//         let response = get_todo(config, request).await;

//         assert!(response.is_ok());
//     }

//     #[tokio::test]
//     async fn test_insert_todo() {
//         let request = Request::default();
//         let (mut parts, mut body) = request.into_parts();
//         parts.method = Method::POST;
//         body = Body::Text(
//             "{
//             \"id\": \"\",
//             \"title\": \"title\",
//             \"description\": \"description\"
//         }"
//             .to_string(),
//         );
//         let request = Request::from_parts(parts, body);

//         let config = create_local_client().await;

//         let response = add_todo(config, request).await;

//         assert!(response.is_ok());
//     }

//     #[tokio::test]
//     async fn test_edit_todo() {
//         let request = Request::default();
//         let (mut parts, mut body) = request.into_parts();
//         parts.method = Method::PUT;
//         body = Body::Text(
//             "{
//             \"id\": \"b71c43f3-e362-4482-8647-c47bf245fec1\",
//             \"title\": \"title (updated)\",
//             \"description\": \"description (updated)\"
//         }"
//             .to_string(),
//         );
//         let request = Request::from_parts(parts, body);

//         let config = create_local_client().await;

//         let response = edit_todo(config, request).await;

//         assert!(response.is_ok());
//     }

//     #[tokio::test]
//     async fn test_delete_todo() {
//         let request = Request::default();
//         let (mut parts, body) = request.into_parts();
//         parts.method = Method::DELETE;
//         parts.uri = Uri::from_static(
//             "https://test-todo/Prod/get-todo/b71c43f3-e362-4482-8647-c47bf245fec1",
//         );

//         let request = Request::from_parts(parts, body);

//         let config = create_local_client().await;

//         let response = delete_todo(config, request).await;

//         assert!(response.is_ok());
//     }

//     #[test]
//     fn json_to_list() {
//         let json = "[
//         {
//             \"created\": \"2024-02-19T19:20:54.702Z\",
//             \"description\": \"description\",
//             \"id\": \"9e4f98b6-e332-478e-b3d5-6be74e5f97c7\",
//             \"title\": \"title\"
//         }
//         ]";

//         let parsed: Vec<Todo> = serde_json::from_str(&json).unwrap();

//         assert!(!parsed.is_empty());
//         println!("{:?}", parsed);
//     }

//     async fn create_local_client() -> Config {
//         env::set_var("AWS_ACCESS_KEY_ID", "DEMO");
//         env::set_var("AWS_SECRET_ACCESS_KEY", "DEMO");
//         env::set_var("AWS_SESSION_TOKEN", "DEMO");
//         env::set_var("AWS_DEFAULT_REGION", "eu-west-1");

//         let sdk_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;

//         aws_sdk_dynamodb::config::Builder::from(&sdk_config)
//             .endpoint_url("http://localhost:8000")
//             .build()
//     }
// }
