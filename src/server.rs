use crate::templates::{HelloTemplate, IndexTemplate};
use askama::Template;
use chrono::Datelike;
use firststep_name_lib::{SitesFile, check_username_from_webserver, download_sites_data};
use futures_util::StreamExt;
use poem::{
    Endpoint, EndpointExt, IntoResponse, Route, Server, endpoint::StaticFilesEndpoint, get,
    handler, http::StatusCode, listener::TcpListener, web::Data, web::Html, web::Path,
    web::websocket::WebSocket,
};
use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[handler]
fn hello(Path(name): Path<String>) -> impl IntoResponse {
    HelloTemplate {
        name: &name,
        title: "Cześć!",
        year: chrono::Utc::now().year(),
    }
    .render()
    .map_err(|e| {
        eprintln!("Failed to render template: {}", e);
        poem::http::StatusCode::INTERNAL_SERVER_ERROR
    })
    .unwrap_or_else(|_| "Internal Server Error".to_string())
}

#[handler]
fn ok() -> impl IntoResponse {
    "ok"
}

#[handler]
fn ws_handler(
    Path(username): Path<String>,
    ws: WebSocket,
    client: Data<&Client>,
    sites_data: Data<&Arc<SitesFile>>,
) -> impl IntoResponse {
    let client = client.clone();
    let sites_data = sites_data.clone();

    ws.on_upgrade(move |socket| async move {
        println!("WebSocket connected for username: {}", username);
        let (sink, _stream) = socket.split();
        let sink = Arc::new(Mutex::new(sink));

        tokio::spawn(async move {
            let _results = check_username_from_webserver(
                &client,
                &username,
                &sites_data.sites,
                1, // Use a single thread for web server checks
                Some(sink),
            )
            .await;
        });
    })
}

#[handler]
async fn index_get() -> impl IntoResponse {
    let template = IndexTemplate {
        title: "First Step - Name by Nutek Security",
        year: chrono::Utc::now().year(),
    };

    match template.render() {
        Ok(html_content) => Html(html_content),
        Err(e) => {
            eprintln!("Failed to render index template: {}", e);
            Html(StatusCode::INTERNAL_SERVER_ERROR.as_str().to_string())
        }
    }
}

#[handler]
async fn fetch_json() -> Result<poem::web::Json<Value>, StatusCode> {
    let client = Client::new();
    let url =
        "https://github.com/buahaha/multilanguage-hello-json/raw/refs/heads/master/hello.json";
    match client.get(url).send().await {
        Ok(response) => match response.json::<Value>().await {
            Ok(json) => Ok(poem::web::Json(json)),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn app() -> impl Endpoint {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();

    let sites_data: Arc<SitesFile> = Arc::new({
        let json_file = "social_sites.json";
        if let Err(e) = download_sites_data(&client, json_file).await {
            eprintln!("Failed to download sites data: {}", e);
        }
        let file = std::fs::File::open(json_file).expect("Failed to open sites data file");
        let reader = std::io::BufReader::new(file);
        serde_json::from_reader(reader).expect("Failed to parse sites data")
    });

    Route::new()
        .at("/hello/:name", get(hello))
        .at("/is_ok", get(ok))
        .at("/", get(index_get))
        .at("/ws/:username", get(ws_handler))
        .at("/fetch_json", get(fetch_json))
        .nest("/static", StaticFilesEndpoint::new("./static"))
        .data(client)
        .data(sites_data)
}

/// Starts the web server to handle requests
pub async fn run_server() -> Result<(), std::io::Error> {
    let app = app().await;
    println!("Starting server on http://127.0.0.1:3003");
    Server::new(TcpListener::bind("0.0.0.0:3003"))
        .run_with_graceful_shutdown(
            app,
            async move {
                let _ = tokio::signal::ctrl_c().await;
                println!("Shutting down server...");
            },
            None,
        )
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use poem::test::TestClient;

    fn send_ctrl_c_signal() {
        let pid = std::process::id() as i32;

        #[cfg(unix)]
        unsafe {
            libc::kill(pid, libc::SIGINT);
        }

        #[cfg(windows)]
        unsafe {
            winapi::um::wincon::GenerateConsoleCtrlEvent(winapi::um::wincon::CTRL_C_EVENT, 0);
        }
    }

    use serial_test::serial;

    // #[tokio::test]
    // async fn test_run_server() {
    //     let server_handle = tokio::spawn(async { run_server().await });
    //     tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    //     send_ctrl_c_signal();
    //     let result = server_handle.await.expect("Server task panicked");
    //     assert!(result.is_ok(), "Server failed to start: {:?}", result.err());
    // }

    #[tokio::test]
    #[serial]
    async fn test_is_ok() {
        let cli = TestClient::new(app().await);
        let respo = cli.get("/is_ok").send().await;
        respo.assert_status_is_ok();
        respo.assert_content_type("text/plain; charset=utf-8");
        assert_eq!(respo.0.into_body().into_string().await.unwrap(), "ok");
    }

    #[tokio::test]
    #[serial]
    async fn test_index() {
        let cli = TestClient::new(app().await);

        let res = cli.get("/").send().await;
        res.assert_status_is_ok();
        res.assert_content_type("text/html; charset=utf-8");

        let html = res.0.into_body().into_string().await.unwrap();
        assert!(html.contains("Check the availability of your name on social platforms"));

        let resp = cli.get("/").query("username", &"jankos").send().await;
        resp.assert_status_is_ok();
    }

    #[tokio::test]
    #[serial]
    async fn test_hello() {
        let cli = TestClient::new(app().await);

        let name = "suczkom";
        let res = cli.get(format!("/hello/{}", name)).send().await;
        res.assert_status_is_ok();
        res.assert_content_type("text/plain; charset=utf-8");

        let body = res.0.into_body().into_string().await.unwrap();
        assert!(
            body.contains(name),
            "Response body should contain the name '{}'",
            name
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_fetch_json() {
        let cli = TestClient::new(app().await);

        let res = cli.get("/fetch_json").send().await;
        res.assert_status_is_ok();
        res.assert_content_type("application/json; charset=utf-8");

        let body = res.0.into_body().into_string().await.unwrap();
        let json: Value = serde_json::from_str(&body).expect("Failed to parse JSON");

        assert!(json.is_array(), "Response should be a JSON array");
        assert!(
            !json.as_array().unwrap().is_empty(),
            "JSON array should not be empty"
        );
        assert!(
            json[0].get("hello").is_some(),
            "JSON should contain 'hello' key"
        );
    }
}
