use crate::components::music_library::track_table::TrackTable;
use crate::components::types::Track as UITrack;
use crate::state::library::{LibraryProvider, LibraryView};
use crate::state::personal::PersonalProvider;
use api::models::{PaginatedResult, Track as ApiTrack, TrackFilter};
use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{LdArrowLeft, LdSearch};
use dioxus_free_icons::Icon;
use tracing::info;

#[component]
pub fn AllTracksDetail() -> Element {
    let mut lib = use_context::<LibraryProvider>();
    let personal = use_context::<PersonalProvider>();
    let view = lib.current_view.read().clone();

    let (title, filter, source_type) = if let LibraryView::AllTracksDetail {
        title,
        filter,
        source_type,
    } = view
    {
        (title, filter, source_type)
    } else {
        return rsx! { div { "无效的视图状态" } };
    };

    let mut current_page = use_signal(|| 1usize);
    let page_size = 20;

    // 数据获取 Resource
    let tracks_resource = use_resource(move || {
        let st = source_type.clone();
        let f = filter.clone();
        let page = current_page();

        async move {
            info!(
                "AllTracksDetail: 开始获取分页数据, source_type={}, filter={:?}, page={}",
                st, f, page
            );
            let res: Result<PaginatedResult<ApiTrack>, String> = if st == "liked" {
                personal
                    .get_liked_tracks_paginated(page_size, (page - 1) * page_size)
                    .await
            } else if st.starts_with("playlist:") {
                let playlist_id = st.strip_prefix("playlist:").unwrap_or_default();
                personal
                    .get_playlist_tracks_paginated(playlist_id, page_size, (page - 1) * page_size)
                    .await
            } else {
                let l = lib.clone();
                l.filter_tracks_paginated(f, page, page_size).await
            };

            match &res {
                Ok(data) => info!(
                    "AllTracksDetail: 成功获取数据, count={}, total={}",
                    data.items.len(),
                    data.total
                ),
                Err(e) => error!("AllTracksDetail: 获取数据失败: {}", e),
            }
            res
        }
    });

    let (tracks, api_tracks_raw, total_count) = if let Some(Ok(res)) = &*tracks_resource.read() {
        let items = res.items.clone();
        (
            items
                .iter()
                .map(|t| UITrack::from(t.clone()))
                .collect::<Vec<UITrack>>(),
            items,
            res.total,
        )
    } else {
        (Vec::new(), Vec::new(), 0)
    };

    let total_pages = (total_count + page_size - 1) / page_size;

    rsx! {
        div { class: "flex flex-col h-full bg-white animate-in fade-in duration-300",
            // Header
            div { class: "flex items-center justify-between px-8 py-6 border-b border-gray-100 bg-white/80 backdrop-blur-md sticky top-0 z-10",
                div { class: "flex items-center gap-4",
                    button {
                        class: "p-2 hover:bg-gray-100 rounded-full transition-colors active:scale-90",
                        onclick: move |_| lib.navigate_back(),
                        Icon { width: 24, height: 24, icon: LdArrowLeft }
                    }
                    div { class: "flex flex-col",
                        h1 { class: "text-2xl font-black tracking-tight", "{title}" }
                        span { class: "text-xs font-bold text-gray-400 uppercase tracking-widest", "{total_count} TRACKS" }
                    }
                }

                // 分页控制
                div { class: "flex items-center gap-2",
                    button {
                        class: "p-2 hover:bg-gray-100 rounded-lg transition-all disabled:opacity-30 disabled:cursor-not-allowed",
                        disabled: current_page() <= 1,
                        onclick: move |_| current_page.set(current_page() - 1),
                        "上一页"
                    }
                    span { class: "px-4 py-1.5 bg-black text-white rounded-full text-sm font-black",
                        "{current_page()} / {total_pages.max(1)}"
                    }
                    button {
                        class: "p-2 hover:bg-gray-100 rounded-lg transition-all disabled:opacity-30 disabled:cursor-not-allowed",
                        disabled: current_page() >= total_pages,
                        onclick: move |_| current_page.set(current_page() + 1),
                        "下一页"
                    }
                }
            }

            // 内容列表
            div { class: "flex-1 overflow-y-auto",
                if tracks_resource.read().is_none() {
                    div { class: "flex items-center justify-center h-full",
                        div { class: "animate-pulse flex flex-col items-center gap-4",
                            div { class: "w-12 h-12 rounded-full bg-gray-200" }
                            div { class: "h-4 w-32 bg-gray-100 rounded" }
                        }
                    }
                } else {
                    TrackTable {
                        tracks: Signal::new(tracks),
                        api_tracks: Signal::new(api_tracks_raw),
                        display_limit: None,
                    }
                }
            }
        }
    }
}
