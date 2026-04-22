use crate::state::PersonalProvider;
use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::*;
use dioxus_free_icons::Icon;

/// 批量添加到歌单组件
///
/// 展示一个图标按钮，点击后弹出模态对话框，展示歌单列表。
#[component]
pub fn BatchAddToPlaylist(track_ids: Vec<String>) -> Element {
    let personal = use_context::<PersonalProvider>();
    let mut show_modal = use_signal(|| false);
    let playlists = personal.playlists();

    rsx! {
        div { class: "relative inline-block",
            // 触发按钮
            button {
                class: "flex items-center gap-1.5 p-1.5 text-gray-400 hover:text-black transition-colors rounded-md hover:bg-gray-100 flex items-center justify-center",
                title: "将本目录下所有音乐添加至歌单",
                onclick: move |e| {
                    e.stop_propagation();
                    show_modal.set(true);
                },
                Icon { icon: LdListPlus, width: 14, height: 14 }
                span { class: "text-[10px] font-black uppercase tracking-tight", "添加至歌单" }
            }

            // 模态对话框
            if (show_modal)() {
                div {
                    class: "fixed inset-0 z-[1000] flex items-center justify-center p-4",

                    // 背景遮罩
                    div {
                        class: "absolute inset-0 bg-black/40 backdrop-blur-sm animate-in fade-in duration-300",
                        onclick: move |e| {
                            e.stop_propagation();
                            show_modal.set(false);
                        }
                    }

                    // 对话框面板
                    div {
                        class: "relative w-full max-w-sm bg-white rounded-[2.5rem] shadow-2xl overflow-hidden animate-in zoom-in-95 fade-in duration-300",

                        // 头部
                        div { class: "px-8 py-6 border-b border-gray-50 flex items-center justify-between",
                            div { class: "flex flex-col",
                                span { class: "text-[10px] font-black text-gray-300 uppercase tracking-[0.2em] leading-none mb-1", "Add to Playlist" }
                                h3 { class: "text-lg font-black text-black tracking-tighter", "添加到歌单" }
                            }
                            button {
                                class: "w-10 h-10 bg-gray-50 hover:bg-gray-100 text-gray-400 hover:text-black rounded-full transition-all flex items-center justify-center active:scale-90",
                                onclick: move |e| {
                                    e.stop_propagation();
                                    show_modal.set(false);
                                },
                                Icon { icon: LdX, class: "w-5 h-5" }
                            }
                        }

                        // 歌单选择区域
                        div { class: "max-h-[60vh] overflow-y-auto custom-scrollbar p-6",
                            if playlists.is_empty() {
                                div { class: "py-20 flex flex-col items-center justify-center text-center gap-4",
                                    div { class: "w-16 h-16 bg-gray-50 rounded-2xl flex items-center justify-center text-gray-200",
                                        Icon { icon: LdListPlus, width: 32, height: 32 }
                                    }
                                    div { class: "space-y-1",
                                        p { class: "text-sm font-black text-gray-400", "暂无歌单" }
                                        p { class: "text-[10px] text-gray-300 font-bold uppercase tracking-widest", "请先在私人空间创建歌单" }
                                    }
                                }
                            } else {
                                div { class: "space-y-2",
                                    {playlists.into_iter().map(|playlist| {
                                        let tids = track_ids.clone();
                                        let pid = playlist.id.clone();
                                        let p_provider = personal.clone();

                                        // 缩略图路径处理
                                        let cover_url = playlist.cover.as_ref().map(|c| format!("http://covers.localhost/{}", c));

                                        rsx! {
                                            div {
                                                key: "{playlist.id}",
                                                class: "group flex items-center gap-4 p-4 rounded-[1.5rem] hover:bg-gray-50 cursor-pointer transition-all active:scale-[0.98]",
                                                onclick: move |e| {
                                                    e.stop_propagation();
                                                    let tids = tids.clone();
                                                    let pid = pid.clone();
                                                    let mut p = p_provider.clone();
                                                    spawn(async move {
                                                        let _ = p.add_track_to_playlist_batch(&pid, tids).await;
                                                    });
                                                    show_modal.set(false);
                                                },

                                                // 缩略图
                                                div { class: "w-14 h-14 bg-gray-100 rounded-xl flex-shrink-0 overflow-hidden shadow-sm group-hover:shadow-md transition-shadow",
                                                    if let Some(url) = cover_url {
                                                        img {
                                                            class: "w-full h-full object-cover",
                                                            src: "{url}"
                                                        }
                                                    } else {
                                                        div { class: "w-full h-full flex items-center justify-center text-gray-300",
                                                            Icon { icon: LdListMusic, width: 24, height: 24 }
                                                        }
                                                    }
                                                }

                                                // 名称和曲目数
                                                div { class: "flex flex-col min-w-0 flex-1",
                                                    span {
                                                        class: "text-sm font-black text-gray-700 truncate group-hover:text-black transition-colors leading-tight",
                                                        "{playlist.name}"
                                                    }
                                                    span { class: "text-[10px] text-gray-400 font-bold uppercase tracking-widest mt-1",
                                                        "{playlist.track_count} TRACKS"
                                                    }
                                                }

                                                // 悬浮显示的快捷箭头
                                                div { class: "opacity-0 group-hover:opacity-100 transition-opacity pr-2",
                                                    Icon { icon: LdPlus, class: "w-4 h-4 text-black" }
                                                }
                                            }
                                        }
                                    })}
                                }
                            }
                        }

                        // 底部提示
                        div { class: "px-8 py-4 bg-gray-50 text-center",
                            p { class: "text-[9px] font-bold text-gray-300 uppercase tracking-[0.2em]",
                                "选中的目录共包含 {track_ids.len()} 首音乐"
                            }
                        }
                    }
                }
            }
        }
    }
}
