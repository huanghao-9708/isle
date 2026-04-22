use crate::state::LibraryProvider;
use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::*;
use dioxus_free_icons::Icon;

#[component]
pub fn TopNavBar(mut active_tab: Signal<String>) -> Element {
    let desktop = dioxus::desktop::use_window();
    let mut library_provider = use_context::<LibraryProvider>();
    let history = library_provider.navigation_history.read();
    let can_go_back = !history.is_empty();

    let logo_path = asset!("/assets/logo.png");

    let nav_items = vec![
        ("discover".to_string(), "发现音乐".to_string()),
        ("library".to_string(), "音乐库".to_string()),
        ("personal".to_string(), "私人空间".to_string()),
        ("settings".to_string(), "设置".to_string()),
    ];

    rsx! {
        div {
            class: "bg-[#FAFAFA] border-b border-gray-200/40 px-8 py-2.5 sticky top-0 z-50 flex items-center justify-between",
            onmousedown: {
                let desktop = desktop.clone();
                move |_| { desktop.drag(); }
            },

            div {
                class: "flex items-center gap-6 group cursor-default select-none",
                onmousedown: move |e| e.stop_propagation(),

                // 全局后退按钮
                button {
                    class: format!(
                        "w-8 h-8 flex items-center justify-center rounded-lg transition-all {}",
                        if can_go_back {
                            "bg-black/5 text-black hover:bg-black hover:text-white"
                        } else {
                            "text-gray-200 cursor-not-allowed"
                        }
                    ),
                    onclick: move |_| {
                        if can_go_back {
                            library_provider.navigate_back();
                        }
                    },
                    Icon { width: 14, height: 14, icon: LdChevronLeft }
                }

                div {
                    class: "flex items-center gap-3",
                    div {
                        class: "w-8 h-8 rounded-full overflow-hidden bg-black/5 flex items-center justify-center shadow-sm transition-transform duration-500 hover:rotate-6 border border-gray-100",
                        img { src: "{logo_path}", class: "w-full h-full object-cover" }
                    }
                    div {
                        class: "flex flex-col -space-y-1",
                        span { class: "text-base font-black text-black tracking-tight", "私屿" }
                        span { class: "text-[9px] text-gray-400 font-bold uppercase tracking-widest", "Isle Music" }
                    }
                }
            }

            div {
                class: "flex gap-8 items-end",
                onmousedown: move |e| e.stop_propagation(),
                {nav_items.iter().map(|(id, label)| {
                    let is_active = *active_tab.read() == *id;
                    let id_clone = id.clone();
                    let label_display = label.as_str();
                    rsx! {
                        button {
                            key: "{id}",
                            class: format!(
                                "nav-btn text-sm font-bold tracking-tight transition-colors duration-300 {}",
                                if is_active { "active text-black" } else { "text-gray-400 hover:text-black" }
                            ),
                            onclick: move |_| {
                                library_provider.active_global_tab.set(id_clone.clone());
                                // 切换全局 Tab 时，自动重置子视图
                                match id_clone.as_str() {
                                    "library" => library_provider.current_view.set(crate::state::library::LibraryView::Main),
                                    "personal" => library_provider.current_view.set(crate::state::library::LibraryView::PersonalOverview),
                                    _ => {}
                                }
                            },
                            "{label_display}"
                        }
                    }
                })}
            }

            div {
                class: "flex items-center gap-1.5",
                onmousedown: move |e| e.stop_propagation(),

                button {
                    class: "w-8 h-8 flex items-center justify-center rounded-md text-gray-400 hover:bg-black/[0.05] hover:text-black transition-all",
                    onclick: {
                        let desktop = desktop.clone();
                        move |_| { desktop.set_minimized(true); }
                    },
                    Icon { width: 14, height: 14, icon: LdMinus }
                }

                button {
                    class: "w-8 h-8 flex items-center justify-center rounded-md text-gray-400 hover:bg-black/[0.05] hover:text-black transition-all",
                    onclick: {
                        let desktop = desktop.clone();
                        move |_| {
                            let is_max = desktop.is_maximized();
                            desktop.set_maximized(!is_max);
                        }
                    },
                    Icon { width: 12, height: 12, icon: LdSquare }
                }

                button {
                    class: "w-8 h-8 flex items-center justify-center rounded-md text-gray-400 hover:bg-red-500 hover:text-white transition-all",
                    onclick: move |_| { desktop.close(); },
                    Icon { width: 14, height: 14, icon: LdX }
                }
            }
        }
    }
}
