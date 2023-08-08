use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::File;

use axum::http::StatusCode;
use axum::response::Html;
use axum::routing::{delete, get, post};
use axum::{Extension, Form, Router, Server};
use chrono::NaiveDateTime;
use eyre::{ContextCompat, Result};
use hyro::{context, RouterExt, Template};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use tower_http::services::ServeDir;

type Hypermedia = Html<Cow<'static, str>>;
type Error = (StatusCode, String);
type MaybeHypermedia = std::result::Result<Hypermedia, Error>;

#[derive(Serialize, Deserialize, FromRow, Debug)]
struct Todo {
    id: i32,
    description: String,
    done: bool,
    created: NaiveDateTime,
    updated: NaiveDateTime,
}

#[tokio::main]
async fn main() -> Result<()> {
    hyro::config::set_template_file_extension("html.j2").unwrap();

    let db_path = std::env::temp_dir().join("hyro.db");
    if !db_path.exists() {
        File::create(&db_path)?;
    }

    let pool = SqlitePool::connect(db_path.to_str().unwrap()).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let router = Router::new()
        .route("/", get(index))
        .route("/todo", get(todo))
        .route("/todo", post(create_todo))
        .route("/todo", delete(delete_todo))
        .route("/todo-done", get(todo_done))
        .route("/todo-edit", get(todo_edit))
        .nest_service("/assets", ServeDir::new("assets"))
        .layer(Extension(pool))
        .into_service_with_hmr();

    Server::from_tcp(hyro::bind("0.0.0.0:1380").await)?
        .serve(router)
        .await?;

    Ok(())
}

async fn index(Extension(pool): Extension<SqlitePool>, template: Template) -> MaybeHypermedia {
    let todos: Vec<Todo> = sqlx::query_as("SELECT * FROM todos")
        .fetch_all(&pool)
        .await
        .map_err(internal_error)?;

    let num_done = todos
        .iter()
        .fold(0, |acc, todo| if todo.done { acc + 1 } else { acc });

    Ok(template.render(context!(todos, num_done)))
}

async fn todo(template: Template) -> Hypermedia {
    template.render(context!())
}

async fn todo_edit(template: Template) -> Hypermedia {
    template.render(context!())
}

async fn todo_done(
    Extension(pool): Extension<SqlitePool>,
    Form(form): Form<HashMap<String, String>>,
) -> Result<(), Error> {
    sqlx::query("UPDATE todos SET done = $1 WHERE id = $2")
        .bind(
            form.get("done")
                .context("please provide done")
                .map_err(bad_request)?
                == "false",
        )
        .bind(
            form.get("id")
                .context("please provide an id")
                .map_err(bad_request)?,
        )
        .execute(&pool)
        .await
        .map_err(internal_error)?;

    Ok(())
}

async fn create_todo(
    Extension(pool): Extension<SqlitePool>,
    template: Template,
) -> MaybeHypermedia {
    let id = if let (Some(description), Some(local_id)) =
        (template.1.get("description"), template.1.get("id"))
    {
        sqlx::query("UPDATE todos SET description = $1 WHERE id = $2")
            .bind(description)
            .bind(local_id)
            .execute(&pool)
            .await
            .map_err(internal_error)?;
        local_id.parse().map_err(internal_error)?
    } else {
        sqlx::query("INSERT INTO todos (description) VALUES ($1) RETURNING id")
            .bind(template.1.get("description").unwrap())
            .execute(&pool)
            .await
            .map_err(internal_error)?
            .last_insert_rowid()
    };

    let todo: Todo = sqlx::query_as("SELECT * FROM todos WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(internal_error)?;

    Ok(template.render(context!(form => todo)))
}

async fn delete_todo(
    Extension(pool): Extension<SqlitePool>,
    Form(form): Form<HashMap<String, String>>,
) -> Result<(), Error> {
    sqlx::query("DELETE FROM todos WHERE id = $1")
        .bind(
            form.get("id")
                .context("please provide an id")
                .map_err(bad_request)?,
        )
        .execute(&pool)
        .await
        .map_err(internal_error)?;

    Ok(())
}

fn bad_request(err: eyre::Error) -> (StatusCode, String) {
    (StatusCode::BAD_REQUEST, err.to_string())
}

fn internal_error<E: std::error::Error>(err: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
