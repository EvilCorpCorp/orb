use axum::Router;

pub mod configs;
pub mod simulator;

#[tokio::main]
async fn main() {
    // TODO: initialize logs


    // get configuration from config.toml
    let config = configs::read_config();

    // build our application with a route
    // TODO: add our endpoint to receive transactions
    let app = Router::new();

    // run our app with hyper, listening globally on port 3000
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(addr).await.expect("port to be free");
    axum::serve(listener, app).await.expect("server to start");
}
