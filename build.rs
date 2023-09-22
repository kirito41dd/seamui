fn main() {
    slint_build::compile("ui/appwindow.slint").unwrap();
    #[cfg(target_os = "windows")]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winresource::WindowsResource::new()
            .set_icon("./static/logo(1).ico")
            .compile()
            .unwrap();
    }
}
