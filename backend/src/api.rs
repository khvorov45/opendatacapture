use crate::{auth, db, Error};
use warp::Filter;

/// All routes
pub fn routes(
    db: std::sync::Arc<db::admin::AdminDB>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let opts = warp::options().map(warp::reply);
    generate_session_token(db.clone())
        .or(get_user_by_token(db.clone()))
        .or(get_users(db))
        .or(opts)
        .with(access_headers())
}

/// CORS headers
fn access_headers() -> warp::filters::reply::WithHeaders {
    let mut headers = warp::http::header::HeaderMap::new();
    headers.insert(
        "Access-Control-Allow-Origin",
        warp::http::header::HeaderValue::from_static("*"),
    );
    headers.insert(
        "Access-Control-Allow-Headers",
        warp::http::header::HeaderValue::from_static("Content-Type"),
    );
    warp::reply::with::headers(headers)
}

/// Generate session token. Returns only the string.
fn generate_session_token(
    db: std::sync::Arc<db::admin::AdminDB>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path!("auth" / "session-token"))
        .and(warp::body::json())
        .and_then(move |cred: auth::EmailPassword| {
            let db = db.clone();
            async move {
                match db.generate_session_token(cred).await {
                    Ok(t) => Ok(warp::reply::json(t.token())),
                    Err(e) => Err(warp::reject::custom(e)),
                }
            }
        })
}

/// Get user by token
fn get_user_by_token(
    db: std::sync::Arc<db::admin::AdminDB>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path!("get" / "user" / "by" / "token" / String))
        .and_then(move |tok: String| {
            let db = db.clone();
            async move {
                match db.get_user_by_token(tok.as_str()).await {
                    Ok(u) => Ok(warp::reply::json(&u)),
                    Err(e) => Err(warp::reject::custom(e)),
                }
            }
        })
}

/// Rejects if the access (as per the Authorization header) is not high enough
fn sufficient_access(
    db: std::sync::Arc<db::admin::AdminDB>,
    req_access: crate::auth::Access,
) -> impl Filter<Extract = ((),), Error = warp::Rejection> + Clone {
    warp::header::<String>("Authorization")
        .and_then(move |tok_raw: String| async move {
            match auth::parse_bearer_header(tok_raw.as_str()) {
                Ok(t) => Ok(t.to_string()),
                Err(e) => Err(warp::reject::custom(e)),
            }
        })
        .and_then(move |tok: String| {
            let db = db.clone();
            async move {
                match db.get_user_by_token(tok.as_str()).await {
                    Ok(u) => Ok(u),
                    Err(e) => Err(warp::reject::custom(e)),
                }
            }
        })
        .and_then(move |u: db::admin::User| async move {
            if u.access() < req_access {
                Err(warp::reject::custom(Error::InsufficientAccess))
            } else {
                Ok(())
            }
        })
}

/// Get all users
pub fn get_users(
    admindb: std::sync::Arc<db::admin::AdminDB>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path!("get" / "users"))
        .and(sufficient_access(admindb.clone(), auth::Access::Admin))
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
