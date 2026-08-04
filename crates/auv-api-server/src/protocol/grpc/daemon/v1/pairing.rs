//! Pairing service adapter.

use auv_api_proto::auv::api::daemon::v1 as proto;
use auv_api_proto::auv::api::daemon::v1::pairing_service_server::PairingService;
use tonic::{Request, Response, Status};

use crate::server::RequestAuth;

#[derive(Clone)]
pub(crate) struct PairingServiceGrpc {
  auth: RequestAuth,
}

impl PairingServiceGrpc {
  pub(crate) fn new(auth: RequestAuth) -> Self {
    Self { auth }
  }

  fn pairing(&self) -> Result<std::sync::Arc<dyn crate::control::Pairing>, Status> {
    self.auth.pairing().ok_or_else(|| Status::unimplemented("pairing is not configured"))
  }
}

#[tonic::async_trait]
impl PairingService for PairingServiceGrpc {
  async fn create_pairing_token(
    &self,
    request: Request<proto::CreatePairingTokenRequest>,
  ) -> Result<Response<proto::CreatePairingTokenResponse>, Status> {
    self.auth.authenticate(&request)?;
    let request = request.into_inner();
    let lifetime = request.ttl.as_ref().map(proto_duration).transpose()?;
    let token = self.pairing()?.issue_token(lifetime).map_err(pairing_status)?;
    let expires_at = lifetime.map(|lifetime| {
      let deadline = std::time::SystemTime::now() + lifetime;
      let duration = deadline.duration_since(std::time::UNIX_EPOCH).expect("current time is after Unix epoch");
      prost_types::Timestamp {
        seconds: i64::try_from(duration.as_secs()).unwrap_or(i64::MAX),
        nanos: i32::try_from(duration.subsec_nanos()).expect("nanoseconds fit i32"),
      }
    });
    Ok(Response::new(proto::CreatePairingTokenResponse {
      token: token.token,
      expires_at,
    }))
  }

  async fn pair_device(&self, request: Request<proto::PairDeviceRequest>) -> Result<Response<proto::PairDeviceResponse>, Status> {
    let request = request.into_inner();
    if request.token.is_empty() || request.device_id.is_empty() {
      return Err(Status::invalid_argument("token and device_id are required"));
    }
    let enrollment = self.pairing()?.enroll(&request.token, request.device_id.clone(), request.label).map_err(pairing_status)?;
    Ok(Response::new(proto::PairDeviceResponse {
      device_id: enrollment.device_id,
      device_credential: enrollment.credential,
    }))
  }

  async fn revoke_device_credential(
    &self,
    request: Request<proto::RevokeDeviceCredentialRequest>,
  ) -> Result<Response<proto::RevokeDeviceCredentialResponse>, Status> {
    self.auth.authenticate(&request)?;
    let device_id = request.into_inner().device_id;
    if device_id.is_empty() {
      return Err(Status::invalid_argument("device_id is required"));
    }
    let revoked = self.pairing()?.revoke_device_credentials(&device_id).map_err(pairing_status)?;
    Ok(Response::new(proto::RevokeDeviceCredentialResponse { revoked }))
  }

  async fn set_paired_device_enabled(
    &self,
    request: Request<proto::SetPairedDeviceEnabledRequest>,
  ) -> Result<Response<proto::SetPairedDeviceEnabledResponse>, Status> {
    self.auth.authenticate(&request)?;
    let request = request.into_inner();
    if request.device_selector.is_empty() {
      return Err(Status::invalid_argument("device_selector is required"));
    }
    let changed = self.pairing()?.set_enabled(&request.device_selector, request.enabled).map_err(pairing_status)?;
    Ok(Response::new(proto::SetPairedDeviceEnabledResponse { changed }))
  }

  async fn unpair_device(&self, request: Request<proto::UnpairDeviceRequest>) -> Result<Response<proto::UnpairDeviceResponse>, Status> {
    self.auth.authenticate(&request)?;
    let selector = request.into_inner().device_selector;
    if selector.is_empty() {
      return Err(Status::invalid_argument("device_selector is required"));
    }
    let removed = self.pairing()?.unpair(&selector).map_err(pairing_status)?;
    Ok(Response::new(proto::UnpairDeviceResponse { removed }))
  }
}

fn proto_duration(value: &prost_types::Duration) -> Result<std::time::Duration, Status> {
  if value.seconds < 0 || value.nanos < 0 || value.nanos >= 1_000_000_000 {
    return Err(Status::invalid_argument("ttl must be a non-negative protobuf Duration"));
  }
  Ok(std::time::Duration::new(
    u64::try_from(value.seconds).expect("non-negative seconds fit u64"),
    u32::try_from(value.nanos).expect("validated nanos fit u32"),
  ))
}

fn pairing_status(error: crate::control::PairingError) -> Status {
  use crate::control::PairingError;
  match error {
    PairingError::InvalidToken => Status::unauthenticated(error.to_string()),
    PairingError::Invalid(_) => Status::invalid_argument(error.to_string()),
    PairingError::NotFound(_) => Status::not_found(error.to_string()),
    PairingError::Ambiguous(_) => Status::failed_precondition(error.to_string()),
    PairingError::Unauthenticated => Status::unauthenticated(error.to_string()),
    PairingError::NotConfigured => Status::unimplemented(error.to_string()),
    PairingError::Persistence(_) => Status::internal(error.to_string()),
  }
}
