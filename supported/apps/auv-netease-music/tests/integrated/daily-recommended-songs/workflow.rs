use auv_netease_music::app::{SongSourceProvider, ViewParts, ViewProvider, ViewScope};
use auv_netease_music::views::player::PlayerView;
use auv_netease_music::views::screen::ScreenView;
use auv_netease_music::views::sidebar::SidebarView;
use auv_netease_music::{MainView, NeteaseCloudMusic, SongListScanResult, SongSource};

struct RecordingProvider;

impl ViewProvider for RecordingProvider {
  fn read_views(&mut self, _scope: ViewScope) -> Result<ViewParts, String> {
    Ok(ViewParts {
      screen: ScreenView::unknown(),
      main: MainView::Unknown,
      sidebar: SidebarView::unknown(),
      player: PlayerView::unknown(),
    })
  }
}

impl SongSourceProvider for RecordingProvider {
  fn songs_from(&mut self, source: SongSource) -> Result<SongListScanResult, String> {
    Err(format!("called with {source:?}"))
  }
}

#[test]
fn daily_recommended_songs_selects_the_owned_song_source() {
  let mut app = NeteaseCloudMusic::from_provider(RecordingProvider);

  let error = app.daily_recommended().songs().expect_err("provider should report the selected source");

  assert!(error.contains("DailyRecommended"));
}
