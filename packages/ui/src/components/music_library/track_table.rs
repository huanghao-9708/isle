use crate::components::personal::AddToPlaylist;
use crate::components::types::Track;
use crate::state::{LibraryProvider, PersonalProvider, PlayerProvider};
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::FaHeart as FaHeartSolid;
use dioxus_free_icons::icons::ld_icons::*;
use dioxus_free_icons::Icon;

#[component]
pub fn TrackTable(
    tracks: ReadSignal<Vec<Track>>,
    api_tracks: ReadSignal<Vec<api::models::Track>>,
    /// 限制展示数量。如果为 Some，则只展示前 N 条。
    display_limit: Option<usize>,
    /// 点击“查看全部”时的回调
    on_view_all: Option<EventHandler<()>>,
    #[props(default)] show_played_at: bool,
) -> Element {
    let mut player = use_context::<PlayerProvider>();
    let personal = use_context::<PersonalProvider>();
    let library = use_context::<LibraryProvider>();
    let current_track = player.current_track.read();
    let liked_track_ids: Vec<String> = personal
        .liked_tracks()
        .iter()
        .map(|t| t.id.clone())
        .collect();

    rsx! {
        div {
            class: "w-full",

            // header
            div {
                class: "grid grid-cols-12 gap-4 px-6 py-4 text-[10px] uppercase tracking-widest text-gray-400 font-black border-b border-gray-100",

                div { class: "col-span-1", "#" }
                div { class: "col-span-1", "" } // cover
                div { class: if show_played_at { "col-span-3" } else { "col-span-4" }, "标题" }
                div { class: "col-span-2", "专辑" }
                div { class: "col-span-1", "时长" }
                if show_played_at {
                    div { class: "col-span-2 text-right", "播放时间" }
                } else {
                    div { class: "col-span-1 text-right", "大小" }
                }
                div { class: "col-span-2 text-right px-4", "操作" }
            }

            // tracks
            div {
                class: "w-full",
                {
                    let tracks_vec = tracks.read().clone();
                    let api_tracks_vec = std::sync::Arc::new(api_tracks.read().clone());
                    let end_idx = if let Some(limit) = display_limit {
                        tracks_vec.len().min(limit)
                    } else {
                        tracks_vec.len()
                    };

                    tracks_vec.into_iter().take(end_idx).enumerate().map(move |(index, track)| {
                        let is_active = current_track.as_ref().map(|ct| ct.id == track.id).unwrap_or(false);
                        let api_tracks_for_closure = api_tracks_vec.clone();

                        rsx! {
                            div {
                                key: "{track.id}",
                                class: format!(
                                    "grid grid-cols-12 gap-4 px-6 py-4 transition-all duration-300 cursor-pointer group rounded-2xl mx-1 my-1 {}",
                                    if is_active { "bg-black/5" } else { "hover:bg-gray-50" }
                                ),
                                ondoubleclick: move |_| {
                                    let api_tracks = (*api_tracks_for_closure).clone();
                                    spawn(async move {
                                        player.play_from_list(api_tracks, index).await;
                                    });
                                },

                                // Index/Status
                                div {
                                    class: "col-span-1 flex items-center text-sm font-black",
                                    if is_active && player.is_playing() {
                                        div { class: "flex items-end gap-0.5 h-3 items-center",
                                            for i in 0..3 {
                                                div {
                                                    class: "w-0.5 bg-black rounded-full animate-bounce",
                                                    style: "animation-delay: {i * 150}ms; height: {4 + (i * 2)}px"
                                                }
                                            }
                                        }
                                    } else {
                                        span { class: "text-gray-200 group-hover:text-black transition-colors", "{index + 1:02}" }
                                    }
                                }

                                // Cover
                                div {
                                    class: "col-span-1 flex items-center justify-center",
                                    div {
                                        class: "w-10 h-10 bg-gray-50 rounded-lg flex items-center justify-center text-gray-400 transition-shadow overflow-hidden border border-gray-100",
                                        if let Some(cover_filename) = &track.cover {
                                            img { src: "http://covers.localhost/{cover_filename}", class: "w-full h-full object-cover" }
                                        } else {
                                            Icon { width: 16, height: 16, icon: LdMusic }
                                        }
                                    }
                                }

                                // Title & Artist
                                div {
                                    class: format!("{} flex flex-col justify-center overflow-hidden", if show_played_at { "col-span-3" } else { "col-span-4" }),
                                    div {
                                        class: format!("text-sm font-black truncate {}", if is_active { "text-black" } else { "text-gray-900" }),
                                        "{track.title}"
                                    }
                                    div {
                                        class: "text-[10px] text-gray-400 truncate uppercase tracking-tight font-bold mr-2 hover:text-black hover:underline cursor-pointer transition-all",
                                        onclick: {
                                            let artist_id = track.artist_id.clone();
                                            move |e| {
                                                e.stop_propagation();
                                                let mut lib = library;
                                                let aid = artist_id.clone();
                                                spawn(async move {
                                                    lib.navigate_to_artist(aid).await;
                                                });
                                            }
                                        },
                                        "{track.artist}"
                                    }
                                }

                                div {
                                    class: "col-span-2 flex items-center text-xs text-gray-400 truncate font-bold hover:text-black hover:underline cursor-pointer transition-all",
                                    onclick: {
                                        let album_id = track.album_id.clone();
                                        move |e| {
                                            e.stop_propagation();
                                            let mut lib = library;
                                            let aid = album_id.clone();
                                            spawn(async move {
                                                lib.navigate_to_album(aid).await;
                                            });
                                        }
                                    },
                                    "{track.album}"
                                }

                                // Duration
                                div {
                                    class: "col-span-1 flex items-center text-xs text-gray-400 font-bold",
                                    "{track.duration}"
                                }

                                // Size or Played At
                                if show_played_at {
                                    div {
                                        class: "col-span-2 flex items-center justify-end text-xs text-gray-400 font-bold tabular-nums",
                                        "{track.played_at.clone().unwrap_or_else(|| \"未知\".to_string())}"
                                    }
                                } else {
                                    div {
                                        class: "col-span-1 flex items-center justify-end text-xs text-gray-400 font-bold tabular-nums",
                                        "{track.size}"
                                    }
                                }

                                // Actions
                                div {
                                    class: "col-span-2 flex items-center justify-end gap-1 px-2 opacity-0 group-hover:opacity-100 transition-opacity",
                                    {
                                        let is_liked = liked_track_ids.contains(&track.id);
                                        let track_id = track.id.clone();
                                        let p_provider = personal.clone();
                                        rsx! {
                                            button {
                                                class: if is_liked { "p-2 text-red-500 transition-colors" } else { "p-2 text-gray-300 hover:text-red-500 transition-colors" },
                                                title: if is_liked { "取消喜欢" } else { "喜欢" },
                                                onclick: move |e| {
                                                    e.stop_propagation();
                                                    let tid = track_id.clone();
                                                    let mut p = p_provider.clone();
                                                    spawn(async move {
                                                        if is_liked {
                                                            let _ = p.unlike_track(&tid).await;
                                                        } else {
                                                            let _ = p.like_track(&tid).await;
                                                        }
                                                    });
                                                },
                                                if is_liked {
                                                    Icon { width: 16, height: 16, icon: FaHeartSolid }
                                                } else {
                                                    Icon { width: 16, height: 16, icon: LdHeart }
                                                }
                                            }
                                        }
                                    }
                                    AddToPlaylist { track_id: track.id.clone() }
                                }
                            }
                        }
                    })
                }
            }

            // 查看全部按钮
            if let Some(limit) = display_limit {
                if tracks.read().len() > limit {
                    div {
                        class: "h-20 flex items-center justify-end px-10",
                        button {
                            class: "group flex items-center gap-2 px-6 py-2.5 bg-black text-white rounded-full hover:bg-gray-800 transition-all active:scale-95 shadow-lg shadow-black/5 hover:shadow-black/10",
                            onclick: move |_| {
                                if let Some(handler) = on_view_all {
                                    handler.call(());
                                }
                            },
                            span { class: "text-sm font-black tracking-tight", "查看全部 {tracks.read().len()} 首歌曲" }
                            Icon {
                                width: 16,
                                height: 16,
                                icon: LdArrowRight,
                                class: "transition-transform group-hover:translate-x-1"
                            }
                        }
                    }
                }
            }
        }
    }
}
