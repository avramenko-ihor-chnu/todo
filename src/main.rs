const DB_URL: &str = "sqlite://sqlite.db";
use askama::Template;
use axum::{
    Router,
    extract::{Form, State},
    http::StatusCode,
    response::Html,
    routing::{get, post},
};
use serde::Deserialize;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use tokio::fs;

#[derive(Clone)]
struct Database {
    pool: SqlitePool,
}

#[derive(Deserialize)]
struct TodoForm {
    title: String,
}

#[derive(askama::Template)]
#[template(path = "todos-table.html")]
struct TodosTable {
    rows: Vec<TodosRow>,
}

#[derive(sqlx::FromRow)]
struct TodosRow {
    id: i64,
    title: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = SqlitePoolOptions::new().connect(DB_URL).await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS todos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    let database = Database { pool };

    let app = Router::new()
        .route("/", get(index))
        .route("/todos", get(todos))
        .route("/add-todo", post(add_todo))
        .with_state(database);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Server running on http://localhost:3000");

    axum::serve(listener, app).await?;
    Ok(())
}

async fn index() -> Result<Html<String>, StatusCode> {
    let index = fs::read_to_string("templates/index.html")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Html(index))
}

async fn todos(State(database): State<Database>) -> Result<Html<String>, StatusCode> {
    let rows = sqlx::query_as::<_, TodosRow>("SELECT id, title FROM todos ORDER BY id")
        .fetch_all(&database.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let table = TodosTable { rows };
    let html = table
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Html(html))
}

async fn add_todo(
    State(database): State<Database>,
    Form(form): Form<TodoForm>,
) -> Result<Html<String>, StatusCode> {
    // Insert new todo
    sqlx::query("INSERT INTO todos (title) VALUES (?)")
        .bind(&form.title)
        .execute(&database.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    todos(State(database)).await
}
