use salvo::prelude::*;

#[fn_handler]
async fn hello_world() -> &'static str {
    "Hello World"
}
#[fn_handler]
async fn hello_world1() -> Result<&'static str, ()> {
    Ok("Hello World1")
}
#[fn_handler]
async fn hello_world2(res: &mut Response) {
    res.render("Hello World2");
}
#[fn_handler]
async fn hello_world3(_req: &mut Request, res: &mut Response) {
    res.render(Text::Plain("Hello World3"));
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    tracing::info!("Listening on http://0.0.0.0:7878");
    Server::new(TcpListener::bind("0.0.0.0:7878")).serve(route()).await;
}

fn route() -> Router {
    Router::new()
        .get(hello_world)
        .push(Router::with_path("hello1").get(hello_world1))
        .push(Router::with_path("hello2").get(hello_world2))
        .push(Router::with_path("hello3").get(hello_world3))
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_hello_world() {
        use salvo::hyper;
        use salvo::prelude::*;

        let service = Service::new(super::route());

        async fn access(service: &Service, name: &str) -> String {
            let req = hyper::Request::builder()
                .method("GET")
                .uri(format!("http://127.0.0.1:7878/{}", name));
            let req: Request = req.body(hyper::Body::empty()).unwrap().into();
            service.handle(req).await.take_text().await.unwrap()
        }

        assert_eq!(access(&service, "hello1").await, "Hello World1");
        assert_eq!(access(&service, "hello2").await, "Hello World2");
        assert_eq!(access(&service, "hello3").await, "Hello World3");
    }
}
