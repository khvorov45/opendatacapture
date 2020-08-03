use crate::{auth, db};
use warp::Filter;

pub fn routes(
    db: std::sync::Arc<db::admin::AdminDB>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let public = authenticate_email_password(db.clone());
    let protected = get_users(db);
    public.or(protected)
}

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

fn auth_header(
) -> impl Filter<Extract = (auth::IdToken,), Error = warp::Rejection> + Clone {
    warp::header::<String>("Authorization").and_then(
        move |auth_header: String| {
            log::info!("{}", auth_header);
            async move {
                match auth::parse_basic_header(auth_header.as_str()) {
                    Ok(cred) => Ok(cred),
                    Err(e) => {
                        log::error!("{}", e);
                        Err(warp::reject::custom(e))
                    }
                }
            }
        },
    )
}

pub fn get_users(
    _db: std::sync::Arc<db::admin::AdminDB>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("users"))
        .and(auth_header())
        .map(|cred| warp::reply::json(&cred))
}
