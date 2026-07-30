//! osu! tracing artifact producers.

use auv_tracing::{ArtifactMetadata, ArtifactPurpose, Attributes, ByteLength, Context, JsonArtifactError, StoreError};
use serde::Serialize;

pub const OSU_STRUCTURED_ARTIFACT_JSON_BYTE_LIMIT: u64 = 4 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum OsuArtifactPublishError {
  #[error("osu! artifact {purpose} failed domain validation: {message}")]
  InvalidPayload {
    purpose: ArtifactPurpose,
    message: String,
  },
  #[error("failed to construct osu! artifact {purpose}: {source}")]
  Json {
    purpose: ArtifactPurpose,
    #[source]
    source: JsonArtifactError,
  },
  #[error("failed to publish osu! artifact {purpose}: {source}")]
  Publication {
    purpose: ArtifactPurpose,
    #[source]
    source: StoreError,
  },
}

pub(crate) async fn publish_json_artifact<T, V>(
  context: Option<&Context>,
  purpose: &'static str,
  value: &T,
  validate: V,
) -> Result<Option<ArtifactMetadata>, OsuArtifactPublishError>
where
  T: Serialize,
  V: FnOnce(&T) -> Result<(), String>,
{
  let Some(context) = context.filter(|context| context.can_publish_artifacts()) else {
    return Ok(None);
  };
  let purpose = ArtifactPurpose::new(purpose);
  validate(value).map_err(|message| OsuArtifactPublishError::InvalidPayload {
    purpose: purpose.clone(),
    message,
  })?;
  let emission = context
    .in_scope(|| {
      auv_tracing::emit_json_artifact(
        purpose.clone(),
        Attributes::empty(),
        ByteLength::new(OSU_STRUCTURED_ARTIFACT_JSON_BYTE_LIMIT).expect("static osu! JSON limit is valid"),
        value,
      )
    })
    .map_err(|source| OsuArtifactPublishError::Json {
      purpose: purpose.clone(),
      source,
    })?;
  emission.await.map_err(|source| OsuArtifactPublishError::Publication { purpose, source })
}

// TODO(auv-inspector): typed osu! artifact readers remain deferred until an
// owner-approved inspector contract defines discovery and validation inputs.
