use auv_driver::vision::TextRecognition;

/// The main view shown after opening the Daily Recommended feature entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DailyRecommendedView {
  title: String,
}

impl DailyRecommendedView {
  pub fn title(&self) -> &str {
    &self.title
  }

  pub(crate) fn parse(recognition: &TextRecognition) -> Option<Self> {
    if recognition.best_contains("播放全部").is_none() {
      return None;
    }
    let title = recognition.regions.iter().map(|region| region.text.trim()).find(|text| text.contains("每日推荐"))?;
    Some(Self {
      title: title.to_string(),
    })
  }
}

#[cfg(test)]
#[path = "daily_recommended_test.rs"]
mod tests;
