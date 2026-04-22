use crate::components::music_library::{StatsBoard, TrackTable};
use crate::state::library::LibraryView;
use crate::state::LibraryProvider;
use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::*;
use dioxus_free_icons::Icon;

#[component]
pub fn MusicLibraryPage() -> Element {
    let mut provider = use_context::<LibraryProvider>();
    let view = provider.current_view.read().clone();
    let mut active_tab = use_signal(|| "songs".to_string());
    let is_scanning = provider.is_scanning;

    let mut show_folder_manager = use_signal(|| false);

    rsx! {
        div {
            class: "flex-1 overflow-y-auto overflow-x-hidden",

            if is_scanning() {
                div {
                    class: "fixed top-6 left-1/2 -translate-x-1/2 z-[100] transition-all duration-500 transform translate-y-0",
                    div {
                        class: "flex items-center gap-3 px-6 py-2.5 bg-black/80 backdrop-blur-xl border border-white/10 rounded-full shadow-2xl text-white",
                        div {
                            class: "w-4 h-4 border-2 border-white/20 border-t-white rounded-full animate-spin"
                        }
                        div {
                            class: "flex flex-col",
                            span { class: "text-[10px] font-black uppercase tracking-[0.2em] text-white/50 leading-none mb-1", "Library Syncing" }
                            span { class: "text-xs font-bold tracking-tight leading-none", "正在深度扫描音频库..." }
                        }
                    }
                }
            }

            if show_folder_manager() {
                crate::components::music_library::LibraryPanel {
                    on_apply: move |new_folders| {
                        info!("MusicLibraryPage: 接收到目录变更请求, 正在关闭弹窗并切换后台执行...");
                        show_folder_manager.set(false);
                        let mut provider = provider.clone();
                        spawn(async move {
                            if let Err(e) = provider.apply_folder_changes(new_folders).await {
                                error!("MusicLibraryPage: 异步目录同步失败: {}", e);
                            } else {
                                info!("MusicLibraryPage: 后台目录同步及扫描任务已完成");
                            }
                        });
                    },
                    on_close: move |_| {
                        info!("MusicLibraryPage: 收到手动关闭请求, 正在隐藏文件夹管理器弹窗...");
                        show_folder_manager.set(false);
                    },
                }
            }

            match view {
                LibraryView::Main => rsx! {
                    div {
                        class: "max-w-7xl mx-auto px-10 py-12 space-y-12",

                        div {
                            class: "flex items-end justify-between",

                            div {
                                class: "space-y-1",
                                h1 {
                                    class: "text-5xl font-black tracking-tighter text-black",
                                    "音乐库"
                                }
                                div {
                                    class: "text-[10px] text-gray-400 font-bold uppercase tracking-widest ml-1",
                                    "Music Library Collection"
                                }
                            }

                            div {
                                class: "flex items-center gap-3",

                                button {
                                    class: "h-10 bg-black hover:bg-gray-800 text-white border-none px-5 rounded-md flex items-center gap-2 transition-all hover:scale-[1.02] active:scale-95 shadow-sm",
                                    onclick: move |_| show_folder_manager.set(true),
                                    Icon { width: 14, height: 14, icon: LdFolderCog }
                                    span { class: "font-bold text-sm",
                                        if is_scanning() { "扫描中..." } else { "音乐目录管理" }
                                    }
                                }

                                button {
                                    class: "h-10 bg-[#E4E6EA] hover:bg-[#D9DBE0] text-black border-none px-5 rounded-md flex items-center gap-2 transition-all active:scale-95",
                                    onclick: move |_| {
                                        spawn(async move {
                                            provider.refresh_all().await;
                                        });
                                    },
                                    Icon {
                                        width: 14,
                                        height: 14,
                                        icon: LdRotateCw,
                                        class: if is_scanning() { "animate-spin" } else { "" }
                                    }
                                    span { class: "font-bold text-sm", "同步" }
                                }
                            }
                        }

                        StatsBoard {
                            track_count: provider.track_count,
                            album_count: provider.album_count,
                            artist_count: provider.artist_count,
                            genre_count: provider.genre_count,
                            folder_count: provider.folder_count,
                        }

                        div {
                            class: "bg-[#F0F1F3] rounded-lg overflow-hidden",

                            div {
                                class: "px-8 pt-8 flex items-center gap-8 border-b border-gray-50",

                                {["songs", "albums", "artists", "genres", "folders"].iter().map(|tab| {
                                    let label = match *tab {
                                        "songs" => "歌曲",
                                        "albums" => "专辑",
                                        "artists" => "艺术家",
                                        "genres" => "流派",
                                        "folders" => "文件夹",
                                        _ => "",
                                    };
                                    let active = active_tab() == *tab;
                                    rsx! {
                                        button {
                                            key: "{tab}",
                                            class: format!(
                                                "pb-4 text-sm transition-all duration-300 font-bold tracking-tight relative {}",
                                                if active {
                                                    "text-black after:content-[''] after:absolute after:bottom-0 after:left-0 after:w-full after:h-1 after:bg-black after:rounded-full"
                                                } else {
                                                    "text-gray-300 hover:text-gray-500"
                                                }
                                            ),
                                            onclick: move |_| {
                                                let mut tab_signal = active_tab;
                                                tab_signal.set(tab.to_string());
                                            },
                                            "{label}"
                                        }
                                    }
                                })}
                            }

                            div {
                                class: "min-h-[500px]",
                                match active_tab().as_str() {
                                    "songs" => rsx! {
                                        div {
                                            class: "flex flex-col",

                                            if provider.filter.read().clone() != api::models::TrackFilter::default() {
                                                div {
                                                    class: "px-6 py-3 bg-black/5 flex items-center justify-between border-b border-gray-100",
                                                    div {
                                                        class: "flex items-center gap-2 text-xs font-bold text-gray-400 uppercase tracking-widest",
                                                        Icon { width: 14, height: 14, icon: LdFilter }
                                                        span { "正在筛选结果" }
                                                    }
                                                    button {
                                                        class: "text-xs font-bold text-black hover:text-gray-600 transition-colors uppercase tracking-widest flex items-center gap-1",
                                                        onclick: move |_| {
                                                            spawn(async move {
                                                                provider.clear_filter().await;
                                                            });
                                                        },
                                                        Icon { width: 12, height: 12, icon: LdX }
                                                        span { "清除所有筛选" }
                                                    }
                                                }
                                            }

                                            TrackTable {
                                                tracks: provider.tracks,
                                                api_tracks: provider.current_api_tracks,
                                                display_limit: Some(20),
                                                on_view_all: move |_| {
                                                    let filter = provider.filter.read().clone();
                                                    provider.navigate_to_all_tracks_detail(
                                                        "音乐库".to_string(),
                                                        filter,
                                                        "library".to_string()
                                                    );
                                                }
                                            }
                                        }
                                    },
                                    "artists" => rsx! {
                                        crate::components::music_library::CategoryList {
                                            category: "artists",
                                            on_select: move |name| {
                                                let mut filter = api::models::TrackFilter::default();
                                                filter.artist = Some(name);
                                                spawn(async move {
                                                    provider.set_filter(filter).await;
                                                    active_tab.set("songs".to_string());
                                                });
                                            }
                                        }
                                    },
                                    "albums" => rsx! {
                                        crate::components::music_library::CategoryList {
                                            category: "albums",
                                            on_select: move |title| {
                                                let mut filter = api::models::TrackFilter::default();
                                                filter.album = Some(title);
                                                spawn(async move {
                                                    provider.set_filter(filter).await;
                                                    active_tab.set("songs".to_string());
                                                });
                                            }
                                        }
                                    },
                                    "genres" => rsx! {
                                        crate::components::music_library::CategoryList {
                                            category: "genres",
                                            on_select: move |name| {
                                                let mut filter = api::models::TrackFilter::default();
                                                filter.genres = Some(vec![name]);
                                                spawn(async move {
                                                    provider.set_filter(filter).await;
                                                    active_tab.set("songs".to_string());
                                                });
                                            }
                                        }
                                    },
                                    "folders" => rsx! {
                                        crate::components::music_library::CategoryList {
                                            category: "folders",
                                            on_select: move |path| {
                                                let mut filter = api::models::TrackFilter::default();
                                                filter.folder_path_prefix = Some(path);
                                                spawn(async move {
                                                    provider.set_filter(filter).await;
                                                    active_tab.set("songs".to_string());
                                                });
                                            }
                                        }
                                    },
                                    _ => rsx! {
                                        div {
                                            class: "flex flex-col items-center justify-center py-32 text-gray-200 gap-6",
                                            Icon { width: 64, height: 64, icon: LdBox, class: "opacity-20" }
                                            span { class: "font-black tracking-[0.2em] uppercase text-xs text-gray-300", "即将上线 Coming Soon" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                LibraryView::AllTracksDetail { .. } => rsx! {
                    crate::components::music_library::AllTracksDetail {}
                },
                _ => rsx! {
                    crate::components::music_library::EntityDetailPage {}
                }
            }
        }
    }
}
