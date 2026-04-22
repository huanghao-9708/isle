use crate::components::music_library::TrackTable;
use crate::components::personal::AddToPlaylist;
use crate::state::{LibraryProvider, PersonalProvider, PlayerProvider};
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::FaHeart as FaHeartSolid;
use dioxus_free_icons::icons::ld_icons::*;
use dioxus_free_icons::Icon;

#[component]
pub fn LikedTracksList() -> Element {
    let provider = use_context::<PersonalProvider>();
    let mut player = use_context::<PlayerProvider>();
    let mut library = use_context::<LibraryProvider>();
    let liked_tracks = provider.liked_tracks();

    rsx! {
        div {
            div { class: "flex items-center justify-between mb-6",
                div { class: "flex items-center gap-4",
                    h2 { class: "text-2xl font-bold", "我喜欢的歌曲" }
                    span { class: "text-gray-500 text-sm", "{liked_tracks.len()} 首歌曲" }
                }
                button {
                    class: "bg-black text-white px-4 py-2 rounded-full text-sm font-bold hover:bg-gray-800 transition-colors",
                    onclick: move |_| {
                        let tracks = provider.liked_tracks();
                        if !tracks.is_empty() {
                            let api_tracks: Vec<_> = tracks.iter().map(|t| t.to_api_track()).collect();
                            spawn(async move {
                                player.set_queue_and_play(api_tracks).await;
                            });
                        }
                    },
                    Icon { icon: LdPlay, class: "w-4 h-4 mr-1 inline" }
                    "播放全部"
                }
            }

            if liked_tracks.is_empty() {
                div { class: "text-gray-400 py-12 text-center", "暂无收藏的歌曲" }
            } else {
                div { class: "bg-white rounded-3xl overflow-hidden border border-gray-100",
                    {
                        let api_data: Vec<api::models::Track> = liked_tracks.iter().map(|t| t.to_api_track()).collect();
                        let ui_data: Vec<crate::components::types::Track> = liked_tracks.clone();
                        rsx! {
                            TrackTable {
                                tracks: use_signal(|| ui_data),
                                api_tracks: use_signal(|| api_data),
                                display_limit: Some(20),
                                on_view_all: move |_| {
                                    library.navigate_to_all_tracks_detail(
                                        "我喜欢的歌曲".to_string(),
                                        api::models::TrackFilter::default(),
                                        "liked".to_string()
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
