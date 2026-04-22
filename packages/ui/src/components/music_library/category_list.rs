use crate::state::LibraryProvider;
use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::*;
use dioxus_free_icons::Icon;

#[derive(Clone, PartialEq, Props)]
pub struct CategoryListProps {
    pub category: String, // "artists", "albums", "genres", "folders"
    pub on_select: EventHandler<String>,
}

#[component]
pub fn CategoryList(props: CategoryListProps) -> Element {
    let library = use_context::<LibraryProvider>();

    // 如果分类是文件夹，展示特殊的层级浏览器
    if props.category == "folders" {
        return rsx! {
            crate::components::music_library::FolderHierarchyBrowser {}
        };
    }

    let items: Vec<(String, String, i32, Option<String>, String, Option<i32>)> =
        match props.category.as_str() {
            "artists" => library
                .artists
                .read()
                .iter()
                .map(|a| {
                    (
                        a.name.clone(),
                        "".to_string(),
                        a.track_count,
                        a.cover_path.clone(),
                        a.id.clone(),
                        None,
                    )
                })
                .collect(),
            "albums" => library
                .albums
                .read()
                .iter()
                .map(|a| {
                    (
                        a.title.clone(),
                        a.artist_name.clone(),
                        a.track_count,
                        a.cover_path.clone(),
                        a.id.clone(),
                        None,
                    )
                })
                .collect(),
            "genres" => library
                .genres
                .read()
                .iter()
                .map(|g| {
                    (
                        g.name.clone(),
                        "".to_string(),
                        g.track_count,
                        g.image.clone(),
                        "".to_string(),
                        Some(g.id),
                    )
                })
                .collect(),
            "folders" => library
                .folder_summaries
                .read()
                .iter()
                .map(|f| {
                    (
                        f.path.clone(),
                        "".to_string(),
                        f.track_count,
                        None,
                        f.path.clone(),
                        None,
                    )
                })
                .collect(),
            _ => Vec::new(),
        };
    let items_count = items.len();

    rsx! {
        div {
            class: "grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6 p-8",

            {items.into_iter().map(|(name, sub, count, cover_path, id, genre_id)| {
                let display_name = name.clone();
                let display_sub = sub.clone();
                let on_select = props.on_select.clone();
                let library = library;
                let category = props.category.clone();
                let _cover_dir = library.cover_dir.read().clone();

                // 强制使用基于 ID 的唯一 Key，防止同名导致的崩溃
                let item_key = if category == "genres" {
                    genre_id.map(|gi| gi.to_string()).unwrap_or_else(|| name.clone())
                } else if !id.is_empty() {
                    id.clone()
                } else {
                    name.clone()
                };

                rsx! {
                    div {
                        key: "{item_key}",
                        class: "group bg-white rounded-xl p-6 border border-gray-100 shadow-sm hover:shadow-md hover:border-black/10 transition-all cursor-pointer flex flex-col items-center text-center gap-4",
                        onclick: move |_| {
                            let category = category.clone();
                            let mut library = library;
                            let id = id.clone();
                            let genre_id = genre_id;

                            match category.as_str() {
                                "artists" => {
                                    spawn(async move {
                                        library.navigate_to_artist(id).await;
                                    });
                                }
                                "albums" => {
                                    spawn(async move {
                                        library.navigate_to_album(id).await;
                                    });
                                }
                                "genres" => {
                                    if let Some(gid) = genre_id {
                                        spawn(async move {
                                            library.navigate_to_genre(gid).await;
                                        });
                                    }
                                }
                                _ => on_select.call(name.clone()),
                            }
                        },

                        // Icon Circle or Cover
                        div {
                            class: "w-16 h-16 bg-[#F0F1F3] rounded-full flex items-center justify-center text-gray-400 group-hover:bg-black group-hover:text-white transition-colors overflow-hidden",
                            if let Some(cover_filename) = cover_path {
                                {
                                    let url = format!("http://covers.localhost/{}", cover_filename);
                                    rsx! {
                                        img { src: "{url}", class: "w-full h-full object-cover" }
                                    }
                                }
                            } else {
                                match category.as_str() {
                                    "artists" => rsx! { Icon { width: 28, height: 28, icon: LdUser } },
                                    "albums" => rsx! { Icon { width: 28, height: 28, icon: LdDisc } },
                                    "genres" => rsx! { Icon { width: 28, height: 28, icon: LdTag } },
                                    "folders" => rsx! { Icon { width: 28, height: 28, icon: LdFolder } },
                                    _ => rsx! { Icon { width: 28, height: 28, icon: LdBox } },
                                }
                            }
                        }

                        // Info
                        div {
                            class: "space-y-1 w-full",
                            div {
                                class: "font-bold text-sm text-black truncate px-2",
                                "{display_name}"
                            }
                            if !display_sub.is_empty() {
                                div {
                                    class: "text-[10px] text-gray-400 uppercase tracking-tight truncate",
                                    "{display_sub}"
                                }
                            }
                            div {
                                class: "text-[10px] font-bold text-gray-300 group-hover:text-black transition-colors uppercase tracking-widest pt-2",
                                "{count} TRACKS"
                            }
                        }
                    }
                }
            })}

            if items_count == 0 {
                div {
                    class: "col-span-full py-20 flex flex-col items-center justify-center text-gray-300 gap-4",
                    Icon { width: 48, height: 48, icon: LdBox, class: "opacity-20" }
                    span { class: "text-xs font-bold uppercase tracking-widest", "暂无内容" }
                }
            }
        }
    }
}
