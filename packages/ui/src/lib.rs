//! This crate contains all shared UI for the workspace.

use dioxus::prelude::*;

pub const STYLES_CSS: Asset = asset!("/assets/styles.css");

mod hero;
pub use hero::Hero;

pub mod components;
pub mod state;

// 针对常用的 Provider 或组件进行精确导出，避免全局 glob 冲突
pub use components::app_root::AppRoot;
pub use components::music_library_page::MusicLibraryPage;
pub use components::personal_space_page::PersonalSpacePage;
pub use state::library::LibraryProvider;
pub use state::personal::PersonalProvider;
pub use state::player::PlayerProvider;
