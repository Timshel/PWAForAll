#[macro_use]
extern crate rocket;
use base64::{Engine as _, engine::general_purpose::URL_SAFE};
use rocket::response::Redirect;
use rocket::response::status::{BadRequest, NotFound};
use rocket::serde::Deserialize;
use rocket::serde::json::serde_json;
use rocket::serde::json::serde_json::{Value, json};
use rocket_dyn_templates::{Template, context};

#[get("/favicon.ico")]
fn favicon() -> NotFound<String> {
    NotFound("No favicon".to_string())
}

#[get("/")]
fn homepage() -> Template {
    Template::render(
        "index",
        context! {
            foo: 123,
        },
    )
}

#[derive(rocket::FromForm)]
struct FormData {
    manifest: String,
}

#[rocket::post("/", data = "<data>")]
fn form_submit(data: rocket::form::Form<FormData>) -> Result<Redirect, BadRequest<String>> {
    match serde_json::from_str::<Value>(&data.manifest) {
        Err(e) => Err(BadRequest(format!("Failed to parse manifest: {e}"))),
        Ok(v) => {
            let encoded = URL_SAFE.encode(v.to_string());
            Ok(Redirect::to(format!("/{encoded}")))
        }
    }
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct Manifest {
    name: String,
    start_url: String,
}

#[get("/<encoded>")]
fn pwa(encoded: &str) -> Result<Template, BadRequest<String>> {
    let manifest = match URL_SAFE.decode(encoded) {
        Err(e) => Err(BadRequest(format!("Failed to decode manifest: {e}"))),
        Ok(m) => Ok(String::from_utf8_lossy(&m).into_owned()),
    }?;

    match serde_json::from_str::<Manifest>(&manifest) {
        Err(e) => Err(BadRequest(format!("Failed to parse manifest: {e}"))),
        Ok(m) => Ok(Template::render("pwa", context! { base_url: m.start_url, manifest: manifest, name: m.name })),
    }
}

#[get("/<encoded>?mode=standalone")]
fn pwa_redirect(encoded: &str) -> Result<Redirect, BadRequest<String>> {
    let manifest = match URL_SAFE.decode(encoded) {
        Err(e) => Err(BadRequest(format!("Failed to decode manifest: {e}"))),
        Ok(m) => Ok(String::from_utf8_lossy(&m).into_owned()),
    }?;

    match serde_json::from_str::<Manifest>(&manifest) {
        Err(e) => Err(BadRequest(format!("Failed to parse manifest: {e}"))),
        Ok(m) => Ok(Redirect::to(m.start_url)),
    }
}

#[get("/<encoded>/manifest.json")]
fn manifest(encoded: &str) -> Result<Value, BadRequest<String>> {
    let manifest = match URL_SAFE.decode(encoded) {
        Err(e) => Err(BadRequest(format!("Failed to decode manifest: {e}"))),
        Ok(m) => Ok(String::from_utf8_lossy(&m).into_owned()),
    }?;

    match serde_json::from_str::<Value>(&manifest) {
        Err(e) => Err(BadRequest(format!("Failed to parse manifest: {e}"))),
        Ok(mut v) => {
            v["start_url"] = json!(format!("/{manifest}?mode=standalone"));
            Ok(v)
        }
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Template::fairing())
        .mount("/", routes![homepage, favicon, form_submit, manifest, pwa, pwa_redirect])
}
