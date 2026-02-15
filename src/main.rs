mod application;
mod domain;
mod infrastructure;

use crate::application::use_cases::create_person::CreatePersonUseCase;
use crate::infrastructure::adapters::input::http::person_handler;
use crate::infrastructure::adapters::output::postgres_person_repository::PostgresPersonRepository;
use axum::{Router, routing::post};
use sqlx::PgPool;
use std::sync::Arc;

pub type AppState = Arc<CreatePersonUseCase<PostgresPersonRepository>>;

#[tokio::main]
async fn main() {
    // 1. Setup Database
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await.unwrap();

    // 2. Setup Hexagonal Layers (Injeção de Dependência)
    let repository = PostgresPersonRepository::new(pool);
    let use_case = CreatePersonUseCase::new(repository);

    // Usamos Arc para compartilhar o Use Case entre as threads do Axum
    let shared_state: AppState = Arc::new(use_case);

    // 3. Setup Routes
    let app = Router::new()
        .route("/people", post(person_handler::create_person))
        .with_state(shared_state);

    // 4. Start Server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
