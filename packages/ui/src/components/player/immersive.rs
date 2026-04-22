use crate::components::personal::AddToPlaylist;
use crate::state::LibraryProvider;
use crate::state::PersonalProvider;
use crate::state::PlayerProvider;
use api::models::PlayMode;
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::FaHeart as FaHeartSolid;
use dioxus_free_icons::icons::ld_icons::*;
use dioxus_free_icons::Icon;

#[derive(Debug, Clone, PartialEq, Default)]
struct LyricLine {
    time: u32, // milliseconds
    text: String,
}

fn parse_lrc(lrc: &str) -> Vec<LyricLine> {
    let mut lines = Vec::new();
    for line in lrc.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if line.starts_with('[') {
            if let Some(end_idx) = line.find(']') {
                let time_str = &line[1..end_idx];
                let text = line[end_idx + 1..].trim().to_string();

                let parts: Vec<&str> = time_str.split(':').collect();
                if parts.len() == 2 {
                    let mins: u32 = parts[0].parse().unwrap_or(0);
                    let secs_parts: Vec<&str> = parts[1].split('.').collect();
                    let secs: u32 = secs_parts[0].parse().unwrap_or(0);
                    let ms: u32 = if secs_parts.len() > 1 {
                        let ms_str = secs_parts[1];
                        let val: u32 = ms_str.parse().unwrap_or(0);
                        if ms_str.len() == 2 {
                            val * 10
                        } else {
                            val
                        }
                    } else {
                        0
                    };

                    let total_ms = mins * 60 * 1000 + secs * 1000 + ms;
                    lines.push(LyricLine {
                        time: total_ms,
                        text,
                    });
                }
            }
        }
    }
    lines.sort_by_key(|l| l.time);
    lines
}

#[component]
pub fn ImmersivePlayer() -> Element {
    let mut player = use_context::<PlayerProvider>();
    let personal_ctx = use_context::<PersonalProvider>();
    let is_visible = (player.is_immersive_visible)();

    let current_track = player.current_track.read();
    let track = match current_track.as_ref() {
        Some(t) => t.clone(),
        None => return rsx! {},
    };

    let is_playing = player.is_playing();
    let progress_sec = player.progress();
    let duration_sec = player.duration();

    // 解决滑块回弹问题的辅助信号
    let mut dragging_progress = use_signal(|| None::<u32>);
    let display_progress = dragging_progress().unwrap_or(progress_sec);

    let lib = use_context::<LibraryProvider>();
    let track_id = track.id.clone();

    // 沉浸页专属：动态按需拉取数据库中的完整音轨信息以获取由于列表脱敏去除的歌词
    let lyrics_resource = use_resource(move || {
        let tid = track_id.clone();
        let l = lib.clone();
        async move {
            l.get_track_with_lyrics(tid)
                .await
                .and_then(|t| t.lyrics)
                .unwrap_or_default()
        }
    });

    let lyrics_memo = use_memo(move || {
        if let Some(lrc) = &*lyrics_resource.read() {
            if !lrc.is_empty() {
                return parse_lrc(lrc);
            }
        }
        if let Some(lrc) = &track.lyrics {
            return parse_lrc(lrc);
        }
        Vec::new()
    });

    // 当前播放进度（毫秒）
    let current_ms = player.progress() * 1000;

    // 计算当前高亮歌词行索引（供 render 使用）
    let lyrics_read = lyrics_memo.read();
    let active_index = lyrics_read
        .iter()
        .rposition(|line| line.time <= current_ms)
        .unwrap_or(0);
    drop(lyrics_read);

    // 自动滚动歌词：在 effect 内部读取信号以建立响应式依赖
    use_effect(move || {
        // 读取 player.progress() 信号，触发 effect 在进度变化时重新执行
        let ms = player.progress() * 1000;
        let vis = (player.is_immersive_visible)();
        if vis {
            let lr = lyrics_memo.read();
            let idx = lr.iter().rposition(|line| line.time <= ms).unwrap_or(0);
            drop(lr);

            let _ = dioxus::document::eval(&format!(
                "const area = document.getElementById('lyrics-scroll-area');
                 const line = document.getElementById('lyric-line-{}');
                 if (area && line) {{
                    const center = area.offsetHeight / 2;
                    const lineTop = line.offsetTop;
                    const lineHeight = line.offsetHeight;
                    area.scrollTo({{
                        top: lineTop - center + lineHeight / 2,
                        behavior: 'smooth'
                    }});
                 }}",
                idx
            ));
        }
    });

    let container_class = if is_visible {
        "fixed inset-0 z-[100] bg-white transition-transform duration-500 ease-out translate-y-0"
    } else {
        "fixed inset-0 z-[100] bg-white transition-transform duration-500 ease-in translate-y-full"
    };

    rsx! {
        style {
            {format!("
            .no-scrollbar::-webkit-scrollbar {{
                display: none;
            }}
            .no-scrollbar:hover::-webkit-scrollbar {{
                display: block;
                width: 6px;
            }}
            .no-scrollbar::-webkit-scrollbar-thumb {{
                background-color: #e5e7eb;
                border-radius: 3px;
            }}
            .lyrics-container {{
                mask-image: linear-gradient(to bottom, transparent, black 15%, black 85%, transparent);
            }}
            ")}
        }
        div {
            class: "{container_class} flex flex-col overflow-hidden",

            // Header
            div {
                class: "flex items-center justify-between px-10 py-6",
                button {
                    class: "w-10 h-10 flex items-center justify-center rounded-full hover:bg-gray-100 transition-colors",
                    onclick: move |_| { player.is_immersive_visible.set(false); },
                    Icon { width: 24, height: 24, icon: LdChevronDown }
                }
                div {
                    class: "flex flex-col items-center",
                    span { class: "text-[10px] font-bold text-gray-400 uppercase tracking-widest", "Now Playing" }
                    span { class: "text-xs font-bold text-black", "{track.title}" }
                }
                div { class: "w-10" }
            }

            // Main Content: 1:1 Split (flex-row)
            div {
                class: "flex-1 flex flex-row px-10 overflow-hidden",

                // Left: Album Cover
                div {
                    class: "w-1/2 flex justify-center items-center p-10",
                    div {
                        class: "w-full max-w-[420px] aspect-square rounded-2xl shadow-2xl overflow-hidden bg-gray-50 flex items-center justify-center",
                        {
                            if let Some(cover) = track.cover.clone() {
                                rsx! {
                                    img {
                                        src: "http://covers.localhost/{cover}",
                                        class: "w-full h-full object-cover",
                                        onerror: move |_| {
                                            tracing::error!("Immersive: cover load failed: {}", cover);
                                        }
                                    }
                                }
                            } else {
                                rsx! {
                                    div {
                                        class: "w-full h-full flex items-center justify-center text-gray-400",
                                        Icon { width: 120, height: 120, icon: LdMusic }
                                    }
                                }
                            }
                        }
                    }
                }

                // Right: Info & Lyrics
                div {
                    class: "w-1/2 h-full flex flex-col items-start justify-center pl-10 pr-20 pt-10",

                    div {
                        class: "mb-6 w-full text-left",
                        h1 { class: "text-2xl font-black text-black mb-1 leading-tight", "{track.title}" }
                        div {
                            class: "flex items-center gap-2 text-xs font-bold text-gray-400",
                            span { class: "hover:text-black cursor-pointer transition-colors text-left", "{track.artist}" }
                            span { "•" }
                            span { class: "hover:text-black cursor-pointer transition-colors text-left", "{track.album}" }
                        }
                    }

                    div {
                        class: "flex-1 w-full overflow-hidden relative lyrics-container",
                        div {
                            class: "h-full overflow-y-auto no-scrollbar scroll-smooth pt-[45%] pb-[50%]",
                            id: "lyrics-scroll-area",

                            {
                                let lyrics = lyrics_memo.read();
                                if lyrics.is_empty() {
                                    rsx! { div { class: "text-lg font-bold text-gray-300 py-4", "暂无歌词" } }
                                } else {
                                    rsx! {
                                        {lyrics.iter().enumerate().map(|(id, line)| {
                                            let is_active = id == active_index;
                                            let line = line.clone();
                                            rsx! {
                                                div {
                                                    key: "{id}",
                                                    id: "lyric-line-{id}",
                                                    class: format!(
                                                        "text-lg font-bold py-3 transition-all duration-500 cursor-pointer hover:text-black/80 text-left w-full {}",
                                                        if is_active { "text-black scale-105 origin-left" } else { "text-gray-300" }
                                                    ),
                                                    onclick: move |_| {
                                                        let time = line.time / 1000;
                                                        spawn(async move { player.seek(time).await; });
                                                    },
                                                    "{line.text}"
                                                }
                                            }
                                        })}
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Bottom Player Bar: Consistent with main UI (Horizontal Layout)
            div {
                class: "px-10 py-4 bg-white/90 backdrop-blur-md border-t border-gray-100 relative z-50",
                div {
                    class: "w-full flex items-center",

                    // Left Section: Track Info & Controls (No thumbnail)
                    div {
                        class: "flex-1 min-w-0 flex items-center gap-4",
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

                    // Center Section: Controls & Progress
                    div {
                        class: "flex-[2] flex flex-col items-center gap-2 px-8 max-w-2xl mx-auto",

                        div {
                            class: "flex items-center gap-8",

                            button {
                                class: "text-gray-400 hover:text-black transition-colors",
                                onclick: move |_| { spawn(async move { player.toggle_play_mode().await; }); },
                                match (player.play_mode)() {
                                    PlayMode::Shuffle => rsx! { Icon { width: 18, height: 18, icon: LdShuffle } },
                                    PlayMode::LoopOne => rsx! { Icon { width: 18, height: 18, icon: LdRepeat1 } },
                                    PlayMode::LoopAll => rsx! { Icon { width: 18, height: 18, icon: LdRepeat } },
                                    PlayMode::Sequence => rsx! { Icon { width: 18, height: 18, icon: LdArrowRight } },
                                }
                            }

                            button {
                                class: "text-black hover:scale-110 transition-transform",
                                onclick: move |_| { spawn(async move { player.prev().await; }); },
                                Icon { width: 24, height: 24, icon: LdSkipBack }
                            }

                            button {
                                class: "w-12 h-12 bg-black rounded-full flex items-center justify-center text-white hover:scale-105 active:scale-95 transition-all shadow-lg",
                                onclick: move |_| { spawn(async move { player.toggle_play().await; }); },
                                if is_playing {
                                    Icon { width: 20, height: 20, icon: LdPause }
                                } else {
                                    Icon { width: 20, height: 20, icon: LdPlay, class: "ml-1" }
                                }
                            }

                            button {
                                class: "text-black hover:scale-110 transition-transform",
                                onclick: move |_| { spawn(async move { player.next().await; }); },
                                Icon { width: 24, height: 24, icon: LdSkipForward }
                            }

                            button {
                                class: "text-gray-400 hover:text-black transition-colors",
                                onclick: move |_| {
                                    let current = (player.is_playlist_visible)();
                                    player.is_playlist_visible.set(!current);
                                },
                                Icon { width: 18, height: 18, icon: LdListMusic }
                            }
                        }

                        // Progress Bar
                        div {
                            class: "flex items-center gap-3 w-full group",
                            span { class: "text-[10px] text-gray-400 w-10 text-right font-mono", "{display_progress/60}:{(display_progress%60):02}" }
                            div {
                                class: "flex-grow h-1.5 relative flex items-center",
                                input {
                                    r#type: "range",
                                    class: "w-full h-1 bg-gray-100 rounded-full appearance-none cursor-pointer accent-black hover:h-1.5 transition-all",
                                    min: 0,
                                    max: if duration_sec > 0 { duration_sec as u32 } else { 100 },
                                    value: "{display_progress}",
                                    oninput: move |e| { if let Ok(v) = e.value().parse::<u32>() { dragging_progress.set(Some(v)); } },
                                    onchange: move |e| {
                                        if let Ok(v) = e.value().parse::<u32>() {
                                            spawn(async move {
                                                player.seek(v).await;
                                                dragging_progress.set(None);
                                            });
                                        } else { dragging_progress.set(None); }
                                    }
                                }
                            }
                            span { class: "text-[10px] text-gray-400 w-10 font-mono", "{duration_sec/60}:{(duration_sec%60):02}" }
                        }
                    }

                    // Right Section: Volume
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
}
