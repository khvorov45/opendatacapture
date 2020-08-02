use crate::db;
use warp::Filter;

pub fn authenticate_email_password(
    db: std::sync::Arc<db::admin::AdminDB>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path("authenticate"))
        .and(warp::path("email-password"))
        .and(warp::body::json())
        .and_then(move |cred: db::admin::EmailPassword| {
            let admin_database = db.clone();
            async move {
                let auth =
                    admin_database.authenticate_email_password(cred).await;
                match auth {
                    Ok(res) => Ok(warp::reply::json(&res)),
                    Err(crate::Error::NoSuchUser(_)) => Err(warp::reject()),
                    Err(e) => Err(warp::reject::custom(e)),
                }
            }
        })
}
