use dioxus::prelude::*;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        // Global app resources
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        div {
            class: "min-h-screen bg-gradient-to-br from-gray-900 to-black flex items-center justify-center p-8",
            div { class: "bg-white/10 backdrop-blur-xl p-10 rounded-3xl border border-white/20 text-center space-y-4",
                h1 { class: "text-4xl font-black text-white", "私屿 Isle" }
                p { class: "text-gray-300 font-medium", "Web 版本正在努力开发中，请下载桌面客户端体验完整功能。" }
            }
        }
    }
}
