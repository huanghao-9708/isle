use crate::components::music_library::TrackTable;
use crate::state::LibraryProvider;
use api::models::TrackFilter;
use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::*;
use dioxus_free_icons::Icon;
use std::path::Path;

#[component]
pub fn FolderHierarchyBrowser() -> Element {
    let library = use_context::<LibraryProvider>();
    let mut current_path = use_signal(|| None::<String>);

    // 计算当前层级的子文件夹
    let sub_folders = use_memo(move || {
        let all_tracks = library.all_api_tracks.read();
        let mut subs = std::collections::HashSet::new();

        match current_path() {
            None => {
                // 初始层级：显示所有扫描根目录
                for f in library.folders.read().iter() {
                    subs.insert(f.path.clone());
                }
            }
            Some(ref prefix) => {
                let prefix_path = Path::new(prefix);
                for track in all_tracks.iter() {
                    let track_path = Path::new(&track.path);
                    if track_path.starts_with(prefix_path) {
                        // 找到相对于当前前缀的剩余路径
                        if let Ok(remainder) = track_path.strip_prefix(prefix_path) {
                            if let Some(first_segment) = remainder.components().next() {
                                if let std::path::Component::Normal(os_str) = first_segment {
                                    // 如果这个片段不是文件本身（即它是一个目录）
                                    if remainder.components().count() > 1 {
                                        let full_sub_path =
                                            prefix_path.join(os_str).to_string_lossy().to_string();
                                        subs.insert(full_sub_path);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut sorted_subs: Vec<_> = subs.into_iter().collect();
        sorted_subs.sort();
        sorted_subs
    });

    // 计算面包屑
    let breadcrumbs = use_memo(move || {
        let mut crumbs = Vec::new();
        if let Some(path_str) = current_path() {
            let path = Path::new(&path_str);
            let mut current = path;
            while let Some(parent) = current.parent() {
                if current == parent || parent.as_os_str().is_empty() {
                    break;
                }
                crumbs.push((
                    current
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    current.to_string_lossy().to_string(),
                ));
                current = parent;
            }
            // 处理根盘符或根目录
            crumbs.push((
                current.to_string_lossy().to_string(),
                current.to_string_lossy().to_string(),
            ));
            crumbs.reverse();
        }
        crumbs
    });

    // 计算当前目录下（包含子级）的所有轨道的ID集合，供“批量添加到歌单”使用
    let full_folder_track_ids = use_memo(move || {
        let all_tracks = library.all_api_tracks.read();
        let mut ids = Vec::new();
        if let Some(prefix) = current_path() {
            let prefix_path = Path::new(&prefix);
            for track in all_tracks.iter() {
                let track_path = Path::new(&track.path);
                if track_path.starts_with(prefix_path) {
                    ids.push(track.id.clone());
                }
            }
        } else {
            ids = all_tracks.iter().map(|t| t.id.clone()).collect();
        }
        ids
    });

    let mut current_page = use_signal(|| 1usize);
    let page_size = 20;

    // 当路径变化时重置页码
    use_effect(move || {
        let _ = current_path();
        current_page.set(1);
    });

    // 局部获取分页数据资源，取代更新全局 library_filter
    let tracks_resource = use_resource(move || {
        let path = current_path();
        let page = current_page();
        let lib = library;
        async move {
            let filter = TrackFilter {
                folder_path_prefix: path,
                ..Default::default()
            };
            lib.filter_tracks_paginated(filter, page, page_size).await
        }
    });

    let (tracks, api_tracks_raw, total_count) = if let Some(Ok(res)) = &*tracks_resource.read() {
        let items = res.items.clone();
        (
            items
                .iter()
                .map(|t| crate::components::types::Track::from(t.clone()))
                .collect::<Vec<_>>(),
            items,
            res.total,
        )
    } else {
        (Vec::new(), Vec::new(), 0)
    };

    let total_pages = (total_count + page_size - 1) / page_size;

    rsx! {
        div { class: "flex flex-col h-full bg-white animate-in fade-in duration-500",

            // --- 顶部导航栏 (面包屑) ---
            div { class: "px-8 py-4 border-b border-gray-50 flex items-center gap-2 overflow-x-auto whitespace-nowrap scrollbar-hide",

                // 批量添加到歌单
                {
                    let track_ids = full_folder_track_ids();
                    rsx! {
                        crate::components::personal::BatchAddToPlaylist { track_ids }
                    }
                }

                div { class: "w-[1px] h-3 bg-gray-100 mx-2" }

                button {
                    class: "flex items-center gap-1.5 text-xs font-black uppercase tracking-widest transition-colors hover:text-black "
                        .to_string() + if current_path().is_none() { "text-black" } else { "text-gray-300" },
                    onclick: move |_| current_path.set(None),
                    Icon { width: 14, height: 14, icon: LdHome }
                    "所有目录"
                }

                for (name, path) in breadcrumbs() {
                    Icon { width: 10, height: 10, icon: LdChevronRight, class: "text-gray-200" }
                    button {
                        class: "text-xs font-black transition-colors hover:text-black text-gray-400",
                        onclick: {
                            let p = path.clone();
                            move |_| current_path.set(Some(p.clone()))
                        },
                        "{name}"
                    }
                }
            }

            // --- 内容区 ---
            div { class: "flex-1 overflow-y-auto pb-20 custom-scrollbar",

                // 1. 子文件夹网格
                if !sub_folders().is_empty() {
                    div { class: "grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-6 p-8",
                        for sub_path in sub_folders() {
                            {
                                let path_obj = Path::new(&sub_path);
                                let folder_name = path_obj.file_name().and_then(|n| n.to_str()).unwrap_or(&sub_path).to_string();
                                let p = sub_path.clone();
                                rsx! {
                                    div {
                                        key: "{sub_path}",
                                        class: "group cursor-pointer flex flex-col items-center gap-3 p-4 rounded-[2rem] hover:bg-gray-50 transition-all",
                                        onclick: move |_| current_path.set(Some(p.clone())),

                                        div { class: "w-20 h-20 bg-gray-50 rounded-[1.5rem] flex items-center justify-center text-gray-200 group-hover:bg-black group-hover:text-white transition-all group-hover:shadow-xl group-hover:shadow-black/10 group-hover:-translate-y-1",
                                            Icon { width: 32, height: 32, icon: LdFolder }
                                        }

                                        span { class: "text-[11px] font-black text-center text-gray-400 group-hover:text-black truncate w-full px-2",
                                            "{folder_name}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "px-8", div { class: "h-[1px] bg-gray-50 w-full" } }
                }

                // 2. 歌曲列表 (递归展示)
                div { class: "p-4",
                    div { class: "px-4 py-2 flex items-center justify-between mb-4",
                        h4 { class: "text-xs font-black text-gray-400 uppercase tracking-[0.2em]",
                            if current_path().is_none() { "库中所有音乐" } else { "目录下的音乐" }
                        }
                        div { class: "flex items-center gap-4",
                            span { class: "text-[10px] bg-gray-100 text-gray-400 px-3 py-1 rounded-full font-bold",
                                "{total_count} 首"
                            }

                            // 分页控制
                            if total_pages > 0 {
                                div { class: "flex items-center gap-2",
                                    button {
                                        class: "p-1 hover:bg-gray-100 rounded-lg transition-all disabled:opacity-30 disabled:cursor-not-allowed",
                                        disabled: current_page() <= 1,
                                        onclick: move |_| current_page.set(current_page() - 1),
                                        Icon { width: 14, height: 14, icon: LdChevronLeft, class: "text-gray-500" }
                                    }
                                    span { class: "text-[10px] font-black text-gray-400",
                                        "{current_page()} / {total_pages.max(1)}"
                                    }
                                    button {
                                        class: "p-1 hover:bg-gray-100 rounded-lg transition-all disabled:opacity-30 disabled:cursor-not-allowed",
                                        disabled: current_page() >= total_pages,
                                        onclick: move |_| current_page.set(current_page() + 1),
                                        Icon { width: 14, height: 14, icon: LdChevronRight, class: "text-gray-500" }
                                    }
                                }
                            }
                        }
                    }
                    if tracks_resource.read().is_none() {
                        div { class: "py-20 flex justify-center",
                            span { class: "text-xs text-gray-300 font-black animate-pulse", "读取中..." }
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
}
