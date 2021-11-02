#[macro_use]
extern crate rocket;
#[macro_use]
extern crate sqlx;
#[macro_use]
extern crate serde;
use std::{borrow::BorrowMut, sync::Arc};

use rocket::{
    form::Form,
    futures::{lock::Mutex, TryFutureExt},
    get, launch,
    response::{content::Html, Redirect},
    Build, Rocket, State,
};
use rocket_dyn_templates::Template;
use serde_json::json;
use sqlx::{Connection, SqliteConnection, SqlitePool, Statement};

type DbConn = SqlitePool;

#[derive(Debug, Serialize)]
struct Post {
    id: i64,
    poster: i64,
    title: String,
    body: String,
}

impl Post {
    pub async fn get(conn: &DbConn, id: i32) -> Option<Post> {
        query_as!(Post, "SELECT * FROM posts WHERE id = ? LIMIT 1", id)
            .fetch_one(conn)
            .await
            .ok()
    }

    pub async fn count(conn: &DbConn) -> i32 {
        query!("SELECT COUNT(*) AS count FROM posts")
            .fetch_one(conn)
            .await
            .map_or(0, |row| row.count)
    }

    pub async fn all(conn: &DbConn) -> Vec<Post> {
        query_as!(Post, "SELECT * FROM posts")
            .fetch_all(conn)
            .await
            .expect("Could not fetch posts")
    }

    pub async fn save(self, conn: &DbConn) {
        query!(
            "INSERT INTO posts VALUES ($1, $2, $3, $4)",
            self.id,
            self.poster,
            self.title,
            self.body
        )
        .execute(conn)
        .await
        .expect("Could not write to database");
    }

    pub async fn create(conn: &DbConn, title: String, body: String) -> i32 {
        query!(
            "INSERT INTO posts (poster, title, body) VALUES (0, $1, $2)",
            title,
            body
        )
        .execute(conn)
        .await
        .expect("Could not insert new row")
        .last_insert_rowid() as i32
    }
}

#[get("/")]
async fn index(conn: &State<DbConn>) -> Html<Template> {
    let posts = Post::all(conn).await;
    let context = json!({ "posts": posts });
    Html(Template::render("index", context))
}

#[derive(FromForm)]
struct PostRequest {
    title: String,
    body: String,
}

#[post("/", data = "<request>")]
async fn create_post(conn: &State<DbConn>, request: Form<PostRequest>) -> Redirect {
    Post::create(conn, request.title.clone(), request.body.clone()).await;
    Redirect::to("/")
}

#[launch]
async fn rocket() -> Rocket<Build> {
    let pool = SqlitePool::connect("sqlite://./db.sqlite").await.unwrap();

    rocket::Rocket::build()
        .manage(pool)
        .attach(Template::fairing())
        .mount("/", routes![index, create_post])
}
