use crate::state::{LibraryProvider, PersonalProvider, PlayerProvider};
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::FaHeart;
use dioxus_free_icons::icons::ld_icons::*;
use dioxus_free_icons::Icon;

#[component]
pub fn PersonalDashboard() -> Element {
    let mut library_provider = use_context::<LibraryProvider>();
    let provider = use_context::<PersonalProvider>();
    let mut player = use_context::<PlayerProvider>();

    let liked_tracks = provider.liked_tracks();
    let playlists = provider.playlists();
    let liked_albums = provider.liked_albums();
    let liked_artists = provider.liked_artists();
    let recently_played = provider.recently_played();

    // 优化点 1: 提取第一首歌的封面作为动态背景
    let first_track_cover = liked_tracks.first().and_then(|t| t.cover.clone());

    rsx! {
        div { class: "space-y-12 animate-in fade-in slide-in-from-bottom-4 duration-700 pb-20",

            // --- Hero Section: 我喜欢的音乐 ---
            div {
                class: "relative group overflow-hidden rounded-[2.5rem] bg-[#0a0b0d] p-10 shadow-2xl flex items-center gap-10 hover:shadow-black/20 transition-all duration-500 cursor-pointer",
                onclick: move |_| library_provider.navigate_to_personal_tab("liked".to_string()),

                // 动态预览背景 (优化点 1)
                if let Some(cover) = first_track_cover {
                    div {
                        class: "absolute inset-0 z-0",
                        img {
                            class: "w-full h-full object-cover blur-[80px] opacity-40 scale-110",
                            src: "http://covers.localhost/{cover}"
                        }
                        div { class: "absolute inset-0 bg-gradient-to-br from-black/60 via-black/20 to-transparent" }
                    }
                } else {
                    div { class: "absolute inset-0 bg-gradient-to-br from-[#1a1c20] to-[#0a0b0d]" }
                }

                // 装饰性背景光
                div { class: "absolute -top-24 -right-24 w-96 h-96 bg-white/5 blur-[100px] rounded-full z-0" }

                // 心形图标区域
                div {
                    class: "relative z-10 w-44 h-44 bg-white/10 backdrop-blur-md rounded-3xl flex items-center justify-center shadow-inner group-hover:scale-105 transition-transform duration-500",
                    Icon { icon: FaHeart, class: "w-20 h-20 text-red-500 drop-shadow-[0_0_20px_rgba(239,68,68,0.4)]" }
                }

                // 文字信息
                div { class: "relative z-10 space-y-4",
                    div { class: "space-y-1",
                        span { class: "text-white/40 text-sm font-bold uppercase tracking-[0.2em]", "私密歌单" }
                        h2 { class: "text-6xl font-black text-white tracking-tighter", "我喜欢的音乐" }
                    }
                    div { class: "flex items-center gap-6",
                        div { class: "flex items-center gap-2 text-white/60 font-medium",
                            span { class: "text-2xl text-white font-black", "{liked_tracks.len()}" }
                            span { class: "text-xs mt-1", "首歌曲" }
                        }
                        div { class: "w-1 h-1 bg-white/20 rounded-full" }
                        div { class: "flex items-center gap-2 text-white/60 font-medium",
                            span { class: "text-2xl text-white font-black", "0" }
                            span { class: "text-xs mt-1", "已下载" }
                        }
                    }
                }

                // 播放悬浮按钮 (优化点 3)
                button {
                    class: "absolute right-12 bottom-12 w-16 h-16 bg-white rounded-full flex items-center justify-center shadow-xl opacity-0 translate-y-4 group-hover:opacity-100 group-hover:translate-y-0 transition-all duration-300 hover:scale-110 active:scale-95 z-20",
                    onclick: {
                        let tracks = liked_tracks.clone();
                        move |e| {
                            e.stop_propagation();
                            if !tracks.is_empty() {
                                let api_tracks: Vec<_> = tracks.iter().map(|t| t.to_api_track()).collect();
                                spawn(async move {
                                    player.set_queue_and_play(api_tracks).await;
                                });
                            }
                        }
                    },
                    Icon { icon: LdPlay, class: "w-8 h-8 text-black fill-black ml-1" }
                }
            }

            // --- 最近播放 ---
            Section {
                title: "最近播放",
                icon: rsx! { Icon { icon: LdHistory, class: "w-5 h-5 text-black" } },
                on_see_all: move |_| library_provider.navigate_to_personal_tab("recent".to_string()),
                div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4",
                    for track in recently_played.iter().take(4) {
                        div {
                            key: "{track.id}",
                            class: "bg-white hover:shadow-xl rounded-2xl p-4 flex items-center gap-4 transition-all duration-500 border border-gray-100 group cursor-pointer",

                            // 封面图及悬浮播放按钮 (优化点 3)
                            div {
                                class: "relative w-14 h-14 rounded-xl overflow-hidden flex-shrink-0 shadow-sm",
                                if let Some(cover) = &track.cover {
                                    img { class: "w-full h-full object-cover", src: "http://covers.localhost/{cover}" }
                                } else {
                                    div { class: "w-full h-full flex items-center justify-center bg-gray-50", Icon { icon: LdMusic, class: "w-6 h-6 text-gray-200" } }
                                }
                                // 悬浮播放蒙层
                                div {
                                    class: "absolute inset-0 bg-black/60 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity duration-300",
                                    onclick: {
                                        let t = track.clone();
                                        move |e| {
                                            e.stop_propagation();
                                            let api_track = t.to_api_track();
                                            spawn(async move {
                                                player.set_queue_and_play(vec![api_track]).await;
                                            });
                                        }
                                    },
                                    Icon { icon: LdPlay, class: "w-6 h-6 text-white fill-white" }
                                }
                            }

                            div { class: "flex-1 min-w-0" ,
                                div { class: "font-black text-sm truncate text-black", "{track.title}" }
                                div { class: "text-[11px] text-gray-500 font-bold truncate mt-0.5", "{track.artist}" }
                                div { class: "text-[9px] text-red-500 font-bold uppercase mt-1 flex items-center gap-1",
                                    div { class: "w-1 h-1 bg-red-500 rounded-full animate-pulse" }
                                    "正在听"
                                }
                            }
                        }
                    }
                    if recently_played.is_empty() {
                        div { class: "col-span-full py-10 text-center text-gray-300 font-medium italic", "暂无播放记录" }
                    }
                }
            }

            // --- 我的歌单 ---
            Section {
                title: "我的歌单",
                icon: rsx! { Icon { icon: LdListMusic, class: "w-5 h-5 text-black" } },
                on_see_all: move |_| library_provider.navigate_to_personal_tab("playlists".to_string()),
                div { class: "grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-6",
                    // 创建新歌单卡片
                    div {
                        class: "group cursor-pointer space-y-3",
                        onclick: move |e| {
                            e.stop_propagation();
                            library_provider.navigate_to_personal_tab("playlists_create".to_string());
                        },
                        div {
                            class: "aspect-square rounded-[2rem] border-2 border-dashed border-gray-200 flex flex-col items-center justify-center gap-3 text-gray-400 group-hover:border-black group-hover:text-black transition-all duration-300",
                            Icon { icon: LdPlus, class: "w-10 h-10" }
                            span { class: "text-xs font-bold", "创建新歌单" }
                        }
                    }
                    // 歌单项主循环
                    {playlists.iter().take(4).map(|playlist| {
                        let id = playlist.id.clone();
                        let p_clone = playlist.clone();
                        let mut lib = library_provider;
                        rsx! {
                            div {
                                key: "{id}",
                                class: "group cursor-pointer space-y-3",
                                onclick: move |_| {
                                    lib.navigate_to_playlist(p_clone.clone());
                                },
                                div {
                                    class: "aspect-square rounded-[2rem] bg-white border border-gray-100 overflow-hidden shadow-sm group-hover:shadow-2xl group-hover:-translate-y-2 transition-all duration-500",
                                    if let Some(cover) = &playlist.cover {
                                        img { class: "w-full h-full object-cover", src: "http://covers.localhost/{cover}" }
                                    } else {
                                        div { class: "w-full h-full flex items-center justify-center bg-gray-50",
                                            Icon { icon: LdListMusic, class: "w-12 h-12 text-gray-100" }
                                        }
                                    }
                                }
                                div { class: "px-2",
                                    div { class: "font-black text-sm truncate text-black", "{playlist.name}" }
                                    div { class: "text-xs text-gray-400 font-bold", "{playlist.track_count} 首歌曲" }
                                }
                            }
                        }
                    })}
                }
            }

            // --- 底部两栏网格 ---
            div { class: "grid grid-cols-1 lg:grid-cols-2 gap-12",
                // 收藏的专辑
                Section {
                    title: "收藏的专辑",
                    icon: rsx! { Icon { icon: LdDisc, class: "w-5 h-5 text-black" } },
                    on_see_all: move |_| library_provider.navigate_to_personal_tab("albums".to_string()),
                    div { class: "grid grid-cols-2 gap-4",
                        for album in liked_albums.iter().take(4) {
                            div {
                                key: "{album.title}",
                                class: "flex items-center gap-4 p-3 rounded-2xl hover:bg-white transition-colors cursor-pointer group",
                                onclick: {
                                    let lib = library_provider;
                                    let id = album.id.clone();
                                    move |_| {
                                        let mut l = lib;
                                        let aid = id.clone();
                                        spawn(async move {
                                            l.navigate_to_album(aid).await;
                                        });
                                    }
                                },
                                div {
                                    class: "w-16 h-16 bg-gray-100 rounded-xl overflow-hidden shadow-sm group-hover:rotate-3 transition-transform duration-300",
                                    if let Some(cover) = &album.cover {
                                        img { class: "w-full h-full object-cover", src: "http://covers.localhost/{cover}" }
                                    } else {
                                        div { class: "w-full h-full flex items-center justify-center", Icon { icon: LdDisc, class: "w-8 h-8 text-gray-300" } }
                                    }
                                }
                                div { class: "flex-1 min-w-0",
                                    div { class: "font-bold text-sm truncate", "{album.title}" }
                                    div { class: "text-xs text-gray-400 truncate", "{album.artist}" }
                                }
                            }
                        }
                        if liked_albums.is_empty() {
                            div { class: "col-span-full py-8 text-center text-gray-300 text-sm", "还没有收藏专辑" }
                        }
                    }
                }

                // 收藏的歌手
                Section {
                    title: "收藏的歌手",
                    icon: rsx! { Icon { icon: LdUser, class: "w-5 h-5 text-black" } },
                    on_see_all: move |_| library_provider.navigate_to_personal_tab("artists".to_string()),
                    div { class: "flex flex-wrap gap-6",
                        for artist in liked_artists.iter().take(4) {
                            div {
                                key: "{artist.name}",
                                class: "flex flex-col items-center gap-2 group cursor-pointer",
                                onclick: {
                                    let lib = library_provider;
                                    let id = artist.id.clone();
                                    move |_| {
                                        let mut l = lib;
                                        let aid = id.clone();
                                        spawn(async move {
                                            l.navigate_to_artist(aid).await;
                                        });
                                    }
                                },
                                div {
                                    class: "w-20 h-20 bg-gray-100 rounded-full flex items-center justify-center overflow-hidden border-2 border-transparent group-hover:border-black transition-all duration-300 shadow-sm group-hover:shadow-lg",
                                    if let Some(cover) = &artist.cover_path {
                                        img { class: "w-full h-full object-cover", src: "http://covers.localhost/{cover}" }
                                    } else {
                                        Icon { icon: LdUser, class: "w-10 h-10 text-gray-300 group-hover:scale-110 transition-transform" }
                                    }
                                }
                                div { class: "text-xs font-bold text-center w-20 truncate", "{artist.name}" }
                                div { class: "text-[10px] text-gray-400 font-bold uppercase", "艺术家" }
                            }
                        }
                        if liked_artists.is_empty() {
                            div { class: "w-full py-8 text-center text-gray-300 text-sm", "还没有收藏歌手" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn Section(
    title: String,
    icon: Element,
    on_see_all: EventHandler<()>,
    children: Element,
) -> Element {
    rsx! {
        div { class: "space-y-6",
            div { class: "flex items-center justify-between",
                div { class: "flex items-center gap-3",
                    div { class: "w-10 h-10 rounded-xl bg-black/5 flex items-center justify-center",
                        {icon}
                    }
                    h3 { class: "text-2xl font-black tracking-tight", "{title}" }
                }
                button {
                    class: "text-xs font-bold text-gray-400 hover:text-black transition-colors flex items-center gap-1 group",
                    onclick: move |_| on_see_all.call(()),
                    "查看全部"
                    Icon { icon: LdChevronRight, class: "w-4 h-4 translate-x-0 group-hover:translate-x-1 transition-transform" }
                }
            }
            {children}
        }
    }
}
