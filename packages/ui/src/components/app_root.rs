use crate::components::layout::TopNavBar;
use crate::components::music_library_page::MusicLibraryPage;
use crate::components::personal_space_page::PersonalSpacePage;
use crate::components::player::PlayerBar;
use crate::state::LibraryProvider;
use dioxus::prelude::*;

#[component]
pub fn AppRoot() -> Element {
    let provider = use_context::<LibraryProvider>();
    let active_tab = provider.active_global_tab;

    rsx! {
        div {
            class: "flex flex-col h-screen bg-[#FAFAFA] text-black font-sans selection:bg-black selection:text-white",

            TopNavBar { active_tab: active_tab }

            div {
                class: "flex-1 flex flex-col overflow-hidden",
                if active_tab() == "library" {
                    MusicLibraryPage {}
                } else if active_tab() == "personal" {
                    PersonalSpacePage {}
                } else {
                    div { class: "flex items-center justify-center h-full",
                        div { class: "text-gray-400",
                            "即将上线..."
                        }
                    }
                }
            }

            PlayerBar {}

            crate::components::player::PlaylistPanel {}
            crate::components::player::ImmersivePlayer {}
        }
    }
}
