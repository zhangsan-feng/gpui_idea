fn main() {
    println!("cargo:rerun-if-changed=assets/app/app.rc");
    println!("cargo:rerun-if-changed=assets/app/app-logo-v3.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        // GPUI loads the native application icon from resource ID 1.
        embed_resource::compile("assets/app/app.rc", embed_resource::NONE)
            .manifest_required()
            .expect("failed to embed the application icon");
    }
}
