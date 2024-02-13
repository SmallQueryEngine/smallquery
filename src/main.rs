use smallquery;
use tokio;

#[tokio::main]
async fn main() {
    std::process::exit(smallquery::run().await);
}
