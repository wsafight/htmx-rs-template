use askama::Template;
use askama_axum::IntoResponse;

#[derive(Template)]
#[template(path = "modal/example.html")]
pub struct ModalExampleTemplate;

pub async fn example() -> impl IntoResponse {
    ModalExampleTemplate
}
