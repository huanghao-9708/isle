use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::*;
use dioxus_free_icons::Icon;
use tracing::info;

#[derive(Clone, PartialEq, Debug)]
pub struct ScanFolder {
    pub id: String,
    pub path: String,
    pub track_count: usize,
    pub enabled: bool,
}

#[component]
pub fn FolderManager(
    folders: ReadSignal<Vec<ScanFolder>>,
    on_apply_changes: EventHandler<Vec<ScanFolder>>,
    on_close: EventHandler<()>,
) -> Element {
    // 维护一个草案列表，用户点击“确定”前所有变更仅限本地
    let mut folders_draft = use_signal(|| folders.read().clone());

    rsx! {
        div {
            class: "modal modal-open flex items-center justify-center p-4 backdrop-blur-md bg-black/40",
            onclick: move |_| on_close.call(()),

            div {
                class: "modal-box relative max-w-xl w-full bg-white shadow-2xl p-0 overflow-hidden rounded-[2rem] border border-gray-100 flex flex-col",
                onclick: move |e| e.stop_propagation(),

                // Header
                div {
                    class: "flex items-center justify-between px-8 py-6 border-b border-gray-50",
                    div {
                        class: "space-y-0.5",
                        h3 { class: "text-xl font-black tracking-tighter text-black", "库目录管理" }
                        div { class: "text-[10px] text-gray-400 font-bold uppercase tracking-widest", "Library Directories" }
                    }
                    button {
                        class: "btn btn-sm btn-circle btn-ghost text-gray-400 hover:text-black transition-colors",
                        onclick: move |_| on_close.call(()),
                        Icon { width: 20, height: 20, icon: LdX }
                    }
                }

                // Folder List Area
                div {
                    class: "px-6 py-4 flex-1 min-h-[300px] max-h-[450px] overflow-y-auto",
                    if folders_draft().is_empty() {
                        div {
                            class: "h-full flex flex-col items-center justify-center py-20 bg-gray-50/50 rounded-[1.5rem] border-2 border-dashed border-gray-100",
                            Icon { width: 48, height: 48, icon: LdFolderPlus, class: "text-gray-200 mb-4" }
                            span { class: "text-sm text-gray-400 font-bold tracking-tight", "尚未添加任何扫描目录" }
                            span { class: "text-[10px] text-gray-300 font-medium mt-1 uppercase tracking-widest", "Click 'Add Folder' to get started" }
                        }
                    } else {
                        div {
                            class: "space-y-3",
                            for (index, folder) in folders_draft().iter().enumerate() {
                                div {
                                    key: "{folder.path}", // 路径是唯一的
                                    class: "group flex items-center gap-4 p-4 bg-white hover:bg-gray-50/80 border border-gray-100 rounded-2xl transition-all duration-300 hover:shadow-sm",

                                    // Checkbox for 'Enabled'
                                    input {
                                        r#type: "checkbox",
                                        class: "checkbox checkbox-sm rounded-lg border-2 border-gray-200 checked:border-black checked:bg-black [--chkbg:black] [--chkfg:white] transition-all",
                                        checked: folder.enabled,
                                        oninput: move |e| {
                                            let mut list = folders_draft.write();
                                            if let Some(f) = list.get_mut(index) {
                                                // Dioxus 0.7 规范：checkbox 的 value 在选中时通常为 "true"
                                                f.enabled = e.value() == "true";
                                                tracing::info!("FolderManager: 切换目录状态 {} -> {}", f.path, f.enabled);
                                            }
                                        }
                                    }

                                    // Folder Path
                                    div {
                                        class: "flex-1 overflow-hidden",
                                        div { class: "text-[13px] font-bold text-gray-800 truncate leading-tight", "{folder.path}" }
                                        div { class: "text-[9px] text-gray-400 font-bold uppercase tracking-tighter mt-0.5", if folder.enabled { "Active • Ready to sync" } else { "Disabled • Ignored" } }
                                    }

                                    // Remove from Draft
                                    button {
                                        class: "btn btn-xs btn-circle btn-ghost text-gray-300 hover:text-red-500 hover:bg-red-50 transition-all opacity-0 group-hover:opacity-100",
                                        onclick: move |_| {
                                            folders_draft.write().remove(index);
                                        },
                                        Icon { width: 14, height: 14, icon: LdTrash2 }
                                    }
                                }
                            }
                        }
                    }
                }

                // Bottom Action Bar
                div {
                    class: "p-8 pt-4 border-t border-gray-50 bg-gray-50/30 flex items-center justify-between",

                    // Left: Add Folder Button
                    button {
                        class: "btn bg-white hover:bg-gray-50 text-black border-gray-200 rounded-2xl px-5 h-12 shadow-sm flex items-center gap-2 font-bold text-sm transition-all active:scale-95",
                        onclick: move |_| {
                            spawn(async move {
                                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                    let path_str = path.to_string_lossy().to_string().replace('\\', "/");
                                    // 检查是否已经在草案中
                                    if !folders_draft.read().iter().any(|f| f.path == path_str) {
                                        folders_draft.write().push(ScanFolder {
                                            id: String::new(), // 新增文件夹暂无 ID
                                            path: path_str,
                                            track_count: 0,
                                            enabled: true,
                                        });
                                    }
                                }
                            });
                        },
                        Icon { width: 18, height: 18, icon: LdFolderPlus }
                        "添加文件夹"
                    }

                    // Right: Confirm Button
                    button {
                        class: "btn bg-black hover:bg-gray-800 text-white border-none rounded-2xl px-8 h-12 shadow-lg shadow-black/10 font-bold text-sm transition-all active:scale-95",
                        onclick: move |_| {
                            info!("FolderManager: 用户点击“确定并更新”，准备提交 {} 个目录记录", folders_draft.read().len());
                            on_apply_changes.call(folders_draft());
                        },
                        "确定并更新"
                    }
                }

                // Security Hint
                div {
                    class: "px-8 py-3 bg-gray-50 flex items-center justify-center gap-2",
                    Icon { width: 10, height: 10, icon: LdShieldCheck, class: "text-gray-400" }
                    span { class: "text-[9px] text-gray-400 font-bold uppercase tracking-widest", "Local processing • Privacy guaranteed" }
                }
            }
        }
    }
}
