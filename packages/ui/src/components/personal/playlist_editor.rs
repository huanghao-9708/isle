use crate::state::PersonalProvider;
use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::*;
use dioxus_free_icons::Icon;

#[component]
pub fn PlaylistEditor(
    playlist: Option<api::models::UserPlaylist>,
    on_close: EventHandler<()>,
    on_save: EventHandler<()>,
) -> Element {
    let personal = use_context::<PersonalProvider>();
    let mut name = use_signal(|| {
        playlist
            .as_ref()
            .map(|p| p.name.clone())
            .unwrap_or_default()
    });
    let mut description = use_signal(|| {
        playlist
            .as_ref()
            .map(|p| p.description.clone())
            .unwrap_or_default()
    });
    let mut tags_str = use_signal(|| {
        playlist
            .as_ref()
            .map(|p| p.tags.join(", "))
            .unwrap_or_default()
    });
    let mut is_saving = use_signal(|| false);

    let is_edit = playlist.is_some();

    rsx! {
        div {
            class: "fixed inset-0 bg-black/40 backdrop-blur-sm z-[1000] flex items-center justify-center p-4 animate-in fade-in duration-300",
            onclick: move |_| on_close.call(()),

            div {
                class: "bg-white rounded-[2.5rem] shadow-2xl w-full max-w-lg overflow-hidden animate-in zoom-in-95 slide-in-from-bottom-10 duration-500",
                onclick: move |e| e.stop_propagation(),

                // 头部
                div { class: "px-10 py-8 flex items-center justify-between",
                    h3 { class: "text-2xl font-black tracking-tight", if is_edit { "编辑歌单资料" } else { "创建新歌单" } }
                    button {
                        class: "p-2 hover:bg-gray-100 rounded-full transition-all text-gray-400 hover:text-black",
                        onclick: move |_| on_close.call(()),
                        Icon { icon: LdX, class: "w-6 h-6" }
                    }
                }

                // 表单内容
                div { class: "px-10 pb-10 space-y-8",
                    // 歌单名称
                    div { class: "space-y-3",
                        label { class: "block text-sm font-bold text-gray-700",
                            "歌单名称 "
                            span { class: "text-red-500", "*" }
                        }
                        input {
                            class: "w-full px-6 py-4 bg-gray-50 border-2 border-transparent focus:border-black/5 focus:bg-white rounded-2xl outline-none transition-all font-medium text-lg placeholder:text-gray-300",
                            placeholder: "例如：周五晚上的爵士乐",
                            value: "{name}",
                            oninput: move |e| name.set(e.value()),
                            autofocus: true,
                        }
                    }

                    // 简介
                    div { class: "space-y-3",
                        label { class: "block text-sm font-bold text-gray-700", "简介" }
                        textarea {
                            class: "w-full px-6 py-4 bg-gray-50 border-2 border-transparent focus:border-black/5 focus:bg-white rounded-2xl outline-none transition-all font-medium min-h-[120px] resize-none placeholder:text-gray-300",
                            placeholder: "添加一些关于这个歌单的描述...",
                            value: "{description}",
                            oninput: move |e| description.set(e.value()),
                        }
                    }

                    // 标签
                    div { class: "space-y-3",
                        label { class: "block text-sm font-bold text-gray-700", "标签" }
                        input {
                            class: "w-full px-6 py-4 bg-white border-2 border-gray-900 rounded-2xl outline-none font-medium placeholder:text-gray-400",
                            placeholder: "用逗号分隔，例如：流行，电子",
                            value: "{tags_str}",
                            oninput: move |e| tags_str.set(e.value()),
                        }
                    }
                }

                // 底部操作栏
                div { class: "px-10 py-8 flex items-center justify-end gap-8 bg-gray-50/50",
                    button {
                        class: "text-sm font-bold text-gray-500 hover:text-black transition-colors",
                        onclick: move |_| on_close.call(()),
                        "取消"
                    }
                    button {
                        class: format!(
                            "px-10 py-4 bg-black text-white rounded-3xl text-sm font-bold hover:scale-105 active:scale-95 transition-all flex items-center justify-center gap-3 shadow-lg shadow-black/10 {}",
                            if (is_saving)() || (name)().is_empty() { "opacity-50 pointer-events-none" } else { "" }
                        ),
                        onclick: move |_| {
                            let mut p = personal.clone();
                            let n = (name)();
                            let d = (description)();
                            let t = (tags_str)().split(&[',', '，'][..])
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty())
                                .collect::<Vec<_>>();
                            let pl_opt = playlist.clone();

                            spawn(async move {
                                is_saving.set(true);
                                if let Some(pl) = pl_opt {
                                    let _ = p.update_playlist(&pl.id, &n, &d, t).await;
                                } else {
                                    let _ = p.create_playlist(&n, &d, t).await;
                                }
                                is_saving.set(false);
                                on_save.call(());
                            });
                        },
                        if (is_saving)() {
                            div { class: "w-4 h-4 border-2 border-white/20 border-t-white rounded-full animate-spin" }
                            "正在保存..."
                        } else {
                            if is_edit { "保存修改" } else { "创建歌单" }
                        }
                    }
                }
            }
        }
    }
}
