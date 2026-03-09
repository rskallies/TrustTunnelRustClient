fn main() {
    // Embed the UAC elevation manifest on Windows builds.
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_manifest_file("installer.manifest");
        res.compile().expect("failed to embed manifest");
    }
}
