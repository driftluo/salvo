use std::fs::create_dir_all;
use std::path::Path;

use salvo::extra::size_limiter::max_size;
use salvo::prelude::*;

#[fn_handler]
async fn index(res: &mut Response) {
    res.render(Text::Html(INDEX_HTML));
}
#[fn_handler]
async fn upload(req: &mut Request, res: &mut Response) {
    let file = req.get_file("file").await;
    if let Some(file) = file {
        let dest = format!("temp/{}", file.file_name().unwrap_or("file"));
        tracing::debug!(dest = %dest, "upload file");
        if let Err(e) = std::fs::copy(&file.path(), Path::new(&dest)) {
            res.set_status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Text::Plain(format!("file not found in request: {}", e)));
        } else {
            res.render(Text::Plain(format!("File uploaded to {}", dest)));
        }
    } else {
        res.set_status_code(StatusCode::BAD_REQUEST);
        res.render(Text::Plain("file not found in request"));
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    create_dir_all("temp").unwrap();
    let router = Router::new()
        .get(index)
        .push(
            Router::new()
                .hoop(max_size(1024 * 1024 * 10))
                .path("limited")
                .post(upload),
        )
        .push(Router::with_path("unlimit").post(upload));
    tracing::info!("Listening on http://0.0.0.0:7878");
    Server::new(TcpListener::bind("0.0.0.0:7878")).serve(router).await;
}

static INDEX_HTML: &str = r#"<!DOCTYPE html>
<html>
    <head>
        <title>Upload file</title>
    </head>
    <body>
        <h1>Upload file</h1>
        <form action="/unlimit" method="post" enctype="multipart/form-data">
            <h3>Unlimit</h3>
            <input type="file" name="file" />
            <input type="submit" value="upload" />
        </form>
        <form action="/limited" method="post" enctype="multipart/form-data">
            <h3>Limited 10MiB</h3>
            <input type="file" name="file" />
            <input type="submit" value="upload" />
        </form>
    </body>
</html>
"#;
