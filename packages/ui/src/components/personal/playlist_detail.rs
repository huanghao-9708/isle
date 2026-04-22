use crate::components::music_library::TrackTable;
use crate::components::personal::PlaylistEditor;
use crate::state::{PersonalProvider, PlayerProvider};
use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::*;
use dioxus_free_icons::Icon;

#[component]
pub fn PlaylistDetail(playlist: api::models::UserPlaylist) -> Element {
    let personal = use_context::<PersonalProvider>();
    let mut player = use_context::<PlayerProvider>();

    // 编辑器状态
    let mut show_editor = use_signal(|| false);

    // 获取歌曲列表资源
    let tracks_res = use_resource({
        let id = playlist.id.clone();
        move || {
            let p = personal;
            let pid = id.clone();
            async move { p.get_playlist_tracks(&pid).await.unwrap_or_default() }
        }
    });

    rsx! {
        div { class: "w-full animate-in fade-in duration-500",
            // 编辑器弹窗
            if (show_editor)() {
                PlaylistEditor {
                    playlist: Some(playlist.clone()),
                    on_close: move |_| show_editor.set(false),
                    on_save: move |_| show_editor.set(false),
                }
            }

            div { class: "space-y-8 pb-20",
                // 详情 Header 卡片
                div {
                    class: "relative overflow-hidden rounded-[2.5rem] p-10 flex gap-10 items-center bg-white border border-gray-100 min-h-[320px] shadow-sm",

                    if let Some(cover) = &playlist.cover {
                        div { class: "absolute inset-0 z-0",
                            img {
                                class: "w-full h-full object-cover blur-[100px] opacity-10 scale-125",
                                src: "http://covers.localhost/{cover}"
                            }
                        }
                    }

                    div { class: "relative z-10 flex gap-10 items-center w-full",
                        div { class: "w-56 h-56 rounded-[2rem] bg-white shadow-2xl overflow-hidden flex-shrink-0 group hover:scale-105 transition-transform duration-500",
                            if let Some(cover) = &playlist.cover {
                                img { class: "w-full h-full object-cover", src: "http://covers.localhost/{cover}" }
                            } else {
                                div { class: "w-full h-full flex items-center justify-center bg-gray-50 text-gray-100",
                                    Icon { icon: LdListMusic, class: "w-24 h-24" }
                                }
                            }
                        }

                        div { class: "flex-1 space-y-6",
                            div { class: "space-y-2",
                                div { class: "text-xs font-black text-gray-400 uppercase tracking-[0.2em]", "Playlist" }
                                h1 { class: "text-6xl font-black text-black tracking-tighter", "{playlist.name}" }
                            }

                            div { class: "flex items-center gap-4 text-sm font-bold text-gray-500",
                                div { class: "flex items-center gap-2",
                                    div { class: "w-6 h-6 rounded-full bg-gray-200 flex items-center justify-center",
                                        Icon { icon: LdUser, class: "w-3 h-3 text-gray-400" }
                                    }
                                    "我的账户"
                                }
                                div { class: "w-1 h-1 bg-gray-300 rounded-full" }
                                span { "{playlist.track_count} 首歌曲" }
                            }

                            if !playlist.description.is_empty() {
                                p { class: "text-gray-500 text-sm max-w-2xl leading-relaxed", "{playlist.description}" }
                            }

                            div { class: "flex items-center gap-4 pt-4",
                                button {
                                    class: "bg-black text-white px-10 py-4 rounded-full text-sm font-black hover:scale-105 active:scale-95 transition-all flex items-center gap-3 shadow-xl shadow-black/10",
                                    onclick: move |_| {
                                        if let Some(tracks) = &*tracks_res.read() {
                                            let api_tracks: Vec<_> = tracks.iter().map(|t| t.to_api_track()).collect();
                                            spawn(async move {
                                                if !api_tracks.is_empty() {
                                                    player.set_queue_and_play(api_tracks).await;
                                                }
                                            });
                                        }
                                    },
                                    Icon { icon: LdPlay, class: "w-5 h-5 fill-white" }
                                    "播放全部"
                                }
                                button {
                                    class: "w-14 h-14 rounded-full bg-white border border-gray-100 flex items-center justify-center hover:bg-gray-50 hover:border-gray-200 transition-all text-gray-400 hover:text-black group relative",
                                    onclick: move |_| show_editor.set(true),
                                    Icon { icon: LdEllipsis, class: "w-6 h-6" }
                                }
                            }
                        }
                    }
                }

                // 歌曲列表区
                div { class: "mt-12",
                    if let Some(tracks) = &*tracks_res.read() {
                        if tracks.is_empty() {
                            div { class: "py-24 text-center space-y-4 bg-gray-50 rounded-[2.5rem] border-2 border-dashed border-gray-100",
                                Icon { icon: LdListMusic, class: "w-16 h-16 text-gray-200 mx-auto" }
                                p { class: "text-gray-400 font-bold", "歌单还是空的，去添加一些歌曲吧" }
                            }
                        } else {
                            {
                                let api_data: Vec<api::models::Track> = tracks.iter().map(|t| t.to_api_track()).collect();
                                let ui_data = tracks.clone();
                                let playlist_id = playlist.id.clone();
                                let playlist_name = playlist.name.clone();
                                let mut lib = use_context::<crate::state::LibraryProvider>();
                                rsx! {
                                    TrackTable {
                                        tracks: use_signal(|| ui_data),
                                        api_tracks: use_signal(|| api_data),
                                        display_limit: Some(20),
                                        on_view_all: move |_| {
                                            lib.navigate_to_all_tracks_detail(
                                                playlist_name.clone(),
                                                api::models::TrackFilter::default(),
                                                format!("playlist:{}", playlist_id)
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        div { class: "py-24 text-center",
                            div { class: "w-10 h-10 border-4 border-black/10 border-t-black rounded-full animate-spin mx-auto mb-4" }
                            p { class: "text-gray-400 font-bold", "正在加载歌曲列表..." }
                        }
                    }
                }
            }
        }
    }
}
