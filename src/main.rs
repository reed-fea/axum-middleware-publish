use axum::{
    extract::Request,
    http::{StatusCode, header::AUTHORIZATION},
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};

#[derive(Clone, Debug)]
struct CurrentUser {
    username: String,
}

async fn auth(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let auth_header = if let Some(auth_header) = auth_header {
        auth_header
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    if let Some(current_user) = authorize_current_user(auth_header).await {
        // insert the current user into a request extension so the handler can
        // extract it
        req.extensions_mut().insert(current_user);
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

async fn simulate_authentication(auth_token: String) -> Option<CurrentUser> {
    // 在真实场景中，执行实际的身份验证操作。
    // 这里是一个模拟，休眠 2 秒然后返回一个用户。
    sleep(Duration::from_secs(2)).await;

    if auth_token == "valid_token" {
        Some(CurrentUser {
            username: "JohnDoe".to_string(),
        })
    } else {
        None
    }
}

async fn authorize_current_user(auth_token: &str) -> Option<CurrentUser> {
    // 克隆 auth_token 以确保它不会逃逸函数体外
    let auth_token = auth_token.to_string();

    // 使用 tokio::spawn 并发运行异步函数
    let handle = tokio::spawn(simulate_authentication(auth_token));

    // 使用 tokio::time::timeout 设置超时
    let timeout_result = tokio::time::timeout(Duration::from_secs(5), handle).await;

    // 匹配超时结果
    match timeout_result {
        Ok(result) => result.unwrap(), // 返回异步任务的结果
        Err(_) => None, // 超时的情况，返回 None
    }
}

#[tokio::main]
async fn main() {
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .layer(middleware::from_fn(auth))
        // `POST /users` goes to `create_user`
        .route("/users", post(create_user));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root(Extension(current_user): Extension<CurrentUser>) -> &'static str {
    println!("{:#?}", current_user.username);
    "Hello, World!"
}

async fn create_user(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
    // insert your application logic here
    let user = User {
        id: 1337,
        username: payload.username,
    };

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(user))
}

// the input to our `create_user` handler
#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

// the output to our `create_user` handler
#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}
