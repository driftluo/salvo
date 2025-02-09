use salvo::prelude::*;

#[fn_handler]
async fn hello_world() -> &'static str {
    "Hello World"
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    tracing::info!("Listening on http://0.0.0.0:7878");
    Server::new(TcpListener::bind("0.0.0.0:7878")).serve(route()).await;
}

// only allow access from http://localhost:7878/, http://0.0.0.0:7878/ will get not found page.
fn route() -> Router {
    Router::new()
        .filter_fn(|req, _| {
            let host = req.uri().host().unwrap_or_default();
            host == "localhost"
        })
        .get(hello_world)
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_custom_filter() {
        use salvo::hyper;
        use salvo::prelude::*;

        let service = Service::new(super::route());

        async fn access(service: &Service, host: &str) -> String {
            let req = hyper::Request::builder().method("GET").uri(format!("http://{}/", host));
            let req: Request = req.body(hyper::Body::empty()).unwrap().into();
            service.handle(req).await.take_text().await.unwrap()
        }

        assert!(access(&service, "127.0.0.1").await.contains("404: Not Found"));
        assert_eq!(access(&service, "localhost").await, "Hello World");
    }
}
