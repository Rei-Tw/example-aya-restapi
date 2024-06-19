use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

/// Gestion d'erreur HTTP simplifiée. Permet de retourner le bon code d'erreur avec un message d'erreur.
///
/// Le message d'erreur sera dans le body de la réponse, sous le format JSON.
#[derive(Debug)]
pub enum ApiError {
    InternalServerError,
}

impl IntoResponse for ApiError {
    /// Retourne un tuple contenant le status code et le message d'erreur selon le type défini (voir l'enum).
    fn into_response(self) -> axum::response::Response {
        let (status, err_msg) = match self {
            Self::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An internal server error occured.",
            ),
        };
        (status, Json(json!({ "error": err_msg }))).into_response()
    }
}
