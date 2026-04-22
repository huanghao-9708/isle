use api::{LibraryService, PersonalService, PlayerService};
use dioxus::prelude::*;
use tracing::info;
use ui::{AppRoot, LibraryProvider, PersonalProvider, PlayerProvider, STYLES_CSS};

const MAIN_CSS: Asset = asset!("/assets/main.css");

use dioxus::desktop::{Config, WindowBuilder};
use http::{Response, StatusCode};
use std::borrow::Cow;
use std::fs;

fn load_icon(bytes: &[u8]) -> dioxus::desktop::tao::window::Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::load_from_memory(bytes)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    dioxus::desktop::tao::window::Icon::from_rgba(icon_rgba, icon_width, icon_height)
        .expect("Failed to open icon")
}

fn main() {
    // 注入 Dioxus 官方日志系统
    dioxus_logger::init(tracing::Level::INFO).expect("failed to init logger");

    info!("Starting Isle Music Player (Desktop)...");

    let icon = load_icon(include_bytes!("../assets/icon.ico"));

    // 配置无边框窗口
    let cfg = Config::default()
        .with_custom_protocol("covers".to_string(), |_webview_id, request| {
            let uri = request.uri().to_string();
            let path = request.uri().path();
            // 去掉开头的 /
            let filename = path.trim_start_matches('/');
            info!(
                "[covers protocol] URI={}, path={}, filename={}",
                uri, path, filename
            );

            if let Some(mut cover_dir) = dirs::config_dir() {
                cover_dir.push("Isle");
                cover_dir.push("covers");
                cover_dir.push(filename);
                info!(
                    "[covers protocol] Trying file: {:?}, exists={}",
                    cover_dir,
                    cover_dir.exists()
                );

                if let Ok(data) = fs::read(&cover_dir) {
                    let mime = if filename.ends_with(".png") {
                        "image/png"
                    } else {
                        "image/jpeg"
                    };
                    return Response::builder()
                        .header("Content-Type", mime)
                        .header("Access-Control-Allow-Origin", "*")
                        .body(Cow::Owned(data))
                        .unwrap();
                }
            }
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Cow::Owned(vec![]))
                .unwrap()
        })
        .with_window(
            WindowBuilder::new()
                .with_title("私屿 Isle")
                .with_window_icon(Some(icon))
                .with_inner_size(dioxus::desktop::LogicalSize::new(1057.0, 752.0)) // 设置初始大小
                .with_decorations(false) // 移除原生边框
                .with_transparent(true), // 开启透明支持
        );

    LaunchBuilder::desktop().with_cfg(cfg).launch(App);
}

#[component]
fn App() -> Element {
    // 1. 初始化并注入音乐库服务
    LibraryProvider::init(|| {
        let mut svc = LibraryService::new();
        if let Some(mut config_dir) = dirs::config_dir() {
            config_dir.push("Isle");
            let _ = std::fs::create_dir_all(&config_dir);

            // 创建封面缓存目录
            let mut cover_dir = config_dir.clone();
            cover_dir.push("covers");
            let _ = std::fs::create_dir_all(&cover_dir);

            config_dir.push("library.db");
            let _ = svc.init_database(config_dir);
            svc.set_cover_dir(cover_dir);
        }
        svc
    });

    // 2. 初始化并注入播放引擎服务
    PlayerProvider::init(PlayerService::new);

    // 3. 初始化并注入私人空间服务
    let mut personal_provider = PersonalProvider::init(|| {
        let mut svc = PersonalService::new();
        if let Some(mut config_dir) = dirs::config_dir() {
            config_dir.push("Isle");
            let _ = std::fs::create_dir_all(&config_dir);
            config_dir.push("library.db");
            let _ = svc.init_database(config_dir);
        }
        svc
    });

    // 加载私人空间数据
    spawn(async move {
        personal_provider.load_all().await;
    });

    rsx! {
        // Global app resources
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: STYLES_CSS }

        AppRoot {}
    }
}
