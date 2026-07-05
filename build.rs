use std::fs;


fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("android") {
        return;
    }
    let app_dir = "AppDir";
    for dir in vec![
        format!("{}/usr/bin", app_dir),
        format!("{}/usr/lib", app_dir),
        format!("{}/usr/share/applications", app_dir),
        format!("{}/usr/share/icons/hicolor/64x64/apps", app_dir),
    ] {
        fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("Failed to create {}: {}", dir, e));
    }
    let icon = "images/RustyChatBox_Icon.png";
    for dest in vec![
        format!("{}/rustychatbox.png", app_dir),
        format!("{}/usr/share/icons/hicolor/64x64/apps/rustychatbox.png", app_dir),
    ] {
        fs::copy(icon, &dest).unwrap_or_else(|e| panic!("Failed to copy icon: {}", e));
    }
    let desktop = "[Desktop Entry]
Name=RustyChatBox
Exec=rustychatbox
Type=Application
Icon=rustychatbox
Terminal=false
Categories=Utility;
StartupWMClass=RustyChatBox
Comment=A chat application built with Rust
";
    let dp = format!("{}/rustychatbox.desktop", app_dir);
    fs::write(&dp, desktop).unwrap();
    fs::copy(&dp, format!("{}/usr/share/applications/rustychatbox.desktop", app_dir)).unwrap();
    println!("cargo:warning=AppDir created at {}.", app_dir);
}