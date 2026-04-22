use crate::components::personal::{
    LikedTracksList, PersonalDashboard, PlaylistDetail, PlaylistManager,
};
use crate::state::library::LibraryView;
use crate::state::{LibraryProvider, PersonalProvider, PlayerProvider};
use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::*;
use dioxus_free_icons::Icon;

#[component]
pub fn PersonalSpacePage() -> Element {
    let library_provider = use_context::<LibraryProvider>();
    let personal_provider = use_context::<PersonalProvider>();
    let player = use_context::<PlayerProvider>();

    let liked_albums = personal_provider.liked_albums();
    let liked_artists = personal_provider.liked_artists();
    let recently_played = personal_provider.recently_played();

    let rp_tracks = use_memo(move || personal_provider.recently_played());
    let rp_api_tracks = use_memo(move || {
        personal_provider
            .recently_played()
            .iter()
            .map(|t| t.to_api_track())
            .collect::<Vec<_>>()
    });

    rsx! {
        div {
            class: "flex-1 overflow-y-auto overflow-x-hidden",

            div {
                class: "max-w-7xl mx-auto px-10 py-12 space-y-12",

                // 页面头部
                div {
                    class: "flex items-center justify-between",

                    div {
                        class: "space-y-1",
                        h1 {
                            class: "text-5xl font-black tracking-tighter text-black",
                            match library_provider.current_view.read().clone() {
                                LibraryView::PersonalOverview => "私人空间",
                                LibraryView::PersonalLiked => "我喜欢的歌曲",
                                LibraryView::PersonalPlaylists
                                | LibraryView::PersonalPlaylistsCreate => "我的歌单",
                                LibraryView::PlaylistDetail(_) => "歌单详情",
                                LibraryView::PersonalAlbums => "收藏的专辑",
                                LibraryView::PersonalArtists => "收藏的歌手",
                                LibraryView::PersonalRecent => "最近播放",
                                _ => "私人空间"
                            }
                        }
                        if let LibraryView::PersonalOverview = library_provider.current_view.read().clone() {
                            p {
                                class: "text-gray-400 text-lg font-medium ml-1",
                                "管理你的音乐收藏"
                            }
                        }
                    }
                }

                // 内容区域
                match library_provider.current_view.read().clone() {
                    LibraryView::PersonalOverview => rsx! {
                        PersonalDashboard {}
                    },
                    LibraryView::PersonalLiked => rsx! {
                        LikedTracksList {}
                    },
                    LibraryView::PersonalPlaylists
                    | LibraryView::PersonalPlaylistsCreate => rsx! {
                        PlaylistManager {}
                    },
                    LibraryView::PlaylistDetail(playlist) => rsx! {
                        PlaylistDetail { playlist: playlist.clone() }
                    },
                    LibraryView::PersonalAlbums => rsx! {
                        div {
                            class: "animate-in fade-in slide-in-from-bottom-4 duration-500",
                            if liked_albums.is_empty() {
                                div { class: "text-gray-400 py-24 text-center bg-gray-50 rounded-3xl border-2 border-dashed border-gray-100",
                                    div { class: "text-4xl mb-4", "💿" }
                                    "还没有收藏专辑"
                                }
                            } else {
                                div { class: "grid grid-cols-2 md:grid-cols-4 lg:grid-cols-5 gap-8",
                                    {liked_albums.iter().map(|album| {
                                        rsx! {
                                            div {
                                                key: "{album.title}",
                                                class: "group cursor-pointer space-y-4",
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
                                                div { class: "aspect-square bg-gray-100 rounded-[2rem] overflow-hidden shadow-sm group-hover:shadow-2xl group-hover:-translate-y-2 transition-all duration-500",
                                                    if let Some(cover) = &album.cover {
                                                        img { class: "w-full h-full object-cover", src: "http://covers.localhost/{cover}" }
                                                    } else {
                                                        div { class: "w-full h-full flex items-center justify-center",
                                                            Icon { icon: LdDisc, class: "w-16 h-16 text-gray-200" }
                                                        }
                                                    }
                                                }
                                                div { class: "px-2",
                                                    div { class: "font-black text-base truncate", "{album.title}" }
                                                    div { class: "text-gray-400 text-sm font-bold", "{album.artist}" }
                                                }
                                            }
                                        }
                                    })}
                                }
                            }
                        }
                    },
                    LibraryView::PersonalArtists => rsx! {
                        div {
                            class: "animate-in fade-in slide-in-from-bottom-4 duration-500",
                            if liked_artists.is_empty() {
                                div { class: "text-gray-400 py-24 text-center bg-gray-50 rounded-3xl border-2 border-dashed border-gray-100",
                                    div { class: "text-4xl mb-4", "👤" }
                                    "还没有收藏歌手"
                                }
                            } else {
                                div { class: "grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-10",
                                    {liked_artists.iter().map(|artist| {
                                        rsx! {
                                            div {
                                                key: "{artist.name}",
                                                class: "flex flex-col items-center gap-4 group cursor-pointer",
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
                                                div { class: "w-full aspect-square bg-gray-100 rounded-full overflow-hidden border-4 border-transparent group-hover:border-black shadow-sm group-hover:shadow-xl transition-all duration-500",
                                                    if let Some(cover) = &artist.cover_path {
                                                        img { class: "w-full h-full object-cover", src: "http://covers.localhost/{cover}" }
                                                    } else {
                                                        div { class: "w-full h-full flex items-center justify-center",
                                                            Icon { icon: LdUser, class: "w-20 h-20 text-gray-200 group-hover:scale-110 transition-transform duration-500" }
                                                        }
                                                    }
                                                }
                                                div { class: "font-black text-center truncate w-full px-2", "{artist.name}" }
                                            }
                                        }
                                    })}
                                }
                            }
                        }
                    },
                    LibraryView::PersonalRecent => rsx! {
                        div {
                            class: "animate-in fade-in slide-in-from-bottom-4 duration-500",
                            if recently_played.is_empty() {
                                div { class: "text-gray-400 py-24 text-center bg-gray-50 rounded-3xl border-2 border-dashed border-gray-100",
                                    div { class: "text-4xl mb-4", "🕒" }
                                    "暂无播放记录"
                                }
                            } else {
                                div {
                                    class: "flex items-center justify-between px-6 mb-6",
                                    div { class: "text-gray-400 text-sm font-bold", "共 {recently_played.len()} 首歌" },
                                    button {
                                        class: "px-6 py-2.5 bg-black text-white text-sm font-black rounded-full hover:scale-105 active:scale-95 transition-all shadow-lg hover:shadow-xl flex items-center gap-2 group",
                                        onclick: move |_| {
                                            let mut p = player;
                                            let tracks = rp_api_tracks.read().clone();
                                            spawn(async move {
                                                p.set_queue_and_play(tracks).await;
                                            });
                                        },
                                        Icon { width: 16, height: 16, icon: LdPlay, class: "group-hover:scale-110 transition-transform" }
                                        "播放全部"
                                    }
                                }
                                div {
                                    class: "bg-white rounded-[2.5rem] py-2 shadow-sm border border-gray-50",
                                    crate::components::music_library::TrackTable {
                                        tracks: rp_tracks,
                                        api_tracks: rp_api_tracks,
                                        show_played_at: true,
                                    }
                                }
                            }
                        }
                    },
                    LibraryView::AllTracksDetail { .. } => rsx! {
                        crate::components::music_library::AllTracksDetail {}
                    },
                    _ => rsx! {
                        PersonalDashboard {}
                    }
                }
            }
        }
    }
}
