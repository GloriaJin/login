// main.rs
use bcrypt::{hash, verify, DEFAULT_COST};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::form::{self, Form};
use rocket::http::{Cookie, Cookies, Status};
use rocket::request::FlashMessage;
use rocket::response::{self, Flash, Redirect};
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::State;
use rocket_sync_db_pools::{database, Pool, sqlite};

#[derive(Serialize, Deserialize, FromForm)]
struct User {
    username: String,
    password: String,
}

#[database("sqlite_database")]
struct DbConn(sqlite::SqliteConnection);

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct DbUser {
    id: i32,
    username: String,
    password_hash: String,
    profile_picture: String,
    hours: i32,
}

#[post("/login", data = "<user_form>")]
async fn login(
    user_form: Form<User>,
    conn: DbConn,
    mut cookies: Cookies<'_>,
) -> Result<Flash<Redirect>, Status> {
    let user_form = user_form.into_inner();
    let db_user = sqlx::query_as::<_, DbUser>("SELECT * FROM users WHERE username = ?")
        .bind(&user_form.username)
        .fetch_optional(&conn)
        .await
        .map_err(|_| Status::InternalServerError)?;

    match db_user {
        Some(user) => {
            if verify(&user_form.password, &user.password_hash).map_err(|_| Status::InternalServerError)? {
                cookies.add_private(Cookie::new("user_id", user.id.to_string()));
                Ok(Flash::success(Redirect::to("/profile"), "Login successful!"))
            } else {
                Err(Status::Unauthorized)
            }
        }
        None => Err(Status::Unauthorized),
    }
}

#[get("/logout")]
fn logout(mut cookies: Cookies<'_>) -> Flash<Redirect> {
    cookies.remove_private(Cookie::named("user_id"));
    Flash::success(Redirect::to("/"), "Logout successful!")
}

#[get("/profile")]
async fn profile(conn: DbConn, cookies: &Cookies<'_>) -> Option<Json<DbUser>> {
    let user_id_cookie = cookies.get_private("user_id")?.value().to_string();
    let user_id: i32 = user_id_cookie.parse().ok()?;

    let db_user = sqlx::query_as::<_, DbUser>("SELECT * FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_optional(&conn)
        .await
        .ok()?;

    db_user.map(Json)
}

#[derive(Debug, Serialize)]
struct TransitTimesResponse {
    // structure
    // properties
}

#[get("/transit_times")]
async fn transit_times() -> Json<TransitTimesResponse> {
    // get transit times here
    let response = TransitTimesResponse {
        // actual data here 
    };

    Json(response)
}

#[get("/")]
fn index(flash: Option<FlashMessage<'_>>) -> String {
    let message = flash.map_or_else(|| String::new(), FlashMessage::msg);

    format!(
        r#"
        <html>
        <head>
            <title>Rust Web App</title>
        </head>
        <body>
            <h1>Welcome to the Rust Web App!</h1>
            <p>{}</p>
            <a href="/login">Login</a>
        </body>
        </html>
        "#,
        message
    )
}

fn main() {
    rocket::ignite()
        .attach(DbConn::fairing())
        .mount("/", routes![index, login, logout, profile, transit_times])
        .launch();
}
