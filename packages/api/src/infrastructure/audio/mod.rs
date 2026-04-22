use rodio::{Decoder, OutputStream, Sink, Source};
use std::fs::File;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tracing::{error, info};

/// 音频引擎命令枚举
///
/// 用于从业务层向音频播放线程发送控制指令。
pub enum AudioCommand {
    /// 加载指定路径的音频文件并开始播放
    Load(String),
    /// 恢复当前暂停的播放
    Play,
    /// 暂停当前播放
    Pause,
    /// 停止播放并释放资源
    Stop,
    /// 跳转到指定秒数位置
    Seek(u32),
    /// 设置音量
    SetVolume(u8),
}

/// 音频引擎状态快照
///
/// 从音频线程同步回来的只读状态，供 Service 层读取。
#[derive(Clone, Debug)]
pub struct AudioStatus {
    /// 当前播放进度（秒）
    pub position: u32,
    /// 当前音轨总时长（秒）
    pub duration: u32,
    /// 是否正在播放
    pub is_playing: bool,
    /// 是否已到达音轨末尾
    pub finished: bool,
}

impl Default for AudioStatus {
    fn default() -> Self {
        Self {
            position: 0,
            duration: 0,
            is_playing: false,
            finished: false,
        }
    }
}

/// 音频引擎
///
/// 管理独立的音频播放线程，通过 mpsc channel 发送命令，
/// 通过 Arc<Mutex<AudioStatus>> 共享状态。
pub struct AudioEngine {
    command_tx: mpsc::Sender<AudioCommand>,
    status: Arc<Mutex<AudioStatus>>,
}

impl Default for AudioEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioEngine {
    /// 创建新的音频引擎实例，启动独立的音频播放线程。
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        let status = Arc::new(Mutex::new(AudioStatus::default()));
        let status_clone = status.clone();

        // 启动真实的音频播放线程
        thread::spawn(move || {
            // 初始化音频输出设备，必须保持 _stream 的生命周期以维持播放
            let (_stream, stream_handle) = match OutputStream::try_default() {
                Ok(res) => res,
                Err(e) => {
                    error!("AudioEngine: Failed to open output stream: {}", e);
                    return;
                }
            };

            let sink = match Sink::try_new(&stream_handle) {
                Ok(s) => s,
                Err(e) => {
                    error!("AudioEngine: Failed to create sink: {}", e);
                    return;
                }
            };

            let mut total_duration = 0;
            let mut current_path: Option<String> = None;
            let mut jump_offset = 0;

            loop {
                // 检查并处理阻塞命令
                while let Ok(cmd) = rx.try_recv() {
                    match cmd {
                        AudioCommand::Load(path) => {
                            info!("AudioEngine: Loading {}", path);
                            current_path = Some(path.clone());
                            jump_offset = 0;
                            match File::open(&path) {
                                Ok(file) => {
                                    match Decoder::new(file) {
                                        Ok(source) => {
                                            // 记录当前文件的总时长
                                            total_duration = source
                                                .total_duration()
                                                .map(|d| d.as_secs() as u32)
                                                .unwrap_or(0);

                                            sink.stop(); // 停止当前播放（如果存在）
                                            sink.append(source);
                                            sink.play();
                                            info!(
                                                "AudioEngine: Started playing {}, duration: {}s",
                                                path, total_duration
                                            );
                                        }
                                        Err(e) => {
                                            error!("AudioEngine: Failed to decode {}: {}", path, e)
                                        }
                                    }
                                }
                                Err(e) => error!("AudioEngine: Failed to open {}: {}", path, e),
                            }
                        }
                        AudioCommand::Play => {
                            sink.play();
                            info!("AudioEngine: Play");
                        }
                        AudioCommand::Pause => {
                            sink.pause();
                            info!("AudioEngine: Pause");
                        }
                        AudioCommand::Stop => {
                            sink.stop();
                            total_duration = 0;
                            current_path = None;
                            jump_offset = 0;
                            info!("AudioEngine: Stop");
                        }
                        AudioCommand::Seek(pos) => {
                            // rodio 的 Sink::try_seek 支持跳转到指定 Duration
                            let target = Duration::from_secs(pos as u64);
                            match sink.try_seek(target) {
                                Ok(_) => {
                                    jump_offset = 0; // 原生跳转成功，进度由 sink 维护，重置补偿
                                    info!("AudioEngine: Seek to {}s success", pos);
                                }
                                Err(e) => {
                                    error!("AudioEngine: try_seek to {}s failed: {:?}. Attempting reload fallback.", pos, e);
                                    // 降级方案：重新加载并跳过。这通常适用于不支持跳转的解码后端或损坏的文件。
                                    if let Some(path) = &current_path {
                                        if let Ok(file) = File::open(path) {
                                            if let Ok(source) = Decoder::new(file) {
                                                sink.stop();
                                                sink.append(source.skip_duration(target));
                                                sink.play();
                                                jump_offset = pos; // 记录手动补偿位移
                                                info!(
                                                    "AudioEngine: Reload fallback to {}s success",
                                                    pos
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        AudioCommand::SetVolume(vol) => {
                            let volume_float = vol as f32 / 100.0;
                            sink.set_volume(volume_float);
                            info!("AudioEngine: Volume set to {}%", vol);
                        }
                    }
                }

                // 更新共享状态
                {
                    let mut s = status_clone.lock().unwrap();
                    let current_pos = sink.get_pos().as_secs() as u32;
                    s.position = current_pos + jump_offset; // 叠加补偿位移
                    s.duration = total_duration;
                    s.is_playing = !sink.empty() && !sink.is_paused();
                    s.finished = sink.empty() && total_duration > 0;
                }

                thread::sleep(Duration::from_millis(200));
            }
        });

        AudioEngine {
            command_tx: tx,
            status,
        }
    }

    /// 加载指定路径的音频文件。
    pub fn load(&self, path: &str) -> Result<(), String> {
        self.command_tx
            .send(AudioCommand::Load(path.to_string()))
            .map_err(|e| e.to_string())
    }

    /// 恢复播放当前已加载的音频。
    pub fn play(&self) -> Result<(), String> {
        self.command_tx
            .send(AudioCommand::Play)
            .map_err(|e| e.to_string())
    }

    /// 暂停播放。
    pub fn pause(&self) -> Result<(), String> {
        self.command_tx
            .send(AudioCommand::Pause)
            .map_err(|e| e.to_string())
    }

    /// 停止播放并重置进度。
    pub fn stop(&self) -> Result<(), String> {
        self.command_tx
            .send(AudioCommand::Stop)
            .map_err(|e| e.to_string())
    }

    /// 跳转到指定的秒数位置。
    pub fn seek(&self, position: u32) -> Result<(), String> {
        self.command_tx
            .send(AudioCommand::Seek(position))
            .map_err(|e| e.to_string())
    }

    /// 设置音量。
    pub fn set_volume(&self, volume: u8) -> Result<(), String> {
        self.command_tx
            .send(AudioCommand::SetVolume(volume))
            .map_err(|e| e.to_string())
    }

    /// 获取当前音频播放状态快照。
    pub fn status(&self) -> AudioStatus {
        self.status.lock().unwrap().clone()
    }
}
