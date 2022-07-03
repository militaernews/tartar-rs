use actix_web::{Error, HttpResponse, post, Responder, web};
use actix_web::error::{BlockingError, HttpError};
use actix_web::web::Json;
use sqlx::{PgConnection, PgPool, Pool, Postgres};

use crate::db;
use crate::error::MyError;
use crate::model::{InputUser, User};

#[post("/")]
pub async fn add_user(
    user: web::Json<InputUser>, pool: web::Data<PgPool>,
) -> Result<HttpResponse, BlockingError> {
    let user_info: InputUser = user.into_inner();

    //   let pool = req.app_data::<web::Data<PgPool>>().unwrap();
    //    let conn = pool.get().expect("couldn't get db connection from pool");

 web::block(move ||
        db::add_user(pool.get_ref(), user_info))
            .await
        .map(|a| HttpResponse::Created().json(a))

   //  .map_err(|e|
     //   Err( Error::new(e.to_string())))

}
