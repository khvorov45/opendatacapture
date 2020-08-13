use crate::{auth, db, db::admin::AdminDB, db::DB, Error};
use std::convert::Infallible;
use std::sync::Arc;
use warp::{http::Method, Filter, Reply};

/// All routes
pub fn routes(
    db: Arc<AdminDB>,
) -> impl Filter<Extract = impl Reply, Error = Infallible> + Clone {
    health(db.clone())
        .or(generate_session_token(db.clone()))
        .or(get_user_by_token(db.clone()))
        .or(get_users(db))
        .recover(handle_rejection)
}

/// Error handling
async fn handle_rejection(
    err: warp::Rejection,
) -> Result<impl warp::Reply, Infallible> {
    use warp::http::StatusCode;
    let status;
    let message;
    // My errors
    if let Some(e) = err.find::<Error>() {
        use Error::*;
        match e {
            NoSuchUser(_) | WrongPassword(_) | NoSuchToken(_) => {
                status = StatusCode::UNAUTHORIZED;
                message = format!("{:?}", e);
            }
            _ => {
                status = StatusCode::INTERNAL_SERVER_ERROR;
                message = e.to_string();
            }
        }
    // Not my errors
    } else if err.is_not_found() {
        status = StatusCode::NOT_FOUND;
        message = "NOT_FOUND".to_string();
    } else if err.find::<warp::filters::cors::CorsForbidden>().is_some() {
        status = StatusCode::FORBIDDEN;
        message = "CORS_FORBIDDEN".to_string();
    } else if err
        .find::<warp::filters::body::BodyDeserializeError>()
        .is_some()
    {
        status = StatusCode::BAD_REQUEST;
        message = "BAD_REQUEST".to_string();
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        status = StatusCode::METHOD_NOT_ALLOWED;
        message = "METHOD_NOT_ALLOWED".to_string();
    } else {
        log::error!("Unhandled rejection: {:?}", err);
        status = StatusCode::INTERNAL_SERVER_ERROR;
        message = "UNHANDLED_REJECTION".to_string();
    }
    let json = warp::reply::json(&message);
    Ok(warp::reply::with_status(json, status))
}

/// Rejects if the access (as per the Authorization header) is not high enough
fn sufficient_access(
    db: Arc<AdminDB>,
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

/// Extracts the database reference
fn with_db(
    db: Arc<AdminDB>,
) -> impl Filter<Extract = (Arc<AdminDB>,), Error = Infallible> + Clone {
    warp::any().map(move || db.clone())
}

// Routes ---------------------------------------------------------------------

/// Health check
fn health(
    db: Arc<AdminDB>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let cors = warp::cors()
        .allow_methods(&[Method::GET])
        .allow_any_origin();
    async fn get_health(
        db: Arc<AdminDB>,
    ) -> Result<impl warp::Reply, Infallible> {
        Ok(warp::reply::json(&db.health().await))
    }
    warp::get()
        .and(warp::path("health"))
        .and(with_db(db))
        .and_then(get_health)
        .with(cors)
}

/// Generate session token. Returns only the string.
fn generate_session_token(
    db: Arc<AdminDB>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let cors = warp::cors()
        .allow_methods(&[Method::POST])
        .allow_any_origin()
        .allow_header("Content-Type");
    warp::post()
        .and(warp::path!("auth" / "session-token"))
        .and(warp::body::json())
        .and_then(move |cred: auth::EmailPassword| {
            let db = db.clone();
            async move {
                match db.generate_session_token(cred).await {
                    Ok(t) => Ok(warp::reply::json(&t.token().to_string())),
                    Err(e) => Err(warp::reject::custom(e)),
                }
            }
        })
        .with(cors)
}

/// Get user by token
fn get_user_by_token(
    db: Arc<AdminDB>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let cors = warp::cors()
        .allow_methods(&[Method::GET])
        .allow_any_origin();
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
        .with(cors)
}

/// Get all users
pub fn get_users(
    admindb: Arc<AdminDB>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let cors = warp::cors()
        .allow_methods(&[Method::GET])
        .allow_any_origin();
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
        .with(cors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::admin;
    use crate::tests;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_api() {
        let _ = pretty_env_logger::try_init();
        let admindb = tests::create_test_admindb("odcadmin_test_api").await;
        let admindb_ref = Arc::new(admindb);
        let session_token_filter = generate_session_token(admindb_ref.clone());

        // Individual filters given good input --------------------------------

        // Get session token
        let user_token_resp = warp::test::request()
            .method("POST")
            .path("/auth/session-token")
            .json(&auth::EmailPassword {
                email: "user@example.com".to_string(),
                password: "user".to_string(),
            })
            .reply(&session_token_filter)
            .await
            .into_body();
        let user_token =
            serde_json::from_slice::<String>(&*user_token_resp).unwrap();
        let admin_token_resp = warp::test::request()
            .method("POST")
            .path("/auth/session-token")
            .json(&auth::EmailPassword {
                email: "admin@example.com".to_string(),
                password: "admin".to_string(),
            })
            .reply(&session_token_filter)
            .await
            .into_body();
        let admin_token =
            serde_json::from_slice::<String>(&*admin_token_resp).unwrap();

        // Get user by token
        let get_user_by_token_filter = get_user_by_token(admindb_ref.clone());
        let user_response = warp::test::request()
            .method("GET")
            .path(format!("/get/user/by/token/{}", user_token).as_str())
            .header("Authorization", format!("Bearer {}", admin_token))
            .reply(&get_user_by_token_filter)
            .await
            .into_body();
        let user_obtained =
            serde_json::from_slice::<admin::User>(&*user_response)
                .unwrap_or_else(|e| {
                    panic!(
                        "Could not deserialize into user: {:?} because {}",
                        std::str::from_utf8(&*user_response),
                        e
                    )
                });
        assert_eq!(user_obtained.email(), "user@example.com");
        assert!(
            argon2::verify_encoded(user_obtained.password_hash(), b"user")
                .unwrap_or_else(|e| panic!(
                    "could not verify hash: {} because {}",
                    user_obtained.password_hash(),
                    e
                ))
        );
        assert_eq!(user_obtained.access(), auth::Access::User);

        // Get users
        let get_users_filter = get_users(admindb_ref.clone());
        let users_response = warp::test::request()
            .method("GET")
            .path("/get/users")
            .header("Authorization", format!("Bearer {}", admin_token))
            .reply(&get_users_filter)
            .await
            .into_body();
        let users_obtained =
            serde_json::from_slice::<Vec<admin::User>>(&*users_response)
                .unwrap_or_else(|e| {
                    panic!(
                        "Could not deserialize into users: {:?} because {}",
                        std::str::from_utf8(&*users_response),
                        e
                    )
                });
        assert_eq!(users_obtained.len(), 2);

        // Rejections ---------------------------------------------------------

        let routes = routes(admindb_ref.clone());

        // Wrong email
        let wrong_email_resp = warp::test::request()
            .method("POST")
            .path("/auth/session-token")
            .json(&auth::EmailPassword {
                email: "user1@example.com".to_string(),
                password: "user".to_string(),
            })
            .reply(&routes)
            .await
            .into_body();
        let wrong_email =
            serde_json::from_slice::<String>(&*wrong_email_resp).unwrap();
        assert_eq!(wrong_email, "NoSuchUser(\"user1@example.com\")");

        // Wrong password
        let wrong_password_resp = warp::test::request()
            .method("POST")
            .path("/auth/session-token")
            .json(&auth::EmailPassword {
                email: "user@example.com".to_string(),
                password: "user1".to_string(),
            })
            .reply(&routes)
            .await
            .into_body();
        let wrong_email =
            serde_json::from_slice::<String>(&*wrong_password_resp).unwrap();
        assert_eq!(wrong_email, "WrongPassword(\"user1\")");
    }
}
