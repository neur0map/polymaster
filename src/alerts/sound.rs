use std::io::{self, Write};

pub fn play_alert_sound() {
    play_sound_internal("/System/Library/Sounds/Ping.aiff");
}

pub fn play_anomaly_sound() {
    play_sound_internal("/System/Library/Sounds/Funk.aiff");
}

/// Play triple beep for repeat/heavy actors or exits
pub fn play_triple_beep() {
    play_alert_sound();
    std::thread::sleep(std::time::Duration::from_millis(100));
    play_alert_sound();
    std::thread::sleep(std::time::Duration::from_millis(100));
    play_alert_sound();
}

fn play_sound_internal(_sound_file: &str) {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("afplay")
            .arg(_sound_file)
            .spawn()
            .ok();
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("paplay")
            .arg("/usr/share/sounds/freedesktop/stereo/message.oga")
            .spawn()
            .or_else(|_| {
                std::process::Command::new("aplay")
                    .arg("/usr/share/sounds/alsa/Front_Center.wav")
                    .spawn()
            })
            .ok();
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("powershell")
            .arg("-c")
            .arg("[console]::beep(800,300)")
            .spawn()
            .ok();
    }

    print!("\x07");
    io::stdout().flush().ok();
}
