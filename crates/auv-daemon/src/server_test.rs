use auv_api_client::protocol::grpc::Client as GrpcClient;
use auv_api_proto::auv::api::daemon::v1 as proto;
use prost::Message as _;
use tokio_util::sync::CancellationToken;

use super::*;

fn config(listeners: Vec<ListenEndpoint>, root: &std::path::Path) -> Config {
  Config {
    listeners,
    store_root: root.join("store"),
    pairing_store: None,
    discovery_file: None,
    publish_discovery: false,
    daemon_idle_timeout: None,
    runner_providers: Vec::new(),
    first_party_runners: Default::default(),
  }
}

#[tokio::test]
async fn typed_control_and_rest_share_the_daemon_backend() {
  let root = tempfile::tempdir().unwrap();
  let server = Server::bind(config(
    vec![ListenEndpoint::Tcp {
      host: "127.0.0.1".into(),
      port: 0,
    }],
    root.path(),
  ))
  .await
  .unwrap();
  let BoundEndpoint::Tcp(address) = server.endpoint() else {
    panic!("TCP endpoint")
  };
  let address = *address;
  let shutdown = CancellationToken::new();
  let task = tokio::spawn(server.serve(shutdown.clone()));
  let client = GrpcClient::connect(format!("http://{address}").parse().unwrap()).await.unwrap();
  let devices = client.devices().list_devices().await.unwrap();
  assert_eq!(devices.len(), 1);
  assert!(devices[0].local);

  let response = reqwest::Client::builder()
    .http2_prior_knowledge()
    .build()
    .unwrap()
    .get(format!("http://{address}/apis/auv/daemon/v1/devices"))
    .send()
    .await
    .unwrap();
  assert_eq!(response.status(), reqwest::StatusCode::OK);
  let listed = proto::ListDevicesResponse::decode(response.bytes().await.unwrap()).unwrap();
  assert_eq!(listed.devices, devices);
  shutdown.cancel();
  task.await.unwrap().unwrap();
}

#[tokio::test]
async fn local_owner_and_paired_bearer_share_live_pairing_administration() {
  let root = tempfile::tempdir().unwrap();
  let mut server_config = config(
    vec![
      ListenEndpoint::Tcp {
        host: "127.0.0.1".into(),
        port: 0,
      },
      ListenEndpoint::Remote {
        host: "127.0.0.1".into(),
        port: 0,
      },
    ],
    root.path(),
  );
  server_config.pairing_store = Some(root.path().join("pairings.json"));
  let server = Server::bind(server_config).await.unwrap();
  let local = server
    .endpoints()
    .iter()
    .find_map(|endpoint| match endpoint {
      BoundEndpoint::Tcp(value) => Some(*value),
      _ => None,
    })
    .unwrap();
  let remote = server
    .endpoints()
    .iter()
    .find_map(|endpoint| match endpoint {
      BoundEndpoint::Remote(value) => Some(*value),
      _ => None,
    })
    .unwrap();
  let shutdown = CancellationToken::new();
  let task = tokio::spawn(server.serve(shutdown.clone()));

  let local_client = GrpcClient::connect(format!("http://{local}").parse().unwrap()).await.unwrap();
  let token_a = local_client.pairing().create_pairing_token(proto::CreatePairingTokenRequest { ttl: None }).await.unwrap().token;
  let enrollment_a = auv_api_client::protocol::grpc::clients::daemon::v1::pairing::Client::pair_device(
    format!("http://{remote}").parse().unwrap(),
    proto::PairDeviceRequest {
      token: token_a,
      device_id: "paired-a".into(),
      label: "Paired A".into(),
    },
  )
  .await
  .unwrap();
  let paired_a = GrpcClient::connect_paired(auv_api_client::PairedConnectConfig {
    endpoint: format!("http://{remote}").parse().unwrap(),
    device_credential: enrollment_a.device_credential,
  })
  .await
  .unwrap();
  let token_b = paired_a.pairing().create_pairing_token(proto::CreatePairingTokenRequest { ttl: None }).await.unwrap().token;
  let enrollment_b = auv_api_client::protocol::grpc::clients::daemon::v1::pairing::Client::pair_device(
    format!("http://{remote}").parse().unwrap(),
    proto::PairDeviceRequest {
      token: token_b,
      device_id: "paired-b".into(),
      label: "Paired B".into(),
    },
  )
  .await
  .unwrap();
  let paired_b = GrpcClient::connect_paired(auv_api_client::PairedConnectConfig {
    endpoint: format!("http://{remote}").parse().unwrap(),
    device_credential: enrollment_b.device_credential,
  })
  .await
  .unwrap();
  paired_a.pairing().set_enabled("Paired B", false).await.unwrap();
  assert_eq!(paired_b.devices().list_devices().await.unwrap_err().code(), tonic::Code::Unauthenticated);
  paired_a.pairing().set_enabled("paired-b", true).await.unwrap();
  paired_b.devices().list_devices().await.unwrap();
  paired_b.pairing().revoke_device_credential("paired-a").await.unwrap();
  assert_eq!(paired_a.devices().list_devices().await.unwrap_err().code(), tonic::Code::Unauthenticated);
  shutdown.cancel();
  task.await.unwrap().unwrap();
}
