fn main() {
    println!("cargo:rerun-if-env-changed=WINNEXUS_BUILD_STAMP");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut kaynak = winres::WindowsResource::new();
        kaynak.set_icon("../../assets/winnexus.ico");
        kaynak.set("FileDescription", "SP GROUP WinNexus");
        kaynak.set("ProductName", "SP GROUP WinNexus");
        kaynak.set("FileVersion", "0.1.0.0");
        kaynak.set("ProductVersion", "0.1.0.0");
        kaynak.set("LegalCopyright", "SP GROUP");
        kaynak
            .compile()
            .expect("Windows sürüm kaynakları derlenemedi");
    }
}
