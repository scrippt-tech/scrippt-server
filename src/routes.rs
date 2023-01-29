use actix_web::{web, get, HttpResponse};
use handlebars::Handlebars;
use serde_json::json;

#[get("/")]
async fn index(hb: web::Data<Handlebars<'_>>) -> HttpResponse {
    let data = json!({
        "name": "Scrippt",
    });
    let body = hb.render("index", &data).unwrap();

    HttpResponse::Ok().body(body)
}