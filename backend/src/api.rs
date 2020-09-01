use crate::{auth, db, db::admin::AdminDB, db::DB, error::Unauthorized, Error};
use std::convert::Infallible;
use std::sync::Arc;
use warp::{Filter, Reply};

/// All routes
pub fn routes(
    db: Arc<AdminDB>,
    prefix: &str,
) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
    let routes = health(db.clone())
        .or(generate_session_token(db.clone()))
        .or(get_user_by_token(db.clone()))
        .or(get_users(db.clone()))
        .or(create_project(db.clone()))
        .or(get_user_projects(db.clone()))
        .or(delete_project(db))
        .recover(handle_rejection)
        .boxed();
    if prefix.is_empty() {
        return routes;
    }
    warp::path(prefix.to_string()).and(routes).boxed()
}

/// All CORS headers.
/// Allowes to apply the same cors headers to every path.
/// Could not get it working when cors headers were different on every path.
pub fn get_cors() -> warp::cors::Builder {
    warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE"])
        .allow_headers(vec!["Content-Type", "Authorization"])
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
        match e {
            Error::Unauthorized(reason) => {
                status = StatusCode::UNAUTHORIZED;
                message = format!("{:?}", reason);
            }
            Error::ProjectAlreadyExists(_, _) => {
                status = StatusCode::CONFLICT;
                message = format!("{:?}", e)
            }
            _ => {
                status = StatusCode::INTERNAL_SERVER_ERROR;
                message = format!("{:?}", e);
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
/// Returns the user who the token belongs to otherwise.
fn sufficient_access(
    db: Arc<AdminDB>,
    req_access: crate::auth::Access,
) -> impl Filter<Extract = (db::admin::User,), Error = warp::Rejection> + Clone
{
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
                Err(warp::reject::custom(Error::Unauthorized(
                    Unauthorized::InsufficientAccess,
                )))
            } else {
                Ok(u)
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
    async fn get_health(
        db: Arc<AdminDB>,
    ) -> Result<impl warp::Reply, Infallible> {
        Ok(warp::reply::json(&db.health().await))
    }
    warp::path("health")
        .and(warp::get())
        .and(with_db(db))
        .and_then(get_health)
}

/// Generate session token. Returns only the string.
fn generate_session_token(
    db: Arc<AdminDB>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("auth" / "session-token")
        .and(warp::post())
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
}

/// Get user by token. If the token is wrong (not found), say unauthorized
/// (instead of not found).
fn get_user_by_token(
    db: Arc<AdminDB>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("get" / "user" / "by" / "token" / String)
        .and(warp::get())
        .and(with_db(db))
        .and_then(move |tok: String, db: Arc<AdminDB>| async move {
            match db.get_user_by_token(tok.as_str()).await {
                Ok(u) => Ok(warp::reply::json(&u)),
                Err(e) => Err(warp::reject::custom(e)),
            }
        })
}

/// Get all users
pub fn get_users(
    db: Arc<AdminDB>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("get" / "users")
        .and(warp::get())
        .and(sufficient_access(db.clone(), auth::Access::Admin))
        .and(with_db(db))
        .and_then(move |_user, db: Arc<AdminDB>| async move {
            match db.get_users().await {
                Ok(users) => Ok(warp::reply::json(&users)),
                Err(e) => Err(warp::reject::custom(e)),
            }
        })
}

/// Create a project
pub fn create_project(
    db: Arc<AdminDB>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("create" / "project" / String)
        .and(warp::put())
        .and(sufficient_access(db.clone(), auth::Access::User))
        .and(with_db(db))
        .and_then(
            move |project_name: String,
                  user: db::admin::User,
                  db: Arc<AdminDB>| async move {
                match db.create_project(user.id(), project_name.as_str()).await
                {
                    Ok(()) => Ok(warp::reply()),
                    Err(e) => Err(warp::reject::custom(e)),
                }
            },
        )
}

/// Delete a project
pub fn delete_project(
    db: Arc<AdminDB>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("delete" / "project" / String)
        .and(warp::delete())
        .and(sufficient_access(db.clone(), auth::Access::User))
        .and(with_db(db))
        .and_then(
            move |project_name: String,
                  user: db::admin::User,
                  db: Arc<AdminDB>| async move {
                match db.remove_project(user.id(), project_name.as_str()).await
                {
                    Ok(()) => Ok(warp::reply()),
                    Err(e) => Err(warp::reject::custom(e)),
                }
            },
        )
}

/// Get user's projects
pub fn get_user_projects(
    db: Arc<AdminDB>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("get" / "projects")
        .and(warp::get())
        .and(sufficient_access(db.clone(), auth::Access::User))
        .and(with_db(db))
        .and_then(move |user: db::admin::User, db: Arc<AdminDB>| async move {
            match db.get_user_projects(user.id()).await {
                Ok(projects) => Ok(warp::reply::json(&projects)),
                Err(e) => Err(warp::reject::custom(e)),
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::admin;
    use crate::tests;
    use std::sync::Arc;
    use warp::http::StatusCode;

    const TEST_DB_NAME: &str = "odcadmin_test_api";

    #[tokio::test]
    async fn test_api() {
        let _ = pretty_env_logger::try_init();

        let admindb =
            tests::create_test_admindb(TEST_DB_NAME, true, true).await;
        tests::insert_test_user(&admindb).await;

        let admindb_ref = Arc::new(admindb);
        assert_eq!(admindb_ref.get_name(), TEST_DB_NAME);

        // Individual filters given good input --------------------------------

        // Health check
        {
            let health_filter = health(admindb_ref.clone());
            let health_resp = warp::test::request()
                .method("GET")
                .path("/health")
                .reply(&health_filter)
                .await
                .into_body();
            let health = serde_json::from_slice::<bool>(&*health_resp).unwrap();
            assert!(health);
        }

        // Get session token
        {
            let session_token_filter =
                generate_session_token(admindb_ref.clone());
            let resp = warp::test::request()
                .method("POST")
                .path("/auth/session-token")
                .json(&auth::EmailPassword {
                    email: "user@example.com".to_string(),
                    password: "user".to_string(),
                })
                .reply(&session_token_filter)
                .await;
            assert_eq!(resp.status(), StatusCode::OK);
        }

        // Generate tokens to be used below
        let admin_token = admindb_ref
            .generate_session_token(auth::EmailPassword {
                email: "admin@example.com".to_string(),
                password: "admin".to_string(),
            })
            .await
            .unwrap();
        let admin_token = admin_token.token();
        let user_token = admindb_ref
            .generate_session_token(auth::EmailPassword {
                email: "user@example.com".to_string(),
                password: "user".to_string(),
            })
            .await
            .unwrap();
        let user_token = user_token.token();

        // Get user by token
        {
            let get_user_by_token_filter =
                get_user_by_token(admindb_ref.clone());
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
            assert!(argon2::verify_encoded(
                user_obtained.password_hash(),
                b"user"
            )
            .unwrap());
            assert_eq!(user_obtained.access(), auth::Access::User);
        }

        // Get users
        {
            let get_users_filter = get_users(admindb_ref.clone());
            let users_response = warp::test::request()
                .method("GET")
                .path("/get/users")
                .header("Authorization", format!("Bearer {}", admin_token))
                .reply(&get_users_filter)
                .await;
            assert_eq!(users_response.status(), StatusCode::OK);
            let users_obtained = serde_json::from_slice::<Vec<admin::User>>(
                &*users_response.body(),
            )
            .unwrap();
            assert_eq!(users_obtained.len(), 2);
        }

        // Create projects

        // Test projects
        let test_project1 = db::admin::Project::new(1, "test");

        // Make sure test projects aren't present
        log::info!("remove test projects");
        crate::tests::remove_dbs(
            admindb_ref.as_ref(),
            &[test_project1.get_dbname(TEST_DB_NAME).as_str()],
        )
        .await;

        {
            let create_project_filter = create_project(admindb_ref.clone());
            let create_project_response = warp::test::request()
                .method("PUT")
                .path("/create/project/test")
                .header("Authorization", format!("Bearer {}", admin_token))
                .reply(&create_project_filter)
                .await;
            assert_eq!(create_project_response.status(), StatusCode::OK);
            let get_projects_filter = get_user_projects(admindb_ref.clone());
            let get_projects_response = warp::test::request()
                .method("GET")
                .path("/get/projects")
                .header("Authorization", format!("Bearer {}", admin_token))
                .reply(&get_projects_filter)
                .await;
            assert_eq!(get_projects_response.status(), StatusCode::OK);
            let projects_obtained =
                serde_json::from_slice::<Vec<admin::Project>>(
                    &*get_projects_response.body(),
                )
                .unwrap();
            assert_eq!(projects_obtained.len(), 1);
        }

        // Delete projects
        {
            let delete_project_filter = delete_project(admindb_ref.clone());
            let delete_project_response = warp::test::request()
                .method("DELETE")
                .path("/delete/project/test")
                .header("Authorization", format!("Bearer {}", admin_token))
                .reply(&delete_project_filter)
                .await;
            assert_eq!(delete_project_response.status(), StatusCode::OK);
            let get_projects_filter = get_user_projects(admindb_ref.clone());
            let get_projects_response = warp::test::request()
                .method("GET")
                .path("/get/projects")
                .header("Authorization", format!("Bearer {}", admin_token))
                .reply(&get_projects_filter)
                .await;
            assert_eq!(get_projects_response.status(), StatusCode::OK);
            let projects_obtained =
                serde_json::from_slice::<Vec<admin::Project>>(
                    &*get_projects_response.body(),
                )
                .unwrap();
            assert_eq!(projects_obtained.len(), 0);
        }

        // Rejections ---------------------------------------------------------

        let routes = routes(admindb_ref.clone(), "");

        // Wrong email
        {
            let resp = warp::test::request()
                .method("POST")
                .path("/auth/session-token")
                .json(&auth::EmailPassword {
                    email: "user1@example.com".to_string(),
                    password: "user".to_string(),
                })
                .reply(&routes)
                .await;
            assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
            let body = serde_json::from_slice::<String>(&*resp.body()).unwrap();
            assert_eq!(body, "NoSuchUserEmail(\"user1@example.com\")");
        }

        // Wrong password
        {
            let resp = warp::test::request()
                .method("POST")
                .path("/auth/session-token")
                .json(&auth::EmailPassword {
                    email: "user@example.com".to_string(),
                    password: "user1".to_string(),
                })
                .reply(&routes)
                .await;
            assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
            let body = serde_json::from_slice::<String>(&*resp.body()).unwrap();
            assert_eq!(body, "WrongPassword(\"user1\")");
        }

        // Wrong token
        {
            let resp = warp::test::request()
                .method("GET")
                .path("/get/user/by/token/123")
                .reply(&routes)
                .await;
            assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
            let body = serde_json::from_slice::<String>(&*resp.body()).unwrap();
            assert_eq!(body, "NoSuchToken(\"123\")");
        }
        {
            let resp = warp::test::request()
                .method("GET")
                .path("/get/users")
                .header("Authorization", "Bearer 123")
                .reply(&routes)
                .await;
            assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
            let body = serde_json::from_slice::<String>(&*resp.body()).unwrap();
            assert_eq!(body, "NoSuchToken(\"123\")");
        }

        // Insufficient access
        {
            let resp = warp::test::request()
                .method("GET")
                .path("/get/users")
                .header("Authorization", format!("Bearer {}", user_token))
                .reply(&routes)
                .await;
            assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
            let body = serde_json::from_slice::<String>(&*resp.body()).unwrap();
            assert_eq!(body, "InsufficientAccess");
        }

        // Wrong authentication type
        {
            let resp = warp::test::request()
                .method("GET")
                .path("/get/users")
                .header("Authorization", "Basic a:a")
                .reply(&routes)
                .await;
            assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
            let body = serde_json::from_slice::<String>(&*resp.body()).unwrap();
            assert_eq!(body, "WrongAuthType(\"Basic\")");
        }

        // Missing header
        {
            let resp = warp::test::request()
                .method("GET")
                .path("/get/users")
                .reply(&routes)
                .await;
            assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
            let body = serde_json::from_slice::<String>(&*resp.body()).unwrap();
            assert_eq!(body, "Missing request header \"Authorization\"");
        }

        // Missing body
        {
            let resp = warp::test::request()
                .method("POST")
                .path("/auth/session-token")
                .reply(&routes)
                .await;
            assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
            let body = serde_json::from_slice::<String>(&*resp.body()).unwrap();
            assert_eq!(
                body,
                "Request body deserialize error: \
                EOF while parsing a value at line 1 column 0"
            );
        }

        // Wrong method
        {
            let resp = warp::test::request()
                .method("GET")
                .path("/auth/session-token")
                .reply(&routes)
                .await;
            assert_eq!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
            let body = serde_json::from_slice::<String>(&*resp.body()).unwrap();
            assert_eq!(body, "HTTP method not allowed");
        }

        // Creating the same project twice
        {
            warp::test::request()
                .method("PUT")
                .path("/create/project/test")
                .header("Authorization", format!("Bearer {}", admin_token))
                .reply(&routes)
                .await;
            let resp = warp::test::request()
                .method("PUT")
                .path("/create/project/test")
                .header("Authorization", format!("Bearer {}", admin_token))
                .reply(&routes)
                .await;
            assert_eq!(resp.status(), StatusCode::CONFLICT);
            assert_eq!(
                serde_json::from_slice::<String>(&*resp.body()).unwrap(),
                "ProjectAlreadyExists(1, \"test\")"
            );
        }

        // Delete a non-existent project
        {
            let resp = warp::test::request()
                .method("DELETE")
                .path("/delete/project/test_nonexistent")
                .header("Authorization", format!("Bearer {}", admin_token))
                .reply(&routes)
                .await;
            assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
            assert_eq!(
                serde_json::from_slice::<String>(&*resp.body()).unwrap(),
                "NoSuchProject(1, \"test_nonexistent\")"
            );
        }

        // Token too old
        {
            let old_token = admindb_ref
                .generate_session_token(auth::EmailPassword {
                    email: "admin@example.com".to_string(),
                    password: "admin".to_string(),
                })
                .await
                .unwrap();
            let old_token = old_token.token();
            admindb_ref
                .execute(
                    format!(
                        "UPDATE \"token\" \
                        SET \"created\" = '2000-08-14 08:15:29.425665+10' \
                        WHERE \"token\" = '{}'",
                        old_token
                    )
                    .as_str(),
                )
                .await
                .unwrap();
            let resp = warp::test::request()
                .method("GET")
                .path("/get/users")
                .header("Authorization", format!("Bearer {}", old_token))
                .reply(&routes)
                .await;
            assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
            let body = serde_json::from_slice::<String>(&*resp.body()).unwrap();
            assert_eq!(body, "TokenTooOld");
        }

        // Not found
        {
            let resp = warp::test::request()
                .method("GET")
                .path("/")
                .reply(&routes)
                .await;
            assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        }

        // CORS ---------------------------------------------------------------

        // When not attached
        {
            let resp = warp::test::request()
                .method("GET")
                .path("/health")
                .header("Origin", "test")
                .reply(&routes)
                .await;
            assert_eq!(resp.status(), StatusCode::OK);
            let body = serde_json::from_slice::<bool>(&*resp.body()).unwrap();
            assert!(body);
            let heads = resp.headers();
            assert!(heads.get("access-control-allow-origin").is_none());
        }

        let routes_cors = routes.clone().with(get_cors());

        // When request is good
        {
            let resp = warp::test::request()
                .method("GET")
                .path("/health")
                .header("Origin", "test")
                .reply(&routes_cors)
                .await;
            assert_eq!(resp.status(), StatusCode::OK);
            let body = serde_json::from_slice::<bool>(&*resp.body()).unwrap();
            assert!(body);
            let heads = resp.headers();
            let allow_origin =
                heads.get("access-control-allow-origin").unwrap();
            assert_eq!(allow_origin, "test");
        }

        // When request fails
        {
            let resp = warp::test::request()
                .method("POST")
                .path("/health")
                .header("Origin", "test")
                .reply(&routes_cors)
                .await;
            assert_eq!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
            let heads = resp.headers();
            let allow_origin =
                heads.get("access-control-allow-origin").unwrap();
            assert_eq!(allow_origin, "test");
        }

        // Options request
        {
            let resp = warp::test::request()
                .method("OPTIONS")
                .path("/health")
                .header("Origin", "test")
                .header("Access-Control-Request-Method", "GET")
                .reply(&routes_cors)
                .await;
            assert_eq!(resp.status(), StatusCode::OK);
            let heads = resp.headers();
            let allow_origin =
                heads.get("access-control-allow-origin").unwrap();
            assert_eq!(allow_origin, "test");
        }

        // Disallowed header
        {
            let resp = warp::test::request()
                .method("OPTIONS")
                .path("/health")
                .header("Origin", "test")
                .header("Access-Control-Request-Method", "GET")
                .header("Access-Control-Request-Headers", "X-Username")
                .reply(&routes_cors)
                .await;
            assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        }
    }
}
