use askama::Template;
use askama_axum::IntoResponse;

#[derive(Template)]
#[template(path = "components/modal/base.html")]
pub struct ModalExampleTemplate;

pub async fn example() -> impl IntoResponse {
    ModalExampleTemplate
}
