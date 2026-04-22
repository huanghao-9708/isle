use crate::state::PersonalProvider;
use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::*;
use dioxus_free_icons::Icon;

/// 添加到歌单组件
///
/// 展示一个图标按钮，点击后弹出当前用户的歌单列表，选择后将指定音轨添加到歌单。
#[component]
pub fn AddToPlaylist(track_id: String) -> Element {
    let personal = use_context::<PersonalProvider>();
    let mut show_menu = use_signal(|| false);
    let playlists = personal.playlists();

    rsx! {
        div { class: "relative inline-block",
            // 触发按钮
            button {
                class: "p-2 text-gray-400 hover:text-black transition-colors rounded-full hover:bg-gray-100 flex items-center justify-center",
                title: "添加到歌单",
                onclick: move |e| {
                    e.stop_propagation();
                    show_menu.toggle();
                },
                Icon { icon: LdListPlus, width: 18, height: 18 }
            }

            // 弹出菜单
            if (show_menu)() {
                // 点击外部关闭的遮罩层
                div {
                    class: "fixed inset-0 z-[100]",
                    onclick: move |e| {
                        e.stop_propagation();
                        show_menu.set(false);
                    }
                }

                // 歌单列表面板
                div {
                    class: "absolute bottom-full mb-2 right-0 w-56 bg-white rounded-2xl shadow-2xl border border-gray-100 py-2 z-[101] animate-in fade-in slide-in-from-bottom-2 duration-200 overflow-hidden",
                    // 头部
                    div { class: "px-4 py-2 border-b border-gray-50 flex items-center justify-between bg-gray-50/50",
                        span { class: "text-[10px] font-black text-gray-400 uppercase tracking-widest", "添加到歌单" }
                        button {
                            class: "p-1 hover:bg-gray-200 rounded-full transition-colors",
                            onclick: move |e| {
                                e.stop_propagation();
                                show_menu.set(false);
                            },
                            Icon { icon: LdX, class: "w-3 h-3 text-gray-400" }
                        }
                    }

                    // 列表滚动区
                    div { class: "max-h-60 overflow-y-auto custom-scrollbar",
                        if playlists.is_empty() {
                            div { class: "px-4 py-8 text-center space-y-2",
                                Icon { icon: LdListPlus, class: "w-10 h-10 text-gray-100 mx-auto" }
                                p { class: "text-[10px] text-gray-400 font-bold", "暂无歌单" }
                                p { class: "text-[9px] text-gray-300", "请在私人空间创建新歌单" }
                            }
                        } else {
                            {playlists.into_iter().map(|playlist| {
                                let tid = track_id.clone();
                                let pid = playlist.id.clone();
                                let p_provider = personal.clone();
                                rsx! {
                                    div {
                                        key: "{playlist.id}",
                                        class: "px-4 py-3 flex items-center gap-3 hover:bg-gray-50 cursor-pointer transition-colors group",
                                        onclick: move |e| {
                                            e.stop_propagation();
                                            let tid = tid.clone();
                                            let pid = pid.clone();
                                            let mut p = p_provider.clone();
                                            spawn(async move {
                                                let _ = p.add_track_to_playlist(&pid, &tid).await;
                                            });
                                            show_menu.set(false);
                                        },
                                        // 封面预览
                                        div { class: "w-8 h-8 rounded-lg bg-gray-100 flex-shrink-0 overflow-hidden shadow-sm",
                                            if let Some(cover) = &playlist.cover {
                                                img { class: "w-full h-full object-cover", src: "{cover}" }
                                            } else {
                                                div { class: "w-full h-full flex items-center justify-center",
                                                    Icon { icon: LdListMusic, class: "w-4 h-4 text-gray-300" }
                                                }
                                            }
                                        }
                                        // 名称和数量
                                        div { class: "flex flex-col min-w-0 flex-1",
                                            span {
                                                class: "text-xs font-bold text-gray-700 truncate group-hover:text-black transition-colors",
                                                "{playlist.name}"
                                            }
                                            span { class: "text-[9px] text-gray-400 font-medium", "{playlist.track_count} 首歌曲" }
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
}
