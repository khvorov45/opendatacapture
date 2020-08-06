use crate::{auth, db};
use warp::Filter;

pub fn routes(
    db: std::sync::Arc<db::admin::AdminDB>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    authenticate_email_password(db.clone())
        .or(get_users(db))
        .or(authenticate_email_password_opts())
}

pub fn authenticate_email_password_opts(
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let mut headers = warp::http::header::HeaderMap::new();
    headers.insert(
        "Access-Control-Allow-Origin",
        warp::http::header::HeaderValue::from_static("*"),
    );
    headers.insert(
        "Access-Control-Allow-Headers",
        warp::http::header::HeaderValue::from_static("Content-Type"),
    );
    warp::options()
        .and(warp::path("authenticate"))
        .and(warp::path("email-password"))
        .map(warp::reply)
        .with(warp::reply::with::headers(headers))
}

pub fn authenticate_email_password(
    db: std::sync::Arc<db::admin::AdminDB>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let mut headers = warp::http::header::HeaderMap::new();
    headers.insert(
        "Access-Control-Allow-Origin",
        warp::http::header::HeaderValue::from_static("*"),
    );
    warp::post()
        .and(warp::path("authenticate"))
        .and(warp::path("email-password"))
        .and(warp::body::json())
        .and_then(move |cred: auth::EmailPassword| {
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
        .with(warp::reply::with::headers(headers))
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

fn access(
    admindb: std::sync::Arc<db::admin::AdminDB>,
    req_access: crate::db::admin::Access,
) -> impl Filter<Extract = ((),), Error = warp::Rejection> + Clone {
    auth_header().and_then(move |cred: auth::IdToken| {
        let admin_database = admindb.clone();
        let req_access = req_access.clone();
        async move {
            match admin_database.verify_access(&cred, req_access).await {
                Ok(_) => Ok(()),
                Err(e) => Err(warp::reject::custom(e)),
            }
        }
    })
}

pub fn get_users(
    admindb: std::sync::Arc<db::admin::AdminDB>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("users"))
        .and(access(admindb.clone(), crate::db::admin::Access::Admin))
        .and_then(move |()| {
            let admindb = admindb.clone();
            async move {
                match admindb.get_users().await {
                    Ok(users) => Ok(warp::reply::json(&users)),
                    Err(e) => Err(warp::reject::custom(e)),
                }
            }
        })
}
