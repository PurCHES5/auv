use std::num::NonZeroUsize;
use std::path::PathBuf;

use auv_driver::RatioRect;
use auv_driver::vision::TextRecognitionOptions;
use clap::Args;

#[derive(Clone, Debug, Default, Args)]
pub(super) struct AppTargetArgs {
  /// Application identifier to target instead of the NetEase default.
  #[arg(long = "app-id")]
  pub app_id: Option<String>,
}

#[derive(Clone, Debug, Default, Args)]
pub(super) struct ScrollArgs {
  /// Maximum number of scroll steps before stopping the scan.
  #[arg(long = "max-scrolls")]
  pub max_scrolls: Option<NonZeroUsize>,
  /// Vertical distance requested for each scroll step.
  #[arg(long = "scroll-amount", value_parser = positive_scroll_amount)]
  pub scroll_amount: Option<f64>,
  /// Delay in milliseconds after each scroll before observing the UI again.
  #[arg(long = "scroll-settle-ms")]
  pub scroll_settle_ms: Option<u64>,
}

#[derive(Clone, Debug, Default, Args)]
pub(super) struct OcrHintArgs {
  /// Add one expected word to OCR recognition; may be repeated.
  #[arg(long = "hint-ocr-custom-word")]
  custom_words: Vec<String>,
  /// Add comma-separated expected OCR words; may be repeated.
  #[arg(long = "hint-ocr-custom-words")]
  custom_word_csvs: Vec<String>,
  /// Load expected OCR words from a UTF-8 file, one word per line.
  #[arg(long = "hint-ocr-custom-words-file")]
  custom_word_files: Vec<PathBuf>,
  /// Add one OCR recognition language tag; may be repeated.
  #[arg(long = "hint-ocr-language")]
  ocr_languages: Vec<String>,
  /// Add comma-separated OCR recognition language tags; may be repeated.
  #[arg(long = "hint-ocr-languages")]
  ocr_language_csvs: Vec<String>,
}

impl OcrHintArgs {
  /// Apply every CLI spelling of OCR hints to the driver's typed options.
  pub fn apply(self, options: &mut TextRecognitionOptions) -> Result<(), String> {
    for word in self.custom_words {
      push_trimmed(&mut options.custom_words, word);
    }
    for csv in self.custom_word_csvs {
      push_csv(&mut options.custom_words, &csv);
    }
    for path in self.custom_word_files {
      load_custom_words_file(&mut options.custom_words, path)?;
    }
    for language in self.ocr_languages {
      push_ocr_language(options, language);
    }
    for csv in self.ocr_language_csvs {
      for language in split_csv(&csv) {
        push_ocr_language(options, language);
      }
    }
    Ok(())
  }
}

pub(super) fn positive_scroll_amount(raw: &str) -> Result<f64, String> {
  let parsed = raw.parse::<f64>().map_err(|_| "expects a number".to_string())?;
  if !parsed.is_finite() || parsed <= 0.0 {
    return Err("must be greater than 0".to_string());
  }
  Ok(parsed)
}

pub(super) fn zero_to_one(raw: &str) -> Result<f64, String> {
  let parsed = raw.parse::<f64>().map_err(|_| "expects a number".to_string())?;
  if !parsed.is_finite() || !(0.0..=1.0).contains(&parsed) {
    return Err("must be between 0 and 1".to_string());
  }
  Ok(parsed)
}

pub(super) fn parse_ratio_region(value: String) -> Result<RatioRect, String> {
  let parts = value
    .split(',')
    .map(str::trim)
    .map(|part| part.parse::<f64>().map_err(|_| "--sidebar-region expects x,y,width,height".to_string()))
    .collect::<Result<Vec<_>, _>>()?;

  if parts.len() != 4 {
    return Err("--sidebar-region expects x,y,width,height".to_string());
  }
  if parts.iter().any(|part| !part.is_finite()) {
    return Err("--sidebar-region expects finite x,y,width,height".to_string());
  }
  if parts[2] <= 0.0 || parts[3] <= 0.0 {
    return Err("--sidebar-region width and height must be greater than 0".to_string());
  }
  Ok(RatioRect::new(parts[0], parts[1], parts[2], parts[3]))
}

fn split_csv(value: &str) -> impl Iterator<Item = String> + '_ {
  value.split(',').map(str::trim).filter(|part| !part.is_empty()).map(ToOwned::to_owned)
}

fn push_trimmed(values: &mut Vec<String>, value: String) {
  let value = value.trim();
  if !value.is_empty() && !values.iter().any(|existing| existing == value) {
    values.push(value.to_string());
  }
}

fn push_csv(values: &mut Vec<String>, value: &str) {
  for part in split_csv(value) {
    push_trimmed(values, part);
  }
}

fn push_ocr_language(options: &mut TextRecognitionOptions, language: String) {
  let language = language.trim();
  if language.is_empty() {
    return;
  }
  let languages = options.recognition_languages.get_or_insert_with(Vec::new);
  if !languages.iter().any(|existing| existing == language) {
    languages.push(language.to_string());
  }
}

fn load_custom_words_file(values: &mut Vec<String>, path: PathBuf) -> Result<(), String> {
  let content = std::fs::read_to_string(&path).map_err(|error| format!("failed to read {}: {error}", path.display()))?;
  for line in content.lines() {
    let word = line.trim();
    if !word.is_empty() && !word.starts_with('#') {
      push_trimmed(values, word.to_string());
    }
  }
  Ok(())
}
