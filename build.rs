fn main() {
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rerun-if-changed=packaging/windows/suntray.rc");
        println!("cargo:rerun-if-changed=assets/icons/sunsynk.ico");

        embed_resource::compile("packaging/windows/suntray.rc", embed_resource::NONE)
            .manifest_optional()
            .expect("failed to embed Windows application resources");
    }
}
