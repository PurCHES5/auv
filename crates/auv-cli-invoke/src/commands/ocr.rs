use crate::{InvokeReport, InvokeReportField, InvokeReportTable, InvokeReportValue};
use auv_cli_common::{
  TableRow,
  outputs::formats::table::{TableOptions, TableRow as TableRowSchema},
};

#[derive(TableRow)]
struct MatchRow {
  #[table(header = "IDX")]
  index: usize,
  text: String,
  point: String,
  bounds: String,
  #[table(wide, header = "CONF", display_with = |confidence: &f64| format!("{confidence:.3}"))]
  confidence: f64,
}

#[derive(TableRow)]
struct SelectedMatchRow {
  #[table(header = "SEL")]
  selected: &'static str,
  #[table(header = "IDX")]
  index: usize,
  text: String,
  point: String,
  bounds: String,
  #[table(wide, header = "CONF", display_with = |confidence: &f64| format!("{confidence:.3}"))]
  confidence: f64,
}

pub(super) fn match_report(matches: &[auv_driver::OcrMatch], selected_index: Option<usize>) -> InvokeReport {
  if let Some(selected_index) = selected_index {
    let rows = matches
      .iter()
      .enumerate()
      .map(|(index, matched)| SelectedMatchRow {
        selected: if index == selected_index { "*" } else { "" },
        index,
        text: matched.text.clone(),
        point: matched.action_point().report_value(),
        bounds: matched.bounds.report_value(),
        confidence: matched.confidence,
      })
      .collect::<Vec<_>>();
    report_from_rows(matches.len(), &rows, vec![None, None, Some(48), None, None])
  } else {
    let rows = matches
      .iter()
      .enumerate()
      .map(|(index, matched)| MatchRow {
        index,
        text: matched.text.clone(),
        point: matched.action_point().report_value(),
        bounds: matched.bounds.report_value(),
        confidence: matched.confidence,
      })
      .collect::<Vec<_>>();
    report_from_rows(matches.len(), &rows, vec![None, Some(48), None, None])
  }
}

fn report_from_rows<R>(match_count: usize, rows: &[R], display_max_chars: Vec<Option<usize>>) -> InvokeReport
where
  R: TableRowSchema,
{
  let mut wide_display_max_chars = display_max_chars.clone();
  wide_display_max_chars.push(None);
  InvokeReport {
    fields: vec![InvokeReportField::new(
      "Result",
      format!("{} text match(es)", match_count),
    )],
    tables: vec![InvokeReportTable::from_rows(rows, TableOptions::default()).with_display_max_chars(display_max_chars)],
    wide_tables: vec![InvokeReportTable::from_rows(rows, TableOptions::default().wide(true)).with_display_max_chars(wide_display_max_chars)],
    sections: Vec::new(),
  }
}

#[cfg(test)]
#[path = "ocr_test.rs"]
mod tests;
