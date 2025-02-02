use crate::features::todo::model::{Error, Todo};

use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::{Client, Config};
use std::collections::HashMap;
use tracing::info;
pub struct TodoRepository {
    client: Client,
    table_name: String,
}

impl TodoRepository {
    pub async fn new(config: Config) -> Self {
        let table_name = String::from("TodoTable");
        info!(
            "Initializing DynamoDB store with table name: {}",
            table_name
        );

        let client = Client::from_conf(config);
        TodoRepository { client, table_name }
    }

    pub async fn get_all(&self) -> Result<Vec<Todo>, Error> {
        let req = self.client.scan().table_name(&self.table_name).limit(20);

        let result = req
            .send()
            .await
            .map(|result| result.items.map(|item| todo_list_mapper(item)))?;

        Ok(result.unwrap())
    }

    pub async fn get_todo(&self, id: &str) -> Result<Option<Todo>, Error> {
        // Example for Shadowing
        let id = AttributeValue::S(id.to_string());

        let req = self
            .client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("id = :id")
            .expression_attribute_values(":id", id)
            .limit(2);

        let result = req
            .send()
            .await
            .map(|result| result.items.map(|item| todo_list_mapper(item)))?;
        let todos = result.unwrap();
        if todos.len() > 1 {
            panic!("More than one item found");
        }

        Ok(todos.get(0).map(|x| x.to_owned()))
    }

    pub async fn insert_todo(&self, todo: Todo) -> Result<(), Error> {
        let id = AttributeValue::S(todo.id);
        let title = AttributeValue::S(todo.title);
        let description = AttributeValue::S(todo.description);
        let created = AttributeValue::N(todo.created.timestamp_millis().to_string());

        let request = self
            .client
            .put_item()
            .table_name(&self.table_name)
            .item("id", id)
            .item("title", title)
            .item("description", description)
            .item("created", created);

        request.send().await?;

        Ok(())
    }

    pub async fn update_todo(&self, todo: Todo) -> Result<(), Error> {
        let id = AttributeValue::S(todo.id);
        let title = AttributeValue::S(todo.title);
        let description = AttributeValue::S(todo.description);

        let request = self
            .client
            .update_item()
            .table_name(&self.table_name)
            .key("id", id)
            .update_expression("set title = :title, description = :description")
            .expression_attribute_values(":title", title)
            .expression_attribute_values(":description", description);

        request.send().await?;

        Ok(())
    }

    pub async fn delete_todo(&self, id: &str) -> Result<(), Error> {
        let id = AttributeValue::S(id.to_string());

        let req = self
            .client
            .delete_item()
            .table_name(&self.table_name)
            .key("id", id);

        req.send().await?;

        Ok(())
    }
}

fn todo_mapper(data: HashMap<String, AttributeValue>) -> Todo {
    let id = data.get("id").unwrap().as_s().unwrap().clone();
    let title = data.get("title").unwrap().as_s().unwrap().clone();
    let description = data.get("description").unwrap().as_s().unwrap().clone();
    let created = data.get("created").unwrap().as_n().unwrap().clone();
    let created_n = created
        .parse::<i64>()
        .expect(&*format!("unparsable DATE/TIME for {}", id));
    Todo {
        id,
        title,
        description,
        created: chrono::DateTime::from_timestamp_millis(created_n).unwrap(),
    }
}

fn todo_list_mapper(data: Vec<HashMap<String, AttributeValue>>) -> Vec<Todo> {
    data.iter().map(|item| todo_mapper(item.clone())).collect()
}

#[cfg(test)]
mod tests {
    use crate::features::todo::model::Todo;
    use crate::features::todo::repo::TodoRepository;
    use aws_sdk_dynamodb::types::{
        AttributeDefinition, KeySchemaElement, KeyType, ProvisionedThroughput, ScalarAttributeType,
    };
    use aws_sdk_dynamodb::*;
    use chrono::Utc;
    use std::env;
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

    // create table
    #[tokio::test]
    async fn dynamo_db_create_table() {
        let config = create_local_client().await;
        let client = Client::from_conf(config);

        let request = client
            .create_table()
            .table_name("TodoTable")
            // 定义属性定义
            .attribute_definitions(
                AttributeDefinition::builder()
                    .attribute_name("id")
                    .attribute_type(ScalarAttributeType::S)
                    .build()
                    .unwrap(),
            )
            // 定义键架构
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name("id")
                    .key_type(KeyType::Hash)
                    .build()
                    .unwrap(),
            )
            // 设置预置吞吐量
            .provisioned_throughput(
                ProvisionedThroughput::builder()
                    .read_capacity_units(1)
                    .write_capacity_units(1)
                    .build()
                    .unwrap(),
            )
            .send()
            .await;

        println!("{:?}", request);
    }

    #[tokio::test]
    async fn dynamo_db_get_tables() {
        let config = create_local_client().await;
        let client = Client::from_conf(config);

        let paginator = client.list_tables().into_paginator().items().send();
        let table_names = paginator.collect::<Result<Vec<_>, _>>().await;

        println!("Tables:");

        for name in &table_names {
            println!(" - {:?}", name);
        }
    }

    #[tokio::test]
    async fn dynamo_db_get_todo_list() {
        let client = create_local_client().await;

        let todo_repository = TodoRepository::new(client).await;
        let todos = todo_repository.get_all().await.unwrap();

        println!("{:?}", todos);
    }

    #[tokio::test]
    async fn dynamo_db_insert_todo() {
        let client = create_local_client().await;

        let todo_repository = TodoRepository::new(client).await;

        let mut todo = Todo::new(
            String::from("Title"),
            String::from("Description"),
            Utc::now(),
        );
        todo.id = String::from("1");

        todo_repository.insert_todo(todo).await.unwrap();
    }

    #[tokio::test]
    async fn dynamo_db_update_todo() {
        let client = create_local_client().await;

        let todo_repository = TodoRepository::new(client).await;

        let mut todo = Todo::new(
            String::from("Title (updated)"),
            String::from("Description (updated)"),
            Utc::now(),
        );
        todo.id = String::from("1");

        todo_repository.update_todo(todo).await.unwrap();
    }

    #[tokio::test]
    async fn dynamo_db_get_todo() {
        let client = create_local_client().await;
        let todo_repository = TodoRepository::new(client).await;
        let todo = todo_repository.get_todo("1").await.unwrap();

        assert!(todo.is_some());
        let todo = todo.unwrap();

        println!("{:?}", todo);
    }

    #[tokio::test]
    async fn dynamo_db_delete_todo() {
        let client = create_local_client().await;

        let todo_repository = TodoRepository::new(client).await;

        todo_repository.delete_todo("1").await.unwrap();
    }
}
