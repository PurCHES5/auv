use auv_driver::vision::TextRecognition;

use super::daily_recommended::DailyRecommendedView;
use super::recommended::RecommendedView;

/// App-owned classification of the current NetEase main content area.
#[derive(Clone, Debug, PartialEq)]
pub enum MainView {
  Recommended(RecommendedView),
  DailyRecommended(DailyRecommendedView),
  Unknown,
}

impl MainView {
  pub fn recommended(&self) -> Option<&RecommendedView> {
    match self {
      Self::Recommended(view) => Some(view),
      Self::DailyRecommended(_) | Self::Unknown => None,
    }
  }

  pub fn daily_recommended(&self) -> Option<&DailyRecommendedView> {
    match self {
      Self::DailyRecommended(view) => Some(view),
      Self::Recommended(_) | Self::Unknown => None,
    }
  }

  pub(crate) fn parse(recognition: &TextRecognition, window_size: auv_driver::Size) -> Self {
    if let Some(view) = DailyRecommendedView::parse(recognition) {
      return Self::DailyRecommended(view);
    }
    if let Some(view) = RecommendedView::parse(recognition, window_size) {
      return Self::Recommended(view);
    }
    // TODO(netease-main-views-v1): playlist, song-detail, and other main-view
    // variants are intentionally deferred until their existing command paths
    // are moved behind this app-owned view seam.
    Self::Unknown
  }
}
