use auv_driver::vision::TextRecognition;

use crate::models::{DailyRecommendedRef, FeaturedEntry, FeaturedEntryKind};

/// One visible card in the leading collection on the Recommended view.
#[derive(Clone, Debug, PartialEq)]
pub struct FeaturedEntryView {
  entry: FeaturedEntry,
  /// Window-local bounds observed for this view generation.
  bounds: auv_driver::Rect,
}

impl FeaturedEntryView {
  pub fn entry(&self) -> &FeaturedEntry {
    &self.entry
  }

  pub fn bounds(&self) -> auv_driver::Rect {
    self.bounds
  }
}

/// The heterogeneous leading collection on the Recommended view.
///
/// It is intentionally not named a playlist collection: the live client mixes
/// Daily Recommended, radio-like features, modes, and other entry types here.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FeaturedEntriesView {
  visible_items: Vec<FeaturedEntryView>,
}

impl FeaturedEntriesView {
  pub fn visible_items(&self) -> &[FeaturedEntryView] {
    &self.visible_items
  }

  pub fn daily(&self) -> Option<&FeaturedEntryView> {
    self.visible_items.iter().find(|item| item.entry.kind() == FeaturedEntryKind::DailyRecommended)
  }
}

/// The NetEase Recommended main view.
#[derive(Clone, Debug, PartialEq)]
pub struct RecommendedView {
  featured_entries: FeaturedEntriesView,
}

impl RecommendedView {
  pub fn featured_entries(&self) -> &FeaturedEntriesView {
    &self.featured_entries
  }

  pub fn daily(&self) -> Option<DailyRecommendedRef> {
    self.featured_entries.daily()?.entry.daily_recommended_ref()
  }

  pub(crate) fn parse(recognition: &TextRecognition, window_size: auv_driver::Size) -> Option<Self> {
    let has_recommended_playlists = recognition.best_contains("推荐歌单").is_some();
    let visible_items = recognition
      .regions
      .iter()
      .filter(|region| {
        let bounds = region.bounds;
        bounds.origin.x >= window_size.width * 0.18 && bounds.origin.y <= window_size.height * 0.34
      })
      .filter_map(|region| {
        let title = region.text.trim();
        let kind = if title.contains("每日推荐") {
          FeaturedEntryKind::DailyRecommended
        } else if title.contains("私人雷达") {
          FeaturedEntryKind::PrivateRadar
        } else if title.contains("心动模式") {
          FeaturedEntryKind::HeartbeatMode
        } else if title.contains("私人漫游") {
          FeaturedEntryKind::PrivateRoaming
        } else if title.contains("相似歌曲") {
          FeaturedEntryKind::SimilarSongs
        } else if title.contains("电音日推") {
          FeaturedEntryKind::ElectronicDaily
        } else if title.to_lowercase().contains("lia") {
          FeaturedEntryKind::Lia
        } else {
          return None;
        };
        Some(FeaturedEntryView {
          entry: FeaturedEntry::from_observed_title(kind, title),
          bounds: region.bounds,
        })
      })
      .collect::<Vec<_>>();

    if !has_recommended_playlists || visible_items.is_empty() {
      return None;
    }

    Some(Self {
      featured_entries: FeaturedEntriesView { visible_items },
    })
  }
}

#[cfg(test)]
#[path = "recommended_test.rs"]
mod tests;
