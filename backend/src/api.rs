use crate::{auth, db, Error};
use warp::Filter;

pub fn routes(
    db: std::sync::Arc<db::admin::AdminDB>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    authenticate_email_password::route(db.clone()).or(get_users(db))
}

mod authenticate_email_password {
    use super::token;
    use crate::db;
    use warp::Filter;

    pub fn route(
        db: std::sync::Arc<db::admin::AdminDB>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        opts().or(post(db))
    }

    fn opts(
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
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

    fn post(
        db: std::sync::Arc<db::admin::AdminDB>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        let mut headers = warp::http::header::HeaderMap::new();
        headers.insert(
            "Access-Control-Allow-Origin",
            warp::http::header::HeaderValue::from_static("*"),
        );
        warp::post()
            .and(warp::path("authenticate"))
            .and(warp::path("email-password"))
            .and(token(db))
            .map(|t| warp::reply::json(&t))
            .with(warp::reply::with::headers(headers))
    }
}

fn token(
    admindb: std::sync::Arc<db::admin::AdminDB>,
) -> impl Filter<Extract = (auth::Token,), Error = warp::Rejection> + Clone {
    use crate::error::Unauthorized;
    warp::body::json().and_then(move |cred: auth::EmailPassword| {
        let admin_database = admindb.clone();
        async move {
            let auth = admin_database.verify_email_password(cred).await;
            match auth {
                Ok(res) => match res {
                    auth::PasswordOutcome::Ok(t) => Ok(t),
                    auth::PasswordOutcome::WrongPassword => {
                        Err(warp::reject::custom(Error::Unauthorized(
                            Unauthorized::WrongPassword,
                        )))
                    }
                    auth::PasswordOutcome::EmailNotFound => {
                        Err(warp::reject::custom(Error::Unauthorized(
                            Unauthorized::EmailNotFound,
                        )))
                    }
                },
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

fn access(
    admindb: std::sync::Arc<db::admin::AdminDB>,
) -> impl Filter<Extract = (auth::Access,), Error = warp::Rejection> + Clone {
    use crate::error::Unauthorized;
    auth_header().and_then(move |cred: auth::IdToken| {
        let admin_database = admindb.clone();
        async move {
            match admin_database.verify_id_token(&cred).await {
                Ok(out) => match out {
                    auth::TokenOutcome::Ok(a) => Ok(a),
                    auth::TokenOutcome::TokenTooOld => {
                        Err(warp::reject::custom(Error::Unauthorized(
                            Unauthorized::TokenTooOld,
                        )))
                    }
                    auth::TokenOutcome::TokenNotFound => {
                        Err(warp::reject::custom(Error::Unauthorized(
                            Unauthorized::TokenNotFound,
                        )))
                    }
                },
                Err(e) => Err(warp::reject::custom(e)),
            }
        }
    })
}

fn sufficient_access(
    admindb: std::sync::Arc<db::admin::AdminDB>,
    req_access: crate::auth::Access,
) -> impl Filter<Extract = ((),), Error = warp::Rejection> + Clone {
    access(admindb).and_then(move |a: auth::Access| {
        let req_access = req_access.clone();
        async move {
            if a < req_access {
                Err(warp::reject::custom(Error::Unauthorized(
                    crate::error::Unauthorized::InsufficientAccess,
                )))
            } else {
                Ok(())
            }
        }
    })
}

pub fn get_users(
    admindb: std::sync::Arc<db::admin::AdminDB>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("users"))
        .and(sufficient_access(
            admindb.clone(),
            crate::auth::Access::Admin,
        ))
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
