use super::person_dto::CreatePersonRequest;
use crate::domain::entities::person::PersonData;
use crate::AppState;
use crate::application::use_cases::create_person::CreatePersonCommand;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::{Json, http::StatusCode};
use uuid::Uuid;

#[axum::debug_handler]
pub async fn create_person(
    State(use_case): State<AppState>,
    Json(payload): Json<CreatePersonRequest>,
) -> impl IntoResponse {
    // Simulação: Em um cenário real, extrairíamos do JWT/Middleware
    let tenant_id = Uuid::parse_str("150ceece-6c86-47c1-8e99-353d43dd9abc").unwrap();
    let user_id = Uuid::parse_str("150ceece-6c86-47c1-8e99-353d43dd9abc").unwrap();

    let domain_data: PersonData = match payload.data.try_into() {
        Ok(data) => data,
        Err(e) => return (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    };

    let command = CreatePersonCommand {
        tenant_id,
        name: payload.name,
        created_by: user_id,
        data: domain_data,
    };

    match use_case.execute(command).await {
        Ok(person) => (StatusCode::CREATED, Json(person)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
