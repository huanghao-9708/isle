use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::*;
use dioxus_free_icons::Icon;

/// 统计卡片的样式常量
const CARD_CLASS: &str =
    "bg-[#F0F1F3] rounded-md p-5 hover:bg-[#EAEBED] transition-all duration-200 group";
const ICON_WRAP_CLASS: &str = "w-11 h-11 bg-black rounded-full flex items-center justify-center text-white transition-transform group-hover:scale-110";

/// 统计看板组件 —— 展示歌曲、专辑、艺术家、流派、文件夹的数量统计
///
/// # 参数
/// - `track_count`: 歌曲总数
/// - `album_count`: 专辑数量
/// - `artist_count`: 艺术家数量
/// - `genre_count`: 流派数量
/// - `folder_count`: 文件夹数量
#[component]
pub fn StatsBoard(
    track_count: Signal<usize>,
    album_count: Signal<usize>,
    artist_count: Signal<usize>,
    genre_count: Signal<usize>,
    folder_count: Signal<usize>,
) -> Element {
    // 不同图标结构体为不同 Rust 类型，无法放入同一数组，因此逐一写出每个统计卡片
    rsx! {
        div {
            class: "grid grid-cols-5 gap-4",

            // 歌曲总数
            StatCard { label: "歌曲总数", value: track_count, icon: rsx! { Icon { width: 22, height: 22, icon: LdMusic } } }
            // 专辑
            StatCard { label: "专辑", value: album_count, icon: rsx! { Icon { width: 22, height: 22, icon: LdDisc } } }
            // 艺术家
            StatCard { label: "艺术家", value: artist_count, icon: rsx! { Icon { width: 22, height: 22, icon: LdUser } } }
            // 流派数量
            StatCard { label: "流派数量", value: genre_count, icon: rsx! { Icon { width: 22, height: 22, icon: LdLibrary } } }
            // 文件夹
            StatCard { label: "文件夹", value: folder_count, icon: rsx! { Icon { width: 22, height: 22, icon: LdFolder } } }
        }
    }
}

/// 单个统计卡片组件
///
/// # 参数
/// - `label`: 统计项名称，如 "歌曲总数"
/// - `value`: 统计值
/// - `icon`: 图标元素（通过 Element 传入以解决不同图标类型不兼容的问题）
#[component]
fn StatCard(label: &'static str, value: Signal<usize>, icon: Element) -> Element {
    rsx! {
        div {
            class: CARD_CLASS,
            div {
                class: "flex items-center gap-4",
                div {
                    class: ICON_WRAP_CLASS,
                    {icon}
                }
                div {
                    div {
                        class: "text-xs text-gray-400 font-medium mb-0.5",
                        "{label}"
                    }
                    div {
                        class: "text-xl font-bold text-black",
                        "{value}"
                    }
                }
            }
        }
    }
}
