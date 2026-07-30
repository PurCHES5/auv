//! Root `auv` core command frontend.

#[tokio::main]
async fn main() {
  std::process::exit(auv_cli::cli_frontend::exit_status(auv_cli::cli_frontend::run_root().await));
}
