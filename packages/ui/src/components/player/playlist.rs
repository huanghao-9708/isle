use crate::components::personal::AddToPlaylist;
use crate::state::{PersonalProvider, PlayerProvider};
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::FaHeart as FaHeartSolid;
use dioxus_free_icons::icons::ld_icons::*;
use dioxus_free_icons::Icon;

#[component]
pub fn PlaylistPanel() -> Element {
    let mut player = use_context::<PlayerProvider>();
    let personal = use_context::<PersonalProvider>();
    let is_visible = (player.is_playlist_visible)();
    let queue = player.queue.read();
    let current_index = (player.queue_index)();
    let liked_tracks = personal.liked_tracks();

    // 侧边栏及遮罩层样式
    let panel_class = if is_visible {
        "fixed top-0 right-0 h-full w-[400px] bg-white shadow-2xl z-[100] transform transition-transform duration-300 ease-out translate-x-0 flex flex-col pt-6 pb-24 border-l border-gray-100"
    } else {
        "fixed top-0 right-0 h-full w-[400px] bg-white shadow-2xl z-[100] transform transition-transform duration-300 ease-out translate-x-full flex flex-col pt-6 pb-24 border-l border-gray-100"
    };

    let overlay_class = if is_visible {
        "fixed inset-0 bg-black/5 backdrop-blur-[2px] z-[90] transition-opacity duration-300 opacity-100"
    } else {
        "fixed inset-0 bg-black/5 backdrop-blur-[2px] z-[90] transition-opacity duration-300 opacity-0 pointer-events-none"
    };

    rsx! {
        // 背景遮罩
        div {
            class: overlay_class,
            onclick: move |_| {
                player.is_playlist_visible.set(false);
            }
        }

        // 侧边面板
        div {
            class: panel_class,

            // 头部：显示总数及清空按钮
            div {
                class: "px-8 mb-6 flex items-center justify-between",
                div {
                    class: "space-y-1",
                    h2 { class: "text-2xl font-black tracking-tight", "当前播放" }
                    div {
                        class: "text-[10px] text-gray-400 font-bold uppercase tracking-widest",
                        "Queue ({queue.len()} tracks)"
                    }
                }

                button {
                    class: "flex items-center gap-2 text-xs font-bold text-gray-400 hover:text-black transition-colors px-3 py-1.5 rounded-full hover:bg-gray-100",
                    onclick: move |_| {
                        spawn(async move {
                            player.clear_queue().await;
                        });
                    },
                    Icon { width: 14, height: 14, icon: LdTrash2 }
                    span { "清空列表" }
                }
            }

            // 列表滚动区
            div {
                class: "flex-1 overflow-y-auto px-4 custom-scrollbar",

                if queue.is_empty() {
                    div {
                        class: "h-full flex flex-col items-center justify-center text-gray-300 gap-4",
                        Icon { width: 48, height: 48, icon: LdListMusic, class: "opacity-20" }
                        span { class: "text-xs font-bold uppercase tracking-widest", "队列是空的" }
                    }
                } else {
                    {queue.iter().enumerate().map(|(index, track)| {
                        let is_active = index == current_index;
                        let track_id = track.id.clone();
                        let is_liked = liked_tracks.iter().any(|t| t.id == track.id);

                        rsx! {
                            div {
                                key: "{track.id}-{index}",
                                class: format!(
                                    "group flex items-center gap-4 px-4 py-3 rounded-xl transition-all cursor-pointer hover:bg-gray-50 {}",
                                    if is_active { "bg-gray-50/50" } else { "" }
                                ),
                                onclick: move |_| {
                                    spawn(async move {
                                        player.play_index(index).await;
                                    });
                                },

                                // 封面展示
                                div {
                                    class: "relative w-12 h-12 flex-shrink-0 rounded-lg overflow-hidden bg-gray-100 flex items-center justify-center",
                                    if let Some(cover) = &track.cover {
                                        img { src: "http://covers.localhost/{cover}", class: "w-full h-full object-cover" }
                                    } else {
                                        Icon { width: 20, height: 20, icon: LdMusic, class: "text-gray-300" }
                                    }

                                    if is_active {
                                        div {
                                            class: "absolute inset-0 bg-black/20 flex items-center justify-center",
                                            div { class: "w-2 h-2 bg-white rounded-full animate-ping" }
                                        }
                                    }
                                }

                                // 歌曲与作者
                                div {
                                    class: "flex-1 min-w-0 flex flex-col gap-0.5",
                                    div {
                                        class: format!(
                                            "text-sm font-bold truncate {}",
                                            if is_active { "text-black" } else { "text-gray-700" }
                                        ),
                                        "{track.title}"
                                    }
                                    div {
                                        class: "text-[11px] text-gray-400 font-medium truncate",
                                        "{track.artist}"
                                    }
                                }

                                // 时长与删除按钮
                                div {
                                    class: "flex items-center gap-3",
                                    span {
                                        class: "text-[10px] font-mono text-gray-300",
                                        "{track.duration}"
                                    }

                                    button {
                                        class: if is_liked { "p-2 text-red-500 transition-all transform hover:scale-110" } else { "opacity-0 group-hover:opacity-100 p-2 text-gray-300 hover:text-red-500 transition-all transform hover:scale-110" },
                                        onclick: move |e| {
                                            e.stop_propagation();
                                        },
                                        if is_liked {
                                            Icon { width: 16, height: 16, icon: FaHeartSolid }
                                        } else {
                                            Icon { width: 16, height: 16, icon: LdHeart }
                                        }
                                    }

                                    AddToPlaylist { track_id: track_id.clone() }

                                    button {
                                        class: "opacity-0 group-hover:opacity-100 p-2 text-gray-300 hover:text-red-500 transition-all transform hover:scale-110",
                                        onclick: move |e| {
                                            e.stop_propagation();
                                            let id = track_id.clone();
                                            spawn(async move {
                                                player.remove_from_queue(id).await;
                                            });
                                        },
                                        Icon { width: 16, height: 16, icon: LdTrash2 }
                                    }
                                }
                            }
                        }
                    })}
                }
            }
        }
    }
}
