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
    log::debug!("recover filter error: {:?}", err);
    // My errors
    if let Some(e) = err.find::<Error>() {
        use Error::*;
        match e {
            NoSuchUser(_) | WrongPassword(_) | NoSuchToken(_)
            | InsufficientAccess | TokenTooOld => {
                status = StatusCode::UNAUTHORIZED;
                message = format!("{:?}", e);
            }
            _ => {
                status = StatusCode::INTERNAL_SERVER_ERROR;
                message = e.to_string();
            }
        }
    // Not my errors
    } else if let Some(e) = err.find::<warp::reject::MissingHeader>() {
        if e.name() == "Authorization" {
            status = StatusCode::UNAUTHORIZED;
        } else {
            status = StatusCode::BAD_REQUEST;
        }
        message = e.to_string();
    } else if let Some(e) = err.find::<warp::filters::cors::CorsForbidden>() {
        status = StatusCode::FORBIDDEN;
        message = e.to_string();
    } else if let Some(e) =
        err.find::<warp::filters::body::BodyDeserializeError>()
    {
        status = StatusCode::BAD_REQUEST;
        message = e.to_string();
    } else if let Some(e) = err.find::<warp::reject::MethodNotAllowed>() {
        status = StatusCode::METHOD_NOT_ALLOWED;
        message = e.to_string();
    } else if err.is_not_found() {
        status = StatusCode::NOT_FOUND;
        message = "NOT_FOUND".to_string();
    } else {
        status = StatusCode::INTERNAL_SERVER_ERROR;
        message = format!("UNHANDLED_REJECTION: {:?}", err);
        log::error!("{}", message);
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
        .and(with_db(db))
        .and_then(
            move |cred: auth::EmailPassword, db: Arc<AdminDB>| async move {
                match db.generate_session_token(cred).await {
                    Ok(t) => Ok(warp::reply::json(&t.token().to_string())),
                    Err(e) => Err(warp::reject::custom(e)),
                }
            },
        )
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
        .and(with_db(db))
        .and_then(move |tok: String, db: Arc<AdminDB>| async move {
            match db.get_user_by_token(tok.as_str()).await {
                Ok(u) => Ok(warp::reply::json(&u)),
                Err(e) => Err(warp::reject::custom(e)),
            }
        })
        .with(cors)
}

/// Get all users
pub fn get_users(
    db: Arc<AdminDB>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let cors = warp::cors()
        .allow_methods(&[Method::GET])
        .allow_any_origin();
    warp::get()
        .and(warp::path!("get" / "users"))
        .and(sufficient_access(db.clone(), auth::Access::Admin))
        .and(with_db(db))
        .and_then(move |(), db: Arc<AdminDB>| async move {
            match db.get_users().await {
                Ok(users) => Ok(warp::reply::json(&users)),
                Err(e) => Err(warp::reject::custom(e)),
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
    use warp::http::StatusCode;

    #[tokio::test]
    async fn test_api() {
        let _ = pretty_env_logger::try_init();

        let admindb =
            tests::create_test_admindb("odcadmin_test_api", true).await;
        tests::insert_test_user(&admindb).await;

        let admindb_ref = Arc::new(admindb);

        // Individual filters given good input --------------------------------

        // Health check
        let health_filter = health(admindb_ref.clone());
        let health_resp = warp::test::request()
            .method("GET")
            .path("/health")
            .reply(&health_filter)
            .await
            .into_body();
        let health = serde_json::from_slice::<bool>(&*health_resp).unwrap();
        assert!(health);

        // Get session token
        let session_token_filter = generate_session_token(admindb_ref.clone());
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
            .await;
        assert_eq!(admin_token_resp.status(), StatusCode::OK);
        let admin_token =
            serde_json::from_slice::<String>(&*admin_token_resp.body())
                .unwrap();

        // Get user by token
        let get_user_by_token_filter = get_user_by_token(admindb_ref.clone());
        let user_response = warp::test::request()
            .method("GET")
            .path(format!("/get/user/by/token/{}", user_token).as_str())
            .header("Authorization", format!("Bearer {}", admin_token))
            .reply(&get_user_by_token_filter)
            .await;
        assert_eq!(user_response.status(), StatusCode::OK);
        let user_obtained =
            serde_json::from_slice::<admin::User>(&*user_response.body())
                .unwrap();
        assert_eq!(user_obtained.email(), "user@example.com");
        assert!(
            argon2::verify_encoded(user_obtained.password_hash(), b"user")
                .unwrap()
        );
        assert_eq!(user_obtained.access(), auth::Access::User);

        // Get users
        let get_users_filter = get_users(admindb_ref.clone());
        let users_response = warp::test::request()
            .method("GET")
            .path("/get/users")
            .header("Authorization", format!("Bearer {}", admin_token))
            .reply(&get_users_filter)
            .await;
        assert_eq!(users_response.status(), StatusCode::OK);
        let users_obtained =
            serde_json::from_slice::<Vec<admin::User>>(&*users_response.body())
                .unwrap();
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
            .await;
        assert_eq!(wrong_email_resp.status(), StatusCode::UNAUTHORIZED);
        let wrong_email =
            serde_json::from_slice::<String>(&*wrong_email_resp.body())
                .unwrap();
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
            .await;
        assert_eq!(wrong_password_resp.status(), StatusCode::UNAUTHORIZED);
        let wrong_email =
            serde_json::from_slice::<String>(&*wrong_password_resp.body())
                .unwrap();
        assert_eq!(wrong_email, "WrongPassword(\"user1\")");

        // Wrong token
        let wrong_token_resp = warp::test::request()
            .method("GET")
            .path("/get/user/by/token/123")
            .reply(&routes)
            .await;
        assert_eq!(wrong_token_resp.status(), StatusCode::UNAUTHORIZED);
        let wrong_token =
            serde_json::from_slice::<String>(&*wrong_token_resp.body())
                .unwrap();
        assert_eq!(wrong_token, "NoSuchToken(\"123\")");
        let wrong_token_resp = warp::test::request()
            .method("GET")
            .path("/get/users")
            .header("Authorization", "Bearer 123")
            .reply(&routes)
            .await;
        assert_eq!(wrong_token_resp.status(), StatusCode::UNAUTHORIZED);
        let wrong_token =
            serde_json::from_slice::<String>(&*wrong_token_resp.body())
                .unwrap();
        assert_eq!(wrong_token, "NoSuchToken(\"123\")");

        // Token too old
        admindb_ref
            .get_con()
            .await
            .unwrap()
            .execute(
                "UPDATE \"token\" \
                SET \"created\" = '2000-08-14 08:15:29.425665+10' \
                WHERE \"user\" = '1'",
                &[],
            )
            .await
            .unwrap();
        let old_token_resp = warp::test::request()
            .method("GET")
            .path("/get/users")
            .header("Authorization", format!("Bearer {}", admin_token))
            .reply(&routes)
            .await;
        assert_eq!(old_token_resp.status(), StatusCode::UNAUTHORIZED);
        let old_token =
            serde_json::from_slice::<String>(&*old_token_resp.body()).unwrap();
        assert_eq!(old_token, "TokenTooOld");

        // Insufficient access
        let ins_access_resp = warp::test::request()
            .method("GET")
            .path("/get/users")
            .header("Authorization", format!("Bearer {}", user_token))
            .reply(&routes)
            .await;
        assert_eq!(ins_access_resp.status(), StatusCode::UNAUTHORIZED);
        let ins_access =
            serde_json::from_slice::<String>(&*ins_access_resp.body()).unwrap();
        assert_eq!(ins_access, "InsufficientAccess");

        // Missing header
        let miss_head_resp = warp::test::request()
            .method("GET")
            .path("/get/users")
            .reply(&routes)
            .await;
        assert_eq!(miss_head_resp.status(), StatusCode::UNAUTHORIZED);
        let miss_head =
            serde_json::from_slice::<String>(&*miss_head_resp.body()).unwrap();
        assert_eq!(miss_head, "Missing request header \"Authorization\"");

        // Missing body
        let miss_body_resp = warp::test::request()
            .method("POST")
            .path("/auth/session-token")
            .reply(&routes)
            .await;
        assert_eq!(miss_body_resp.status(), StatusCode::BAD_REQUEST);
        let miss_body =
            serde_json::from_slice::<String>(&*miss_body_resp.body()).unwrap();
        assert_eq!(
            miss_body,
            "Request body deserialize error: \
            EOF while parsing a value at line 1 column 0"
        );

        // Wrong method
        let wrong_method_resp = warp::test::request()
            .method("GET")
            .path("/auth/session-token")
            .reply(&routes)
            .await;
        assert_eq!(wrong_method_resp.status(), StatusCode::METHOD_NOT_ALLOWED);
        let wrong_method =
            serde_json::from_slice::<String>(&*wrong_method_resp.body())
                .unwrap();
        assert_eq!(wrong_method, "HTTP method not allowed");
    }
}
