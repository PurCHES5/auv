//! Daemon-owned NetEase Music Runner implementation.

use crate::DEFAULT_APP_ID;
use crate::api::v1 as proto;
use crate::api::v1::netease_music_service_server::{NeteaseMusicService, NeteaseMusicServiceServer};
use tonic::{Request, Response, Status};

#[derive(Default)]
struct Service;

#[tonic::async_trait]
impl NeteaseMusicService for Service {
  async fn get_now_playing(&self, request: Request<proto::GetNowPlayingRequest>) -> Result<Response<proto::GetNowPlayingResponse>, Status> {
    let requested =
      request.into_inner().application_bundle_id.filter(|value| !value.trim().is_empty()).unwrap_or_else(|| DEFAULT_APP_ID.to_string());
    let state = auv_media_macos::now_playing().map_err(|error| Status::unavailable(format!("now-playing read failed: {error}")))?;
    let state = if state.source_bundle_id.as_deref() == Some(requested.as_str()) {
      state
    } else {
      auv_media_macos::NowPlayingState::default()
    };
    Ok(Response::new(proto::GetNowPlayingResponse {
      present: state.present,
      is_playing: state.is_playing,
      source_bundle_id: state.source_bundle_id,
      title: state.title,
      artist: state.artist,
      album: state.album,
      duration_seconds: state.duration_seconds,
      elapsed_seconds: state.elapsed_seconds,
      playback_rate: state.playback_rate,
      content_item_id: state.content_item_id,
      supports_like: state.supports_like,
      is_liked: state.is_liked,
    }))
  }
}

#[cfg(unix)]
pub async fn serve_inherited() -> Result<(), String> {
  let (incoming, parent_disconnected) = auv_api_server::runner_transport::inherited_transport()?.into_parts();
  let service = NeteaseMusicServiceServer::new(Service);
  let (health_reporter, health) = tonic_health::server::health_reporter();
  health_reporter.set_serving::<NeteaseMusicServiceServer<Service>>().await;
  let reflection = tonic_reflection::server::Builder::configure()
    .register_encoded_file_descriptor_set(crate::api::FILE_DESCRIPTOR_SET)
    .build_v1()
    .map_err(|error| format!("failed to build NetEase Runner reflection: {error}"))?;
  tonic::transport::Server::builder()
    .add_service(health)
    .add_service(reflection)
    .add_service(service)
    .serve_with_incoming_shutdown(incoming, parent_disconnected)
    .await
    .map_err(|error| format!("NetEase Runner transport failed: {error}"))
}

#[cfg(not(unix))]
pub async fn serve_inherited() -> Result<(), String> {
  // TODO(netease-runner-windows-ipc): enable after the API server transport
  // supports daemon-owned inherited named-pipe handles.
  Err("the NetEase Runner currently requires Unix inherited IPC".to_string())
}
