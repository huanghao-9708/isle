use crate::components::personal::PlaylistEditor;
use crate::state::library::LibraryView;
use crate::state::{LibraryProvider, PersonalProvider};
use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::*;
use dioxus_free_icons::Icon;

#[component]
pub fn PlaylistManager() -> Element {
    let mut library = use_context::<LibraryProvider>();
    let personal = use_context::<PersonalProvider>();
    let playlists = personal.playlists();
    let current_view = library.current_view.read().clone();

    // 编辑器状态
    let mut show_editor = use_signal(|| false);
    let mut editing_playlist = use_signal(|| None::<api::models::UserPlaylist>);

    // 基于全局路由判断当前状态
    let is_creating = matches!(current_view, LibraryView::PersonalPlaylistsCreate);

    // 如果路由要求创建，且编辑器未开启，则开启
    use_effect(move || {
        if is_creating {
            editing_playlist.set(None);
            show_editor.set(true);
        }
    });

    rsx! {
        div { class: "w-full animate-in fade-in duration-500",
            // 编辑器弹窗
            if (show_editor)() {
                PlaylistEditor {
                    playlist: (editing_playlist)(),
                    on_close: move |_| {
                        show_editor.set(false);
                        if is_creating { library.navigate_back(); }
                    },
                    on_save: move |_| {
                        show_editor.set(false);
                        if is_creating { library.navigate_back(); }
                    },
                }
            }

            // 网格视图
            PlaylistGridView {
                playlists: playlists.clone(),
                on_select: move |id| {
                    if let Some(p) = playlists.iter().find(|pl| pl.id == id) {
                        library.navigate_to_playlist(p.clone());
                    }
                },
                on_create: move |_| {
                    editing_playlist.set(None);
                    show_editor.set(true);
                }
            }
        }
    }
}

#[component]
fn PlaylistGridView(
    playlists: Vec<api::models::UserPlaylist>,
    on_select: EventHandler<String>,
    on_create: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "space-y-8",
            // 头部
            div { class: "flex items-center gap-4",
                h2 { class: "text-3xl font-black tracking-tight", "全部歌单" }
            }

            // 网格
            div { class: "grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-8",

                // 创建入口卡片
                div {
                    class: "group cursor-pointer space-y-4",
                    onclick: move |_| on_create.call(()),
                    div {
                        class: "aspect-square rounded-[2.5rem] bg-white border-2 border-dashed border-gray-200 flex flex-col items-center justify-center gap-4 text-gray-400 group-hover:border-black group-hover:text-black transition-all duration-300 shadow-sm hover:shadow-xl",
                        Icon { icon: LdPlus, class: "w-12 h-12" }
                        span { class: "text-sm font-bold", "创建新歌单" }
                    }
                }

                // 歌单卡片列表
                {playlists.into_iter().map(|playlist| {
                    let id = playlist.id.clone();
                    rsx! {
                        div {
                            key: "{id}",
                            class: "group cursor-pointer space-y-4",
                            onclick: move |_| on_select.call(id.clone()),

                            // 封面容器
                            div {
                                class: "aspect-square rounded-[2.5rem] bg-white border border-gray-100 overflow-hidden shadow-sm group-hover:shadow-2xl group-hover:-translate-y-2 transition-all duration-500",
                                if let Some(cover) = &playlist.cover {
                                    img { class: "w-full h-full object-cover", src: "http://covers.localhost/{cover}" }
                                } else {
                                    div { class: "w-full h-full flex items-center justify-center bg-gray-50",
                                        Icon { icon: LdListMusic, class: "w-16 h-16 text-gray-100" }
                                    }
                                }
                            }

                            // 信息
                            div { class: "px-2 space-y-0.5",
                                div { class: "font-black text-lg truncate text-black", "{playlist.name}" }
                                div { class: "text-sm text-gray-400 font-bold", "{playlist.track_count} 首歌曲" }
                            }
                        }
                    }
                })}
            }
        }
    }
}
