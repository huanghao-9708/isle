use crate::components::music_library::TrackTable;
use crate::state::library::LibraryView;
use crate::state::{LibraryProvider, PersonalProvider, PlayerProvider};
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::FaHeart as FaHeartSolid;
use dioxus_free_icons::icons::ld_icons::*;
use dioxus_free_icons::Icon;

#[component]
pub fn EntityDetailPage() -> Element {
    let mut provider = use_context::<LibraryProvider>();
    let mut player = use_context::<PlayerProvider>();
    let personal_ctx = use_context::<PersonalProvider>();
    let view = provider.current_view.read().clone();
    let mut active_tab = use_signal(|| "songs".to_string());

    // 提取当前实体的显示数据
    let (entity_type, entity_id, title, subtitle, description, cover, is_artist) = match view {
        LibraryView::ArtistDetail(artist) => (
            "ARTIST",
            artist.id,
            artist.name,
            Some("艺术家".to_string()),
            artist
                .bio
                .unwrap_or_else(|| "这位艺术家很神秘，暂时没有简介。".to_string()),
            artist.cover_path,
            true,
        ),
        LibraryView::AlbumDetail(album) => (
            "ALBUM",
            album.id,
            album.title,
            Some(album.artist_name),
            album
                .description
                .unwrap_or_else(|| "暂无专辑介绍。".to_string()),
            album.cover_path,
            false,
        ),
        LibraryView::GenreDetail(genre) => (
            "GENRE",
            genre.id.to_string(),
            genre.name.clone(),
            None,
            genre.description.unwrap_or_else(|| {
                format!(
                    "深入探索 {} 的音乐世界。我们为您精选了该流派下最经典、最具代表性的声音。",
                    genre.name
                )
            }),
            genre.image,
            false,
        ),
        _ => return rsx! { div { "无效的视图状态" } },
    };

    let is_artist_followed =
        entity_type == "ARTIST" && personal_ctx.liked_artists().iter().any(|a| a.name == title);
    let is_album_liked = entity_type == "ALBUM"
        && personal_ctx
            .liked_albums()
            .iter()
            .any(|a| a.id == entity_id);

    let cover_url = if let Some(path) = &cover {
        format!("http://covers.localhost/{}", path)
    } else {
        "".to_string()
    };

    rsx! {
        div {
            class: "flex-1 overflow-y-auto bg-white transition-all duration-500",

            // 详情页眉区域
            div {
                class: "px-10 py-12",
                div {
                    class: "bg-[#F8F9FA] rounded-[32px] p-12 flex items-center gap-16 relative overflow-hidden group",

                    // 封面展示
                    div {
                        class: format!(
                            "relative z-10 w-64 h-64 shrink-0 shadow-2xl transition-transform duration-700 group-hover:scale-[1.02] {}",
                            if is_artist { "rounded-full overflow-hidden" } else { "rounded-2xl overflow-hidden" }
                        ),
                        if !cover_url.is_empty() {
                            img {
                                src: "{cover_url}",
                                class: "w-full h-full object-cover",
                            }
                        } else {
                            div { class: "w-full h-full bg-gray-200 flex items-center justify-center",
                                if is_artist {
                                    Icon { width: 64, height: 64, icon: LdUser, class: "text-gray-400" }
                                } else {
                                    Icon { width: 64, height: 64, icon: LdDisc, class: "text-gray-400" }
                                }
                            }
                        }
                    }

                    // 信息展示
                    div {
                        class: "relative z-10 flex-1 space-y-6",
                        div {
                            class: "space-y-2",
                            span { class: "text-[10px] font-black uppercase tracking-[0.3em] text-gray-400", "{entity_type}" }
                            h2 { class: "text-6xl font-black tracking-tighter text-black leading-tight", "{title}" }
                            if let Some(sub) = &subtitle {
                                p { class: "text-xl font-bold text-gray-500", "{sub}" }
                            }
                        }

                        p {
                            class: "text-gray-400 text-sm leading-relaxed max-w-2xl font-medium",
                            "{description}"
                        }

                        // 操作按钮
                        div {
                            class: "flex items-center gap-4 pt-4",
                            button {
                                class: "h-12 bg-black hover:bg-gray-800 text-white px-8 rounded-full flex items-center gap-2 transition-all hover:scale-105 active:scale-95 shadow-lg",
                                onclick: move |_| {
                                    let tracks = provider.detail_api_tracks.read().clone();
                                    let is_genre = entity_type == "GENRE";
                                    spawn(async move {
                                        if is_genre {
                                            player.set_queue_and_play(tracks).await;
                                        } else {
                                            player.set_queue_and_play(tracks).await;
                                        }
                                    });
                                },
                                if entity_type == "GENRE" {
                                    Icon { width: 18, height: 18, icon: LdShuffle }
                                } else {
                                    Icon { width: 18, height: 18, icon: LdPlay }
                                }
                                span { class: "font-bold", if entity_type == "GENRE" { "随机播放" } else { "播放全部" } }
                            }

                            if entity_type == "ARTIST" {
                                button {
                                    class: format!(
                                        "h-12 px-8 rounded-full font-bold transition-all active:scale-95 {}",
                                        if is_artist_followed { "bg-black text-white" } else { "bg-white text-black border border-gray-200 hover:bg-gray-50" }
                                    ),
                                    onclick: move |_| {
                                        let p = personal_ctx;
                                        let name = title.clone();
                                        spawn(async move {
                                            let mut p = p;
                                            if is_artist_followed {
                                                let _ = p.unlike_artist(&name).await;
                                            } else {
                                                let _ = p.like_artist(&name).await;
                                            }
                                        });
                                    },
                                    if is_artist_followed { "已关注" } else { "关注" }
                                }
                            } else if entity_type == "ALBUM" {
                                button {
                                    class: format!(
                                        "w-12 h-12 rounded-full flex items-center justify-center transition-all active:scale-95 {}",
                                        if is_album_liked { "bg-red-50 text-red-500 border border-red-100" } else { "bg-white text-black border border-gray-200 hover:bg-gray-50" }
                                    ),
                                    onclick: move |_| {
                                        let p = personal_ctx;
                                        let aid = entity_id.clone();
                                        let t = title.clone();
                                        let art = subtitle.clone().unwrap_or_default();
                                        let cov = cover.clone();
                                        spawn(async move {
                                            let mut p = p;
                                            if is_album_liked {
                                                let _ = p.unlike_album(&aid).await;
                                            } else {
                                                let _ = p.like_album(&aid, &t, &art, cov.as_deref()).await;
                                            }
                                        });
                                    },
                                    if is_album_liked {
                                        Icon { width: 20, height: 20, icon: FaHeartSolid }
                                    } else {
                                        Icon { width: 20, height: 20, icon: LdHeart }
                                    }
                                }
                                button {
                                    class: "w-12 h-12 bg-white hover:bg-gray-50 border border-gray-200 rounded-full flex items-center justify-center transition-all active:scale-95",
                                    Icon { width: 20, height: 20, icon: LdEllipsis }
                                }
                            } else {
                                button {
                                    class: "w-12 h-12 bg-white hover:bg-gray-50 border border-gray-200 rounded-full flex items-center justify-center transition-all active:scale-95",
                                    Icon { width: 20, height: 20, icon: LdHeart }
                                }
                                button {
                                    class: "w-12 h-12 bg-white hover:bg-gray-50 border border-gray-200 rounded-full flex items-center justify-center transition-all active:scale-95",
                                    Icon { width: 20, height: 20, icon: LdEllipsis }
                                }
                            }
                        }
                    }

                    // 背景装饰效果 (模糊的水印或者光晕)
                    div {
                        class: "absolute top-0 right-0 w-1/2 h-full opacity-5 pointer-events-none select-none",
                        if is_artist {
                            Icon { width: 400, height: 400, icon: LdUser, class: "translate-x-1/4 -translate-y-1/4" }
                        } else {
                            Icon { width: 400, height: 400, icon: LdDisc, class: "translate-x-1/4 -translate-y-1/4" }
                        }
                    }
                }
            }

            // Tab 切换 (仅限艺术家详情)
            if is_artist {
                div {
                    class: "px-10 mb-8 border-b border-gray-100 flex gap-12",
                    button {
                        class: format!("pb-4 font-black tracking-tight text-xl transition-all relative {}",
                            if active_tab() == "songs" { "text-black" } else { "text-gray-300 hover:text-gray-500" }),
                        onclick: move |_| active_tab.set("songs".to_string()),
                        "歌曲"
                        if active_tab() == "songs" {
                            div { class: "absolute bottom-0 left-0 w-full h-1.5 bg-black rounded-t-full" }
                        }
                    }
                    button {
                        class: format!("pb-4 font-black tracking-tight text-xl transition-all relative {}",
                            if active_tab() == "albums" { "text-black" } else { "text-gray-300 hover:text-gray-500" }),
                        onclick: move |_| active_tab.set("albums".to_string()),
                        "专辑"
                        if active_tab() == "albums" {
                            div { class: "absolute bottom-0 left-0 w-full h-1.5 bg-black rounded-t-full" }
                        }
                    }
                }
            }

            // 内容区域
            div {
                class: "px-10 pb-20",
                if !is_artist || active_tab() == "songs" {
                    div {
                        class: "border-t border-gray-100",
                        TrackTable {
                            tracks: provider.detail_tracks,
                            api_tracks: provider.detail_api_tracks,
                            display_limit: Some(20),
                            on_view_all: move |_| {
                                let view = provider.current_view.read().clone();
                                let mut filter = api::models::TrackFilter::default();
                                let (title, source_type) = match view {
                                    LibraryView::ArtistDetail(a) => {
                                        filter.artist_id = Some(a.id.clone());
                                        (format!("{} 的作品", a.name), "artist".to_string())
                                    }
                                    LibraryView::AlbumDetail(a) => {
                                        filter.album_id = Some(a.id.clone());
                                        (a.title, "album".to_string())
                                    }
                                    LibraryView::GenreDetail(g) => {
                                        filter.genre_id = Some(g.id);
                                        (g.name, "genre".to_string())
                                    }
                                    _ => ("列表".to_string(), "detail".to_string()),
                                };
                                provider.navigate_to_all_tracks_detail(title, filter, source_type);
                            }
                        }
                    }
                } else {
                    // 专辑展示网格 (仅限艺术家)
                    div {
                        class: "grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-8",
                        {provider.detail_albums.read().iter().map(|album| {
                            let album = album.clone();
                            let mut provider = provider;
                            let cover_filename = album.cover_path.clone();
                            let url = if let Some(path) = cover_filename {
                                format!("http://covers.localhost/{}", path)
                            } else {
                                "".to_string()
                            };

                            rsx! {
                                div {
                                    key: "{album.id}",
                                    class: "group cursor-pointer space-y-4",
                                    onclick: move |_| {
                                        let album = album.clone();
                                        spawn(async move {
                                            provider.navigate_to_album(album.id).await;
                                        });
                                    },

                                    // 封面磁贴
                                    div {
                                        class: "aspect-square bg-[#F8F9FA] rounded-[24px] overflow-hidden shadow-sm group-hover:shadow-xl transition-all duration-500 transform group-hover:-translate-y-2 relative",
                                        if !url.is_empty() {
                                            img {
                                                src: "{url}",
                                                class: "w-full h-full object-cover transition-transform duration-700 group-hover:scale-110",
                                            }
                                        } else {
                                            div { class: "w-full h-full flex items-center justify-center",
                                                Icon { width: 48, height: 48, icon: LdDisc, class: "text-gray-200" }
                                            }
                                        }
                                        // 悬浮遮罩
                                        div {
                                            class: "absolute inset-0 bg-black/0 group-hover:bg-black/5 transition-colors duration-500 flex items-center justify-center",
                                            div {
                                                class: "w-12 h-12 bg-white rounded-full shadow-lg flex items-center justify-center opacity-0 translate-y-4 group-hover:opacity-100 group-hover:translate-y-0 transition-all duration-500",
                                                Icon { width: 20, height: 20, icon: LdPlay, class: "text-black translate-x-0.5" }
                                            }
                                        }
                                    }

                                    // 专辑信息
                                    div {
                                        class: "space-y-1 px-1",
                                        h3 { class: "font-black tracking-tight text-black truncate group-hover:text-gray-600 transition-colors", "{album.title}" }
                                        p { class: "text-[10px] font-bold text-gray-400 uppercase tracking-widest", "{album.track_count} 首歌曲" }
                                    }
                                }
                            }
                        })}
                    }
                }
            }
        }
    }
}
