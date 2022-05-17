#[macro_use]
extern crate rocket;

use rocket::data::{Data, ToByteUnit};
use rocket::fairing::{self, AdHoc};
use rocket::figment::{
    providers::Toml,
    providers::{Env, Format},
    Figment,
};
use rocket::{Build, Rocket};
use rocket_db_pools::sqlx::{self, Row};
use rocket_db_pools::{Connection, Database};
use uuid::Uuid;

#[derive(Database)]
#[database("pastes")]
struct Pastes(sqlx::SqlitePool);

#[get("/<id>")]
async fn get_paste(mut db: Connection<Pastes>, id: &str) -> Option<String> {
    sqlx::query("SELECT paste FROM pastes WHERE id = ?")
        .bind(id)
        .fetch_one(&mut *db)
        .await
        .and_then(|p| Ok(p.try_get(0)?))
        .ok()
}

#[get("/")]
async fn usage() -> &'static str {
    return "Usage:

    GET /
        This usage page!
    
    GET /<id>
        Returns the paste with the given <id>.
        
    POST /
        Uploads the posted data as a paste. Response is the <id> of the paste.";
}

#[post("/", data = "<paste>")]
async fn upload(
    mut db: Connection<Pastes>,
    paste: Data<'_>,
) -> Result<String, rocket::http::Status> {
    let paste = paste
        .open(10.kibibytes())
        .into_string()
        .await
        .map_err(|_| rocket::http::Status::InternalServerError)
        .and_then(|s| {
            if s.is_complete() {
                Ok(s.into_inner())
            } else {
                Err(rocket::http::Status::BadRequest)
            }
        })?;

    let id = Uuid::new_v4().to_string();

    sqlx::query("INSERT INTO pastes(id, paste) VALUES (?, ?)")
        .bind(&id)
        .bind(&paste)
        .execute(&mut *db)
        .await
        .map_err(|_| rocket::http::Status::InternalServerError)?;

    Ok(id)
}

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    match Pastes::fetch(&rocket) {
        Some(db) => match sqlx::migrate!("./migrations").run(&**db).await {
            Ok(_) => Ok(rocket),
            Err(e) => {
                error!("Failed to run migrations on SQLx database: {}", e);
                Err(rocket)
            }
        },
        None => Err(rocket),
    }
}

#[launch]
fn rocket() -> _ {
    let figment = Figment::from(rocket::Config::default())
        .merge(Toml::file("Rocket.toml").nested())
        .merge((
            "port",
            Env::var_or("PORT", "8000")
                .parse::<u16>()
                .expect("PORT is not a valid port number"),
        ));

    rocket::custom(figment)
        .attach(Pastes::init())
        .attach(AdHoc::try_on_ignite("SQLx Migrations", run_migrations))
        .mount("/", routes![usage, get_paste, upload])
}
