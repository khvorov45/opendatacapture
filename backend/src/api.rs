use crate::{auth, db, error::Unauthorized, Error};
use db::admin::{AdminDB, Project, User};
use db::user::table::RowJson;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{http::StatusCode, Filter, Reply};

type DBRef = Arc<Mutex<AdminDB>>;

/// CORS routes
pub fn routes_cors(
    db: DBRef,
    prefix: &str,
) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
    // Note here that if cors come before the recover filter then rejection
    // replies don't have cors on them. Having cors be after the recover
    // filter makes it so their rejections never enter that filter.
    // It's possible to have the recover filter on both sides but I'll
    // just let cors rejections handle themselves
    routes(db, prefix).with(get_cors())
}

/// Standard routes
pub fn routes(
    db: DBRef,
    prefix: &str,
) -> impl Filter<Extract = impl Reply, Error = Infallible> + Clone {
    base_routes(db, prefix).recover(handle_rejection)
}

/// All routes, no recovery
fn base_routes(
    db: DBRef,
    prefix: &str,
) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
    let routes = health(db.clone())
        .or(generate_session_token(db.clone()))
        .or(refresh_token(db.clone()))
        .or(remove_token(db.clone()))
        .or(get_user_by_token(db.clone()))
        .or(get_users(db.clone()))
        .or(create_user(db.clone()))
        .or(remove_user(db.clone()))
        .or(create_project(db.clone()))
        .or(get_user_project(db.clone()))
        .or(get_user_projects(db.clone()))
        .or(delete_project(db.clone()))
        .or(create_table(db.clone()))
        .or(remove_table(db.clone()))
        .or(get_table_names(db.clone()))
        .or(get_all_meta(db.clone()))
        .or(get_table_meta(db.clone()))
        .or(get_table_data(db.clone()))
        .or(insert_data(db.clone()))
        .or(remove_all_user_table_data(db))
        .boxed();
    if prefix.is_empty() {
        return routes;
    }
    warp::path(prefix.to_string()).and(routes).boxed()
}

/// All CORS headers.
/// Allowes to apply the same cors headers to every path.
/// Could not get it working when cors headers were different on every path.
fn get_cors() -> warp::cors::Builder {
    warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE"])
        .allow_headers(vec!["Content-Type", "Authorization"])
}

/// Error handling
async fn handle_rejection(
    err: warp::Rejection,
) -> Result<impl warp::Reply, Infallible> {
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
            Error::ProjectAlreadyExists(_, _)
            | Error::TableAlreadyExists(_)
            | Error::NoSuchColumns(_) => {
                status = StatusCode::CONFLICT;
                message = format!("{:?}", e)
            }
            Error::NoSuchProject(_, _) | Error::NoSuchTable(_) => {
                status = StatusCode::NOT_FOUND;
                message = format!("{:?}", e);
            }
            // All my errors that could happen through requests should be
            // handled above. If they aren't then log them here and implement
            // a handler above later.
            _ => {
                status = StatusCode::INTERNAL_SERVER_ERROR;
                message = format!("{:?}", e);
                log::error!("{}", message);
            }
        }
    // Not my errors
    } else if let Some(e) = err.find::<warp::reject::MissingHeader>() {
        if e.name() == "Authorization" {
            status = StatusCode::UNAUTHORIZED;
        // I don't require any other headers at the moment so the below
        // case shouldn't happen
        } else {
            status = StatusCode::BAD_REQUEST;
        }
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
    // Again, all errors that can happen through requests should be handled
    // above. If they aren't then log and implement a handler.
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
    db: DBRef,
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
                match db.lock().await.get_user_by_token(tok.as_str()).await {
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

/// Extracts a project given its id. Rejects if project is not found.
async fn extract_project(
    project_name: String,
    user: db::admin::User,
    db: DBRef,
) -> std::result::Result<db::admin::Project, warp::Rejection> {
    match db
        .lock()
        .await
        .get_user_project(user.id(), project_name.as_str())
        .await
    {
        Ok(p) => Ok(p),
        Err(e) => Err(warp::reject::custom(e)),
    }
}

/// Extracts project name and table name
async fn extract_project_and_table(
    project_name: String,
    table_name: String,
    user: User,
    db: DBRef,
) -> std::result::Result<(Project, String), warp::Rejection> {
    Ok((extract_project(project_name, user, db).await?, table_name))
}

/// Extracts the database reference
fn with_db(
    db: DBRef,
) -> impl Filter<Extract = (DBRef,), Error = Infallible> + Clone {
    warp::any().map(move || db.clone())
}

/// Reply with the no content status
fn reply_no_content() -> impl warp::Reply {
    warp::reply::with_status(warp::reply(), StatusCode::NO_CONTENT)
}

// Routes ---------------------------------------------------------------------

/// Health check
fn health(
    db: DBRef,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    async fn get_health(db: DBRef) -> Result<impl warp::Reply, Infallible> {
        Ok(warp::reply::json(&db.lock().await.health().await))
    }
    warp::path("health")
        .and(warp::get())
        .and(with_db(db))
        .and_then(get_health)
}

/// Generate session token. Returns only the string.
fn generate_session_token(
    db: DBRef,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("auth" / "session-token")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_db(db))
        .and_then(move |cred: auth::EmailPassword, db: DBRef| async move {
            match db.lock().await.generate_session_token(cred).await {
                Ok(t) => Ok(warp::reply::json(&t)),
                Err(e) => Err(warp::reject::custom(e)),
            }
        })
}

/// Refresh a token, i.e. generate new given old
fn refresh_token(
    db: DBRef,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("auth" / "refresh-token" / String)
        .and(warp::post())
        .and(with_db(db))
        .and_then(move |old_token: String, db: DBRef| async move {
            match db.lock().await.refresh_token(old_token.as_str()).await {
                Ok(t) => Ok(warp::reply::json(&t)),
                Err(e) => Err(warp::reject::custom(e)),
            }
        })
}

/// Removes the given token regardless of validity
fn remove_token(
    db: DBRef,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("auth" / "remove-token" / String)
        .and(warp::delete())
        .and(with_db(db))
        .and_then(move |token: String, db: DBRef| async move {
            match db.lock().await.remove_token(token.as_str()).await {
                Ok(()) => Ok(reply_no_content()),
                Err(e) => Err(warp::reject::custom(e)),
            }
        })
}

/// Get user by token. If the token is wrong (not found), say unauthorized
/// (instead of not found).
fn get_user_by_token(
    db: DBRef,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("get" / "user" / "by" / "token" / String)
        .and(warp::get())
        .and(with_db(db))
        .and_then(move |tok: String, db: DBRef| async move {
            match db.lock().await.get_user_by_token(tok.as_str()).await {
                Ok(u) => Ok(warp::reply::json(&u)),
                Err(e) => Err(warp::reject::custom(e)),
            }
        })
}

/// Get all users
fn get_users(
    db: DBRef,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("get" / "users")
        .and(warp::get())
        .and(sufficient_access(db.clone(), auth::Access::Admin))
        .and(with_db(db))
        .and_then(move |_user, db: DBRef| async move {
            match db.lock().await.get_users().await {
                Ok(users) => Ok(warp::reply::json(&users)),
                Err(e) => Err(warp::reject::custom(e)),
            }
        })
}

/// Create a new user
fn create_user(
    db: DBRef,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("create" / "user")
        .and(warp::put())
        .and(warp::body::json())
        .and(with_db(db))
        .and_then(move |u: auth::EmailPassword, db: DBRef| async move {
            match db
                .lock()
                .await
                .insert_user(
                    u.email.as_str(),
                    u.password.as_str(),
                    auth::Access::User,
                )
                .await
            {
                Ok(()) => Ok(reply_no_content()),
                Err(e) => Err(warp::reject::custom(e)),
            }
        })
}

/// Remove user by email. Require admin authorization
fn remove_user(
    db: DBRef,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("remove" / "user" / String)
        .and(warp::delete())
        .and(with_db(db.clone()))
        .and(sufficient_access(db, auth::Access::Admin))
        .and_then(move |email: String, db: DBRef, _| async move {
            match db.lock().await.remove_user(email.as_str()).await {
                Ok(()) => Ok(reply_no_content()),
                Err(e) => Err(warp::reject::custom(e)),
            }
        })
}

/// Create a project
fn create_project(
    db: DBRef,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("create" / "project" / String)
        .and(warp::put())
        .and(sufficient_access(db.clone(), auth::Access::User))
        .and(with_db(db))
        .and_then(
            move |project_name: String,
                  user: db::admin::User,
                  db: DBRef| async move {
                let db = db.lock().await;
                match db.create_project(user.id(), project_name.as_str()).await
                {
                    Ok(()) => Ok(reply_no_content()),
                    Err(e) => Err(warp::reject::custom(e)),
                }
            },
        )
}

/// Delete a project
fn delete_project(
    db: DBRef,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("delete" / "project" / String)
        .and(warp::delete())
        .and(sufficient_access(db.clone(), auth::Access::User))
        .and(with_db(db))
        .and_then(
            move |project_name: String,
                  user: db::admin::User,
                  db: DBRef| async move {
                let mut db = db.lock().await;
                match db.remove_project(user.id(), project_name.as_str()).await
                {
                    Ok(()) => {
                        Ok(
                            warp::reply::with_status(warp::reply(),
                            StatusCode::NO_CONTENT)
                        )
                    }
                    Err(e) => Err(warp::reject::custom(e)),
                }
            },
        )
}

/// Get user's projects
fn get_user_projects(
    db: DBRef,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("get" / "projects")
        .and(warp::get())
        .and(sufficient_access(db.clone(), auth::Access::User))
        .and(with_db(db))
        .and_then(move |user: db::admin::User, db: DBRef| async move {
            match db.lock().await.get_user_projects(user.id()).await {
                Ok(projects) => Ok(warp::reply::json(&projects)),
                Err(e) => Err(warp::reject::custom(e)),
            }
        })
}

/// Get a specific project
fn get_user_project(
    db: DBRef,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("get" / "project" / String)
        .and(warp::get())
        .and(sufficient_access(db.clone(), auth::Access::User))
        .and(with_db(db))
        .and_then(move |name: String, user: User, db: DBRef| async move {
            match db
                .lock()
                .await
                .get_user_project(user.id(), name.as_str())
                .await
            {
                Ok(projects) => Ok(warp::reply::json(&projects)),
                Err(e) => Err(warp::reject::custom(e)),
            }
        })
}

/// Create table in a user's database
fn create_table(
    db: DBRef,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("project" / String / "create" / "table")
        .and(warp::put())
        .and(sufficient_access(db.clone(), auth::Access::User))
        .and(with_db(db.clone()))
        .and_then(extract_project)
        .and(warp::body::json())
        .and(with_db(db))
        .and_then(
            move |project: db::admin::Project,
                  table: db::user::table::TableMeta,
                  db: DBRef| async move {
                match db.lock().await.create_user_table(&project, &table).await
                {
                    Ok(()) => Ok(reply_no_content()),
                    Err(e) => Err(warp::reject::custom(e)),
                }
            },
        )
}

/// Remove table from a user's database
fn remove_table(
    db: DBRef,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("project" / String / "remove" / "table" / String)
        .and(warp::delete())
        .and(sufficient_access(db.clone(), auth::Access::User))
        .and(with_db(db.clone()))
        .and_then(extract_project_and_table)
        .and(with_db(db))
        .and_then(
            move |(project, table_name): (Project, String),
                  db: DBRef| async move {
                match db
                    .lock()
                    .await
                    .remove_user_table(&project, table_name.as_str())
                    .await
                {
                    Ok(()) => Ok(reply_no_content()),
                    Err(e) => Err(warp::reject::custom(e)),
                }
            },
        )
}

/// Get a list of table names in a user's database
fn get_table_names(
    db: DBRef,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("project" / String / "get" / "tablenames")
        .and(warp::get())
        .and(sufficient_access(db.clone(), auth::Access::User))
        .and(with_db(db.clone()))
        .and_then(extract_project)
        .and(with_db(db))
        .and_then(move |project: Project, db: DBRef| async move {
            match db.lock().await.get_user_table_names(&project).await {
                Ok(tn) => Ok(warp::reply::json(&tn)),
                Err(e) => Err(warp::reject::custom(e)),
            }
        })
}

/// Get user table metadata
fn get_table_meta(
    db: DBRef,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("project" / String / "get" / "table" / String / "meta")
        .and(warp::get())
        .and(sufficient_access(db.clone(), auth::Access::User))
        .and(with_db(db.clone()))
        .and_then(extract_project_and_table)
        .and(with_db(db))
        .and_then(move |(project, table_name): (Project, String), db: DBRef| {
            async move {
                match db
                    .lock()
                    .await
                    .get_user_table_meta(&project, table_name.as_str())
                    .await
                {
                    Ok(tm) => Ok(warp::reply::json(&tm)),
                    Err(e) => Err(warp::reject::custom(e))
                }
            }
        })
}

/// Get all table metadata for a project
fn get_all_meta(
    db: DBRef,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("project" / String / "get" / "meta")
        .and(warp::get())
        .and(sufficient_access(db.clone(), auth::Access::User))
        .and(with_db(db.clone()))
        .and_then(extract_project)
        .and(with_db(db))
        .and_then(move |project: Project, db: DBRef| async move {
            match db.lock().await.get_all_meta(&project).await {
                Ok(tm) => Ok(warp::reply::json(&tm)),
                Err(e) => Err(warp::reject::custom(e)),
            }
        })
}

/// Insert data into a user's table
fn insert_data(
    db: DBRef,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("project" / String / "insert" / String)
        .and(warp::put())
        .and(sufficient_access(db.clone(), auth::Access::User))
        .and(with_db(db.clone()))
        .and_then(extract_project_and_table)
        .and(warp::body::json())
        .and(with_db(db))
        .and_then(
            move |(project, table_name): (Project, String),
                  data: Vec<RowJson>,
                  db: DBRef| {
                async move {
                    match db
                        .lock()
                        .await
                        .insert_user_table_data(
                            &project,
                            table_name.as_str(),
                            &data,
                        )
                        .await
                    {
                        Ok(()) => Ok(reply_no_content()),
                        Err(e) => Err(warp::reject::custom(e)),
                    }
                }
            },
        )
}

/// Remove all data from a user's table
fn remove_all_user_table_data(
    db: DBRef,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("project" / String / "remove" / String / "all")
        .and(warp::delete())
        .and(sufficient_access(db.clone(), auth::Access::User))
        .and(with_db(db.clone()))
        .and_then(extract_project_and_table)
        .and(with_db(db))
        .and_then(
            move |(project, table_name): (Project, String),
                  db: DBRef| {
                async move {
                    match db
                        .lock()
                        .await
                        .remove_all_user_table_data(
                            &project,
                            table_name.as_str(),
                        )
                        .await
                    {
                        Ok(()) => Ok(reply_no_content()),
                        Err(e) => Err(warp::reject::custom(e)),
                    }
                }
            },
        )
}

/// Get data from a user's table
fn get_table_data(
    db: DBRef,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("project" / String / "get" / "table" / String / "data")
        .and(warp::get())
        .and(sufficient_access(db.clone(), auth::Access::User))
        .and(with_db(db.clone()))
        .and_then(extract_project_and_table)
        .and(with_db(db))
        .and_then(move |(project, table_name): (Project, String), db: DBRef| {
            async move {
                match db
                    .lock()
                    .await
                    .get_user_table_data(&project, table_name.as_str())
                    .await
                {
                    Ok(td) => Ok(warp::reply::json(&td)),
                    Err(e) => Err(warp::reject::custom(e))
                }
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

    async fn gen_user_tok(admindb: DBRef) -> auth::Token {
        admindb
            .lock()
            .await
            .generate_session_token(auth::EmailPassword {
                email: "user@example.com".to_string(),
                password: "user".to_string(),
            })
            .await
            .unwrap()
    }

    async fn gen_admin_tok(admindb: DBRef) -> auth::Token {
        admindb
            .lock()
            .await
            .generate_session_token(auth::EmailPassword {
                email: "admin@example.com".to_string(),
                password: "admin".to_string(),
            })
            .await
            .unwrap()
    }

    /// Meant to test individual filters given good input
    struct FilterTester {
        method: String,
        path: String,
        json: Option<Box<dyn erased_serde::Serialize>>,
        headers: std::collections::HashMap<String, String>,
        status: Option<StatusCode>,
        body: Option<Vec<u8>>,
        headers_response: Option<warp::http::HeaderMap>,
    }

    impl FilterTester {
        pub fn new() -> Self {
            Self {
                method: "".to_string(),
                path: "".to_string(),
                json: None,
                headers: std::collections::HashMap::new(),
                status: None,
                body: None,
                headers_response: None,
            }
        }
        pub fn method(mut self, method: &str) -> Self {
            self.method = method.to_string();
            self
        }
        pub fn path<T: AsRef<str>>(mut self, path: T) -> Self {
            self.path = path.as_ref().to_string();
            self
        }
        pub fn json(mut self, val: impl serde::Serialize + 'static) -> Self {
            self.json = Some(Box::new(val));
            self
        }
        pub fn header<T: AsRef<str>>(mut self, name: &str, value: T) -> Self {
            self.headers
                .insert(name.to_string(), value.as_ref().to_string());
            self
        }
        pub fn bearer_header(self, tok: &str) -> Self {
            self.header("Authorization", format!("Bearer {}", tok))
        }
        pub fn origin_header(self) -> Self {
            self.header("Origin", "test")
        }
        pub async fn reply<F>(mut self, f: &F) -> Self
        where
            F: warp::Filter + 'static,
            F::Extract: warp::Reply + Send,
        {
            let mut req = warp::test::request()
                .method(self.method.as_str())
                .path(self.path.as_str());
            if let Some(v) = &self.json {
                req = req.json(&v);
            }
            for (name, value) in &self.headers {
                req = req.header(name, value);
            }
            let resp = req.reply(f).await;
            self.status = Some(resp.status());
            self.body = Some((&*resp.body()).to_vec());
            self.headers_response = Some(resp.headers().clone());
            self
        }
        pub fn expect_status(self, status: StatusCode) -> Self {
            assert_eq!(
                self.status.unwrap(),
                status,
                "status of {} method to {} path",
                self.method,
                self.path
            );
            self
        }
        pub fn expect_body<T>(&self) -> T
        where
            T: serde::de::DeserializeOwned,
        {
            let bod = serde_json::from_slice::<T>(&self.body.clone().unwrap());
            assert!(
                bod.is_ok(),
                "body of {} method to {} path",
                self.method,
                self.path
            );
            bod.unwrap()
        }
        pub fn expect_error<T: AsRef<str>>(self, msg: T) {
            let bod = self.expect_body::<String>();
            assert_eq!(
                bod,
                msg.as_ref(),
                "error of {} method to {} path",
                self.method,
                self.path
            );
        }
        pub fn expect_header<T: AsRef<str>>(
            self,
            header_name: &str,
            header_content: T,
        ) {
            if let Some(head) = self.headers_response.unwrap().get(header_name)
            {
                assert_eq!(
                    head,
                    header_content.as_ref(),
                    "header of {} method to {} path",
                    self.method,
                    self.path
                );
            } else {
                panic!(
                    "Missing header {} of {} method to {} path",
                    header_name, self.method, self.path
                )
            }
        }
        pub fn expect_no_header(self, header_name: &str) {
            assert!(self.headers_response.unwrap().get(header_name).is_none());
        }
        pub fn expect_origin_header(self) {
            self.expect_header("access-control-allow-origin", "test");
        }
    }

    #[tokio::test]
    async fn test_api() {
        let _ = pretty_env_logger::try_init();

        let admindb =
            tests::create_test_admindb(TEST_DB_NAME, true, true).await;
        tests::insert_test_user(&admindb).await;

        let admindb_ref = Arc::new(Mutex::new(admindb));

        assert_eq!(admindb_ref.lock().await.get_name(), TEST_DB_NAME);

        // Individual filters given good input --------------------------------

        // Health check
        FilterTester::new()
            .method("GET")
            .path("/health")
            .reply(&health(admindb_ref.clone()))
            .await
            .expect_status(StatusCode::OK)
            .expect_body::<bool>();

        // Create/refresh/remove session token
        let tok = FilterTester::new()
            .method("POST")
            .path("/auth/session-token")
            .json(auth::EmailPassword {
                email: "user@example.com".to_string(),
                password: "user".to_string(),
            })
            .reply(&generate_session_token(admindb_ref.clone()))
            .await
            .expect_status(StatusCode::OK)
            .expect_body::<auth::Token>();
        let tok = FilterTester::new()
            .method("POST")
            .path(format!("/auth/refresh-token/{}", tok.token()))
            .reply(&refresh_token(admindb_ref.clone()))
            .await
            .expect_status(StatusCode::OK)
            .expect_body::<auth::Token>();
        FilterTester::new()
            .method("DELETE")
            .path(format!("/auth/remove-token/{}", tok.token()))
            .reply(&remove_token(admindb_ref.clone()))
            .await
            .expect_status(StatusCode::NO_CONTENT);
        drop(tok);

        // Generate tokens to be used below
        let admin_token_full = gen_admin_tok(admindb_ref.clone()).await;
        let admin_token = admin_token_full.token();
        let user_token_full = gen_user_tok(admindb_ref.clone()).await;
        let user_token = user_token_full.token();

        // Get user by token
        let usr = FilterTester::new()
            .method("GET")
            .path(format!("/get/user/by/token/{}", user_token))
            .reply(&get_user_by_token(admindb_ref.clone()))
            .await
            .expect_status(StatusCode::OK)
            .expect_body::<admin::User>();
        assert_eq!(usr.email(), "user@example.com");
        assert_eq!(usr.access(), auth::Access::User);
        drop(usr);

        // Get users
        FilterTester::new()
            .method("GET")
            .path("/get/users")
            .bearer_header(admin_token)
            .reply(&get_users(admindb_ref.clone()))
            .await
            .expect_status(StatusCode::OK)
            .expect_body::<Vec<admin::User>>();

        // Create/remove user
        FilterTester::new()
            .method("PUT")
            .path("/create/user")
            .json(auth::EmailPassword {
                email: "newuser@example.com".to_string(),
                password: "newpassword".to_string(),
            })
            .reply(&create_user(admindb_ref.clone()))
            .await
            .expect_status(StatusCode::NO_CONTENT);
        FilterTester::new()
            .method("DELETE")
            .path("/remove/user/newuser@example.com")
            .bearer_header(admin_token)
            .reply(&remove_user(admindb_ref.clone()))
            .await
            .expect_status(StatusCode::NO_CONTENT);

        // Test projects
        let test_project1 = db::admin::Project::new(1, "test");

        // Make sure the corresponding databases aren't present
        log::info!("remove test projects");
        crate::tests::remove_dbs(
            &*admindb_ref.lock().await,
            &[test_project1.get_dbname(TEST_DB_NAME).as_str()],
        )
        .await;

        // Create projects
        FilterTester::new()
            .method("PUT")
            .path(format!("/create/project/{}", test_project1.get_name()))
            .bearer_header(admin_token)
            .reply(&create_project(admindb_ref.clone()))
            .await
            .expect_status(StatusCode::NO_CONTENT);
        // Get them
        let projects_obtained = FilterTester::new()
            .method("GET")
            .path("/get/projects")
            .bearer_header(admin_token)
            .reply(&get_user_projects(admindb_ref.clone()))
            .await
            .expect_status(StatusCode::OK)
            .expect_body::<Vec<admin::Project>>();
        assert_eq!(projects_obtained.len(), 1);
        assert_eq!(projects_obtained[0].get_name(), test_project1.get_name());
        drop(projects_obtained);
        let project_obtained = FilterTester::new()
            .method("GET")
            .path(format!("/get/project/{}", test_project1.get_name()))
            .bearer_header(admin_token)
            .reply(&get_user_project(admindb_ref.clone()))
            .await
            .expect_status(StatusCode::OK)
            .expect_body::<admin::Project>();
        assert_eq!(project_obtained.get_name(), test_project1.get_name());
        drop(project_obtained);

        // Create table
        let table = crate::tests::get_test_primary_table();

        FilterTester::new()
            .method("PUT")
            .path("/project/test/create/table")
            .bearer_header(admin_token)
            .json(table.clone())
            .reply(&create_table(admindb_ref.clone()))
            .await
            .expect_status(StatusCode::NO_CONTENT);

        // Get table list
        let table_list = FilterTester::new()
            .method("GET")
            .path("/project/test/get/tablenames")
            .bearer_header(admin_token)
            .reply(&get_table_names(admindb_ref.clone()))
            .await
            .expect_status(StatusCode::OK)
            .expect_body::<Vec<String>>();
        assert_eq!(table_list, vec![table.name.clone()]);
        drop(table_list);

        // Get table metadata
        let table_meta = FilterTester::new()
            .method("GET")
            .path(format!("/project/test/get/table/{}/meta", table.name))
            .bearer_header(admin_token)
            .reply(&get_table_meta(admindb_ref.clone()))
            .await
            .expect_status(StatusCode::OK)
            .expect_body::<db::user::table::TableMeta>();
        assert_eq!(table_meta, table);
        drop(table_meta);

        // Get all metadata
        let all_meta = FilterTester::new()
            .method("GET")
            .path("/project/test/get/meta")
            .bearer_header(admin_token)
            .reply(&get_all_meta(admindb_ref.clone()))
            .await
            .expect_status(StatusCode::OK)
            .expect_body::<db::user::table::TableSpec>();
        assert_eq!(all_meta, vec![table.clone()]);
        drop(all_meta);

        // Insert table data
        let data = crate::tests::get_primary_data();
        FilterTester::new()
            .method("PUT")
            .path(format!("/project/test/insert/{}", table.name.as_str()))
            .bearer_header(admin_token)
            .json(data.clone())
            .reply(&insert_data(admindb_ref.clone()))
            .await
            .expect_status(StatusCode::NO_CONTENT);

        // Get table data
        let data_obtained = FilterTester::new()
            .method("GET")
            .path(format!(
                "/project/test/get/table/{}/data",
                table.name.as_str()
            ))
            .bearer_header(admin_token)
            .reply(&get_table_data(admindb_ref.clone()))
            .await
            .expect_status(StatusCode::OK)
            .expect_body::<Vec<RowJson>>();
        assert_eq!(data_obtained, data);
        drop(data_obtained);

        // Remove all table data
        FilterTester::new()
            .method("DELETE")
            .path(format!("/project/test/remove/{}/all", table.name.as_str()))
            .bearer_header(admin_token)
            .reply(&remove_all_user_table_data(admindb_ref.clone()))
            .await
            .expect_status(StatusCode::NO_CONTENT);

        // Remove table
        FilterTester::new()
            .method("DELETE")
            .path(format!(
                "/project/test/remove/table/{}",
                table.name.as_str()
            ))
            .bearer_header(admin_token)
            .reply(&remove_table(admindb_ref.clone()))
            .await
            .expect_status(StatusCode::NO_CONTENT);

        // Delete projects
        FilterTester::new()
            .method("DELETE")
            .path("/delete/project/test")
            .bearer_header(admin_token)
            .reply(&delete_project(admindb_ref.clone()))
            .await
            .expect_status(StatusCode::NO_CONTENT);

        // Rejections ---------------------------------------------------------

        let routes = routes(admindb_ref.clone(), "");

        // Wrong email
        FilterTester::new()
            .method("POST")
            .path("/auth/session-token")
            .json(auth::EmailPassword {
                email: "user1@example.com".to_string(),
                password: "user".to_string(),
            })
            .reply(&routes)
            .await
            .expect_status(StatusCode::UNAUTHORIZED)
            .expect_error("NoSuchUserEmail(\"user1@example.com\")");

        // Wrong password
        FilterTester::new()
            .method("POST")
            .path("/auth/session-token")
            .json(auth::EmailPassword {
                email: "user@example.com".to_string(),
                password: "user1".to_string(),
            })
            .reply(&routes)
            .await
            .expect_status(StatusCode::UNAUTHORIZED)
            .expect_error("WrongPassword(\"user1\")");

        // Wrong token
        FilterTester::new()
            .method("GET")
            .path("/get/user/by/token/123")
            .reply(&routes)
            .await
            .expect_status(StatusCode::UNAUTHORIZED)
            .expect_error("NoSuchToken(\"123\")");
        FilterTester::new()
            .method("GET")
            .path("/get/users")
            .bearer_header("123")
            .reply(&routes)
            .await
            .expect_status(StatusCode::UNAUTHORIZED)
            .expect_error("NoSuchToken(\"123\")");
        FilterTester::new()
            .method("POST")
            .path("/auth/refresh-token/123")
            .reply(&routes)
            .await
            .expect_status(StatusCode::UNAUTHORIZED)
            .expect_error("NoSuchToken(\"123\")");

        // Insufficient access
        FilterTester::new()
            .method("GET")
            .path("/get/users")
            .bearer_header(user_token)
            .reply(&routes)
            .await
            .expect_status(StatusCode::UNAUTHORIZED)
            .expect_error("InsufficientAccess");

        // Wrong authentication type
        FilterTester::new()
            .method("GET")
            .path("/get/users")
            .header("Authorization", "Basic a:a")
            .reply(&routes)
            .await
            .expect_status(StatusCode::UNAUTHORIZED)
            .expect_error("WrongAuthType(\"Basic\")");

        // Missing header
        FilterTester::new()
            .method("GET")
            .path("/get/users")
            .reply(&routes)
            .await
            .expect_status(StatusCode::UNAUTHORIZED)
            .expect_error("Missing request header \"Authorization\"");

        // Missing body
        FilterTester::new()
            .method("POST")
            .path("/auth/session-token")
            .reply(&routes)
            .await
            .expect_status(StatusCode::BAD_REQUEST)
            .expect_error(
                "Request body deserialize error: \
                EOF while parsing a value at line 1 column 0",
            );

        // Wrong method
        FilterTester::new()
            .method("GET")
            .path("/auth/session-token")
            .reply(&routes)
            .await
            .expect_status(StatusCode::METHOD_NOT_ALLOWED)
            .expect_error("HTTP method not allowed");

        // Delete a non-existent project
        FilterTester::new()
            .method("DELETE")
            .path("/delete/project/test_nonexistent")
            .bearer_header(admin_token)
            .reply(&routes)
            .await
            .expect_status(StatusCode::NOT_FOUND)
            .expect_error("NoSuchProject(1, \"test_nonexistent\")");

        // Get a non-existent project
        FilterTester::new()
            .method("GET")
            .path("/get/project/test_nonexistent")
            .bearer_header(admin_token)
            .reply(&routes)
            .await
            .expect_status(StatusCode::NOT_FOUND)
            .expect_error("NoSuchProject(1, \"test_nonexistent\")");

        // Create table in a non-existent project
        FilterTester::new()
            .method("PUT")
            .path("/project/test_nonexistent/create/table")
            .bearer_header(admin_token)
            .reply(&routes)
            .await
            .expect_status(StatusCode::NOT_FOUND)
            .expect_error("NoSuchProject(1, \"test_nonexistent\")");

        // Create a project that will be used later ---------------------------
        FilterTester::new()
            .method("PUT")
            .path("/create/project/test")
            .bearer_header(admin_token)
            .reply(&routes)
            .await;

        // Creating the same project twice
        FilterTester::new()
            .method("PUT")
            .path("/create/project/test")
            .bearer_header(admin_token)
            .reply(&routes)
            .await
            .expect_status(StatusCode::CONFLICT)
            .expect_error("ProjectAlreadyExists(1, \"test\")");

        log::info!("delete a non-existent table");
        FilterTester::new()
            .method("DELETE")
            .path("/project/test/remove/table/nonexistent")
            .bearer_header(admin_token)
            .reply(&routes)
            .await
            .expect_status(StatusCode::NOT_FOUND)
            .expect_error("NoSuchTable(\"nonexistent\")");

        // Delete the project created earlier ---------------------------------
        FilterTester::new()
            .method("DELETE")
            .path("/delete/project/test")
            .bearer_header(admin_token)
            .reply(&routes)
            .await;

        // Token too old
        let old_token = admindb_ref
            .lock()
            .await
            .generate_session_token(auth::EmailPassword {
                email: "admin@example.com".to_string(),
                password: "admin".to_string(),
            })
            .await
            .unwrap();
        sqlx::query(
            format!(
                "UPDATE \"token\" \
                    SET \"created\" = '2000-08-14 08:15:29.425665+10' \
                    WHERE \"token\" = '{}'",
                auth::hash_fast(old_token.token())
            )
            .as_str(),
        )
        .execute(admindb_ref.lock().await.get_pool())
        .await
        .unwrap();
        FilterTester::new()
            .method("GET")
            .path("/get/users")
            .bearer_header(old_token.token())
            .reply(&routes)
            .await
            .expect_status(StatusCode::UNAUTHORIZED)
            .expect_error("TokenTooOld");
        drop(old_token);

        // Not found
        FilterTester::new()
            .method("GET")
            .path("/")
            .reply(&routes)
            .await
            .expect_status(StatusCode::NOT_FOUND);

        // CORS ---------------------------------------------------------------

        // When not attached
        FilterTester::new()
            .method("GET")
            .path("/health")
            .header("Origin", "test")
            .reply(&routes)
            .await
            .expect_no_header("access-control-allow-origin");

        let routes_cors = routes_cors(admindb_ref.clone(), "");

        // When request is good
        FilterTester::new()
            .method("GET")
            .path("/health")
            .origin_header()
            .reply(&routes_cors)
            .await
            .expect_origin_header();

        log::info!("cors still present when request fails");
        FilterTester::new()
            .method("POST") // Wrong method
            .path("/health")
            .origin_header()
            .reply(&routes_cors)
            .await
            .expect_origin_header();

        // Options request
        FilterTester::new()
            .method("OPTIONS")
            .path("/health")
            .origin_header()
            .header("Access-Control-Request-Method", "GET")
            .reply(&routes_cors)
            .await
            .expect_status(StatusCode::OK)
            .expect_origin_header();

        log::info!("disallowed header");
        FilterTester::new()
            .method("OPTIONS")
            .path("/health")
            .origin_header()
            .header("Access-Control-Request-Method", "GET")
            .header("Access-Control-Request-Headers", "X-Username")
            .reply(&routes_cors)
            .await
            .expect_status(StatusCode::FORBIDDEN);

        // Path prefix --------------------------------------------------------

        let routes_prefixed = super::routes(admindb_ref.clone(), "prefix");

        FilterTester::new()
            .method("GET")
            .path("/prefix/health")
            .reply(&routes_prefixed)
            .await
            .expect_status(StatusCode::OK);

        // Remove the test database -------------------------------------------

        crate::tests::remove_test_db(&*admindb_ref.lock().await.get_db()).await;
    }
}
