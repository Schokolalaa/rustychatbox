use serde::{Deserialize, Serialize};
use std::process::Command;
use eframe::egui;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaLinkOptions {
    pub enabled: bool,
    pub use_music_note_prefix: bool,
    pub show_pause_emoji: bool,
    pub auto_switch_state: bool,
    pub auto_switch_session: bool,
    pub forget_session_seconds: u32,
    pub show_progress: bool,
    pub seekbar_style: String,
}

impl Default for MediaLinkOptions {
    fn default() -> Self {
        MediaLinkOptions {
            enabled: true,
            use_music_note_prefix: false,
            show_pause_emoji: false,
            auto_switch_state: true,
            auto_switch_session: true,
            forget_session_seconds: 30,
            show_progress: false,
            seekbar_style: "Small numbers".to_string(),
        }
    }
}

impl MediaLinkOptions {
    pub fn show_medialink_options(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let mut response = ui.interact(
            egui::Rect::EVERYTHING,
            ui.id().with("medialink_options"),
            egui::Sense::hover(),
        );
        ui.label("Basic options");
        ui.label(egui::RichText::new("Customize how your media looks in your chatbox").text_style(egui::TextStyle::Small));
        response |= ui.checkbox(&mut self.use_music_note_prefix, "Change 'Listening to:' prefix to 🎵");
        response |= ui.checkbox(&mut self.show_pause_emoji, "Show ⏸ when music is paused");
        response |= ui.checkbox(&mut self.auto_switch_state, "Auto switch when media state changes");
        response |= ui.checkbox(&mut self.auto_switch_session, "Auto switch when a new session is detected");
        ui.horizontal(|ui| {
            ui.label("Forget session after");
            response |= ui.add(egui::DragValue::new(&mut self.forget_session_seconds).speed(1.0));
            ui.label("seconds");
        });
        ui.label("Media progress bar");
        ui.label(egui::RichText::new("Customize how your seek bar looks").text_style(egui::TextStyle::Small));
        ui.label("Seekbar style");
        let combo_response = egui::ComboBox::from_label("")
            .selected_text(&self.seekbar_style)
            .show_ui(ui, |ui| {
                let seekbar_styles = ["Small numbers", "Custom", "None"];
                let mut combo_response = ui.selectable_value(&mut self.seekbar_style, seekbar_styles[0].to_string(), seekbar_styles[0]);
                for style in seekbar_styles.iter().skip(1) {
                    combo_response |= ui.selectable_value(&mut self.seekbar_style, style.to_string(), *style);
                }
                combo_response
            });
        response |= combo_response.response;
        response |= ui.checkbox(&mut self.show_progress, "Show media progress");
        response
    }
}

pub struct MediaLinkModule;

impl MediaLinkModule {
    pub fn new() -> Self {
        Self
    }

    fn playerctl_smart(&self, args: &[&str]) -> Option<String> {
        let candidates = ["strawberry", "tauon", "vlc", "haruna", "plasma-browser-integration", "chromium", "brave", "firefox", "%any"];

        for player in candidates {
            let mut cmd = Command::new("playerctl");
            cmd.arg("--player").arg(player);
            cmd.args(args);

            if let Ok(output) = cmd.output() {
                if output.status.success() {
                    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !s.is_empty() && !s.contains("No player") && !s.contains("Could not") && !s.contains("No players found") {
                        if args.first() == Some(&"status") && s == "Stopped" {
                            continue;
                        }
                        return Some(s);
                    }
                }
            }
        }
        None
    }

    pub fn get_formatted_track(&self, options: &MediaLinkOptions) -> Option<String> {
        let status = self.playerctl_smart(&["status"]);
        if let Some(ref s) = status {
            if s == "Playing" {

                let track = self.playerctl_smart(&[
                    "metadata",
                    "--format",
                    "{{xesam:artist}} - {{xesam:title}}",
                ])?
                .trim()
                .to_string();

                let track = if track.starts_with(" - ") || track.ends_with(" - ") || track == " - " {
                    self.playerctl_smart(&["metadata", "--format", "{{xesam:title}}"])?
                        .trim()
                        .to_string()
                } else {
                    track
                };

                if track.is_empty() || track == " - " || track.contains("null") {
                    return None;
                }

                let prefix = if options.use_music_note_prefix {
                    "🎵 "
                } else {
                    "Listening to: "
                };
                return Some(format!("{}{}", prefix, track));
            } else {
              
                return Some(if options.show_pause_emoji {
                    "⏸".to_string()
                } else {
                    "Paused".to_string()
                });
            }
        }

        Some(if options.show_pause_emoji {
            "⏸".to_string()
        } else {
            "Paused".to_string()
        })
    }

    pub fn is_playing(&self) -> bool {
        self.playerctl_smart(&["status"]).map(|s| s == "Playing").unwrap_or(false)
    }

    pub fn play_pause(&self) {
        Command::new("playerctl")
            .arg("play-pause")
            .output()
            .unwrap_or_else(|e| {
                eprintln!("Failed to toggle play/pause: {}", e);
                std::process::Output {
                    status: std::process::ExitStatus::default(),
                    stdout: vec![],
                    stderr: vec![],
                }
            });
    }

    pub fn next(&self) {
        Command::new("playerctl")
            .arg("next")
            .output()
            .unwrap_or_else(|e| {
                eprintln!("Failed to skip to next: {}", e);
                std::process::Output {
                    status: std::process::ExitStatus::default(),
                    stdout: vec![],
                    stderr: vec![],
                }
            });
    }

    pub fn previous(&self) {
        Command::new("playerctl")
            .arg("previous")
            .output()
            .unwrap_or_else(|e| {
                eprintln!("Failed to go to previous: {}", e);
                std::process::Output {
                    status: std::process::ExitStatus::default(),
                    stdout: vec![],
                    stderr: vec![],
                }
            });
    }

    pub fn seek(&self, position: f32) {
        Command::new("playerctl")
            .arg("position")
            .arg(position.to_string())
            .output()
            .unwrap_or_else(|e| {
                eprintln!("Failed to seek: {}", e);
                std::process::Output {
                    status: std::process::ExitStatus::default(),
                    stdout: vec![],
                    stderr: vec![],
                }
            });
    }

    pub fn get_position(&self) -> Option<f32> {
        self.playerctl_smart(&["position"])
            .and_then(|s| s.parse::<f32>().ok())
    }

    pub fn get_duration(&self) -> Option<f32> {
        self.playerctl_smart(&["metadata", "mpris:length"])
            .and_then(|s| s.parse::<f64>().ok())
            .map(|microseconds| (microseconds / 1_000_000.0) as f32)
    }
}