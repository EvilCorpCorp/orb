use axum::{
    routing::{
		get,
		post
	},
    Router,
	Json,
	response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use tokio_postgres::{Client, NoTls, Error};

pub mod configs;
pub mod simulator;

#[derive(Deserialize, Serialize)]
struct InputData {
    name: String,
    txId: u8,
}

#[derive(sqlx::FromRow)]
struct Tx {
    //transatoin
}


//Create connection
async fn connect_to_db() -> Result<Client, Error> {
    // Replace with your database URL
    let (client, connection) = tokio_postgres::connect(
        "host=localhost port=5432 user=postgres password=password dbname=tx_db",
        NoTls,
    )
    .await?;

    // Spawn a new task to manage the database connection
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    Ok(client)
}

async fn signal(client: &Client, txId: i32) {
    let tx = client.execute("SELECT * FROM transactions WHERE txId = $1", &[&txId]).await?;
    Ok(tx);
    let updatedTx = client.execute("UPDATE transactios SET flagged = TRUE WHERE txId = $1",  &[&txId]).await?;
    Ok(updatedTx);

}


// Ideally we shoudl define together the type, and just think a bit of how can we conver it
// Other route to just like get info from the db
// Do we setup a db ?
// What is the priority
async fn recv_t(Json(payload): Json<InputData>) -> String {
    let client = connect_to_db().await?;

    
    //Once connected, should check that the transactions is not yet registered
    //Check that is well malicious ??
    //Maybe the user put a reason or idk
    //We should have a list of simulated transaction, to check weather or not we already simulated
    //or not
    format!("Received name: {}, age: {}", payload.name, payload.txId)
}


#[tokio::main]
async fn main() {
    // TODO: initialize logs


    // get configuration from config.toml
    let config = configs::read_config();

    // build our application with a route
    // TODO: add our endpoint to receive transactions
    let app = Router::new()
    .route("/signalement", post(recv_t));

    // run our app with hyper, listening globally on port 3000
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(addr).await.expect("port to be free");
    axum::serve(listener, app).await.expect("server to start");
}

