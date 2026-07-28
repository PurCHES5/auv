# NetEase Cloud Music App-owned Views and Operations

Date: 2026-06-03
Updated: 2026-07-27

Status: partially implemented. The Recommended and Daily Recommended slice is
implemented; playlist and song-detail migration remains deferred.

## Purpose

`auv-netease-music` has app-specific knowledge that cannot be expressed well as
a flat list of commands. The desktop client contains a sidebar, main content,
and bottom player. The main content changes between Recommended, playlist,
Daily Recommended, and song-detail views. Lists also contain visible row items
that are useful for interaction but are not durable domain identity.

The app crate therefore owns two related interfaces:

- `models/` contains app-domain values and semantic references.
- `views/` describes the currently readable GUI hierarchy and its visible
  items.
- `NeteaseCloudMusic` owns IO, navigation, cache invalidation, and operations
  that may scan beyond the current viewport.
- `commands/` remains the frontend adapter and calls the app-owned operation
  surface.

This is not a requirement for every app to expose the same generic tree.
TextEdit can be primarily document-shaped, while Notes needs note and folder
models backed by several views.

## Live Evidence

On 2026-07-27, a live capture of NetEase Music at 1645×957 showed this
Recommended-page structure:

```text
main: Recommended
  leading horizontal collection
    每日推荐
    私人雷达
    心动模式
    私人漫游
    相似歌曲
    电音日推
    听·Lia
  推荐歌单
    playlist cards ...
```

The leading collection is heterogeneous. It is named `FeaturedEntriesView`,
not `FeaturedPlaylistsView`, because several entries are modes or generated
features rather than playlists.

Selecting `每日推荐` opens a different main view containing `播放全部` and a
scrollable song list. A live `playlist songs ls` run after the implementation
change read songs 1–17 across two visible pages. The previous scan returned an
empty list because its crop started at 30% of window width (x=493.5), to the
right of the row index and title anchors. The current crop starts at 20%
(x=329), and the first-row boundary is inclusive.

## Public Shape

Callers can inspect the current hierarchy:

```rust
let daily_entry = app
  .views(ViewRead::fresh())?
  .recommended()
  .and_then(|view| view.featured_entries().daily());
```

Callers that want the deep operation do not need to manually replay every
navigation step:

```rust
let songs = app.daily_recommended().songs()?;
```

The shorter form does not flatten the model. Its live implementation navigates
through Recommended, locates the Daily Recommended entry again, opens the
detail view, and scans the song rows. The same operation is also available as
`app.songs_from(DailyRecommendedRef)` when the source is already known.

`DailyRecommendedRef` intentionally contains no coordinates or OCR candidate
identifier. GUI positions belong to one `AppViews` generation; an operation
must locate the entry again before delivering input.

## View Read Semantics

`AppViews` contains app-owned read views for the screen, main content, sidebar,
and player. `ViewScope` controls which expensive areas are read. `ViewRead`
chooses a fresh read or reuse of a short-lived cache.

Mutating operations invalidate the cache before and after delivery. A caller
must not treat a pre-action `AppViews` value as the current GUI after
navigation, scrolling, or playback actions.

The current live provider still reads the sidebar separately because its
existing scan has scroll and reconstruction behavior that a single screenshot
does not replace. Main, screen, and player classification share one window
capture where their requested scopes overlap.

## Boundary With `auv-view`

`auv-view` supplies generic geometry, reconstruction, and scroll vocabulary.
NetEase-specific concepts remain in this crate:

- `RecommendedView`, `FeaturedEntriesView`, and `DailyRecommendedView`
- playlist sidebar sections and rows
- `DailyRecommendedRef` and `SongSource`
- navigation rules and list scanning

The current evidence does not justify a workspace-wide generic `View` trait for
all applications. Shared primitives should move to `auv-view` only after a
second app needs the same meaning and lifecycle.

## Current Deferrals

- `MainView` does not yet model ordinary playlist and song-detail variants.
  Their command paths should move behind `NeteaseCloudMusic` before those
  variants become public contracts.
- `SongSource` does not yet include ordinary playlists for the same reason.
- The sidebar scan does not yet consume the same capture generation as the
  main and player views; preserving its existing scroll semantics takes
  priority in this slice.
- Existing scan result records remain in place. This design does not introduce
  parallel confidence, artifact, output, or observation contracts; run tracing
  remains owned by `auv-tracing` and the shared runtime.
