use crate::components::personal::AddToPlaylist;
use crate::state::PersonalProvider;
use crate::state::PlayerProvider;
use api::models::PlayMode;
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::FaHeart as FaHeartSolid;
use dioxus_free_icons::icons::ld_icons::*;
use dioxus_free_icons::Icon;

#[component]
pub fn PlayerBar() -> Element {
    let mut player = use_context::<PlayerProvider>();
    let personal_ctx = use_context::<PersonalProvider>();

    let current_track_opt = player.current_track.read();
    let track = match current_track_opt.as_ref() {
        Some(t) => t.clone(),
        None => crate::components::types::Track {
            id: "".to_string(),
            path: "".to_string(),
            title: "未在播放".to_string(),
            artist: "请选择歌曲".to_string(),
            artist_id: "".to_string(),
            album: "".to_string(),
            album_id: "".to_string(),
            duration: "0:00".to_string(),
            size: "".to_string(),
            genres: Vec::new(),
            cover: None,
            lyrics: None,
            played_at: None,
        },
    };

    let is_playing = player.is_playing();
    let progress_sec = player.progress();
    let duration_sec = player.duration();

    // 解决滑块回弹问题的辅助信号
    let mut dragging_progress = use_signal(|| None::<u32>);
    let display_progress = dragging_progress().unwrap_or(progress_sec);

    // 播放历史自动记录：监听当前音轨变化
    use_effect(move || {
        if let Some(track) = player.current_track.read().clone() {
            if !track.id.is_empty() {
                let p = personal_ctx;
                let tid = track.id.clone();
                spawn(async move {
                    let mut p = p;
                    let _ = p.add_play_history(&tid).await;
                });
            }
        }
    });

    rsx! {
        div {
            class: "bg-[#FAFAFA] border-t border-gray-200/40 px-10 py-4 relative z-50",
            div {
                class: "w-full flex items-center",

                // Left Section - Track Info
                div {
                    class: "flex-1 min-w-0 flex items-center gap-4 cursor-pointer hover:bg-gray-100/60 transition-colors rounded-xl p-1 -ml-1",
                    onclick: move |_| {
                        player.is_immersive_visible.set(true);
                    },

                    div {
                        class: "w-14 h-14 bg-gray-100 rounded-lg flex-shrink-0 flex items-center justify-center text-gray-400 overflow-hidden shadow-sm",
                        if let Some(cover) = &track.cover {
                            img { src: "http://covers.localhost/{cover}", class: "w-full h-full object-cover" }
                        } else {
                            Icon { width: 24, height: 24, icon: LdMusic }
                        }
                    }

                    div {
                        class: "flex flex-col gap-0.5 overflow-hidden",
                        div { class: "font-bold text-black text-sm truncate", "{track.title}" }
                        div { class: "text-xs text-gray-400 font-medium truncate", "{track.artist}" }
                    }

                    div { class: "flex items-center gap-1 ml-4",
                        {
                            let liked_tracks = personal_ctx.liked_tracks();
                            let is_liked = liked_tracks.iter().any(|t| t.id == track.id);
                            let track_id = track.id.clone();
                            let mut p = personal_ctx;

                            rsx! {
                                button {
                                    class: if is_liked { "text-red-500 transition-colors" } else { "text-gray-300 hover:text-black transition-colors" },
                                    onclick: move |_| {
                                        let tid = track_id.clone();
                                        spawn(async move {
                                            if is_liked {
                                                let _ = p.unlike_track(&tid).await;
                                            } else {
                                                let _ = p.like_track(&tid).await;
                                            }
                                        });
                                    },
                                    if is_liked {
                                        Icon { width: 18, height: 18, icon: FaHeartSolid }
                                    } else {
                                        Icon { width: 18, height: 18, icon: LdHeart }
                                    }
                                }
                            }
                        }
                        if !track.id.is_empty() {
                            AddToPlaylist { track_id: track.id.clone() }
                        }
                    }
                }

                // Center Section - Controls & Progress
                div {
                    class: "flex-[2] flex flex-col items-center gap-2 px-8 max-w-2xl mx-auto",

                    div {
                        class: "flex items-center gap-8",

                        button {
                            class: "text-gray-400 hover:text-black transition-colors",
                            onclick: move |_| {
                                spawn(async move { player.toggle_play_mode().await; });
                            },
                            match (player.play_mode)() {
                                PlayMode::Shuffle => rsx! { Icon { width: 18, height: 18, icon: LdShuffle } },
                                PlayMode::LoopOne => rsx! { Icon { width: 18, height: 18, icon: LdRepeat1 } },
                                PlayMode::LoopAll => rsx! { Icon { width: 18, height: 18, icon: LdRepeat } },
                                PlayMode::Sequence => rsx! { Icon { width: 18, height: 18, icon: LdArrowRight } },
                            }
                        }

                        button {
                            class: "text-black hover:scale-110 transition-transform",
                            onclick: move |_| {
                                spawn(async move { player.prev().await; });
                            },
                            Icon { width: 24, height: 24, icon: LdSkipBack }
                        }

                        button {
                            class: "w-12 h-12 bg-black rounded-full flex items-center justify-center text-white hover:scale-105 active:scale-95 transition-all shadow-lg",
                            onclick: move |_| {
                                spawn(async move { player.toggle_play().await; });
                            },
                            if is_playing {
                                Icon { width: 20, height: 20, icon: LdPause }
                            } else {
                                Icon { width: 20, height: 20, icon: LdPlay, class: "ml-1" }
                            }
                        }

                        button {
                            class: "text-black hover:scale-110 transition-transform",
                            onclick: move |_| {
                                spawn(async move { player.next().await; });
                            },
                            Icon { width: 24, height: 24, icon: LdSkipForward }
                        }

                        button {
                            class: format!("transition-colors {}", if (player.is_playlist_visible)() { "text-black" } else { "text-gray-400 hover:text-black" }),
                            onclick: move |_| {
                                let current = (player.is_playlist_visible)();
                                player.is_playlist_visible.set(!current);
                            },
                            Icon { width: 18, height: 18, icon: LdListMusic }
                        }
                    }

                    div {
                        class: "flex items-center gap-3 w-full group",
                        span { class: "text-[10px] text-gray-400 w-10 text-right font-mono", "{display_progress/60}:{(display_progress%60):02}" }
                        div {
                            class: "flex-grow h-1.5 relative flex items-center",
                            input {
                                r#type: "range",
                                class: "w-full h-1 bg-gray-100 rounded-full appearance-none cursor-pointer accent-black hover:h-1.5 transition-all opacity-80 hover:opacity-100",
                                min: 0,
                                max: if duration_sec > 0 { duration_sec as u32 } else { 100 },
                                value: "{display_progress}",
                                oninput: move |e| {
                                    if let Ok(v) = e.value().parse::<u32>() {
                                        dragging_progress.set(Some(v));
                                    }
                                },
                                onchange: move |e| {
                                    if let Ok(v) = e.value().parse::<u32>() {
                                        spawn(async move {
                                            player.seek(v).await;
                                            dragging_progress.set(None);
                                        });
                                    } else {
                                        dragging_progress.set(None);
                                    }
                                }
                            }
                        }
                        span { class: "text-[10px] text-gray-400 w-10 font-mono", "{duration_sec/60}:{(duration_sec%60):02}" }
                    }
                }

                // Right Section - Volume
                div {
                    class: "flex-1 flex items-center justify-end gap-6",
                    div {
                        class: "flex items-center gap-3 w-full max-w-[140px]",
                        button {
                            class: "text-gray-400 hover:text-black transition-colors",
                            if (player.volume)() == 0 {
                                Icon { width: 18, height: 18, icon: LdVolumeX }
                            } else {
                                Icon { width: 18, height: 18, icon: LdVolume2 }
                            }
                        }
                        input {
                            r#type: "range",
                            class: "flex-grow h-1 bg-gray-100 rounded-full appearance-none cursor-pointer accent-black",
                            min: 0,
                            max: 100,
                            value: "{(player.volume)()}",
                            oninput: move |e| {
                                if let Ok(v) = e.value().parse::<u8>() {
                                    spawn(async move { player.set_volume(v).await; });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
