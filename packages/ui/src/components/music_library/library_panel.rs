use crate::components::music_library::folder_manager::FolderManager;
use crate::state::LibraryProvider;
use dioxus::prelude::*;

/// 音乐库管理面板（智能组件）
///
/// 该组件负责从全局 Context 获取 LibraryProvider，
/// 并将其逻辑注入到现有的 UI 组件中。
#[component]
pub fn LibraryPanel(
    on_apply: EventHandler<Vec<crate::components::music_library::folder_manager::ScanFolder>>,
    on_close: EventHandler<()>,
) -> Element {
    let provider = use_context::<LibraryProvider>();
    let folders = provider.folders;
    let _is_scanning = (provider.is_scanning)();

    // 将后端模型转换为 UI 组件需要的模型
    let ui_folders = use_memo(move || {
        folders()
            .iter()
            .map(
                |f| crate::components::music_library::folder_manager::ScanFolder {
                    id: f.id.clone(),
                    path: f.path.clone(),
                    track_count: 0,
                    enabled: f.enabled,
                },
            )
            .collect::<Vec<_>>()
    });

    rsx! {
        FolderManager {
            folders: ui_folders,
            on_apply_changes: move |new_folders| {
                on_apply.call(new_folders);
            },
            on_close: on_close
        }
    }
}
