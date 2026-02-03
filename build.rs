fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // 设置 rpath 以便运行时能找到 sherpa-onnx 动态库
    // 库文件由 sherpa-rs-sys 放置在 target/{profile}/deps/ 目录

    #[cfg(target_os = "linux")]
    {
        // Linux: 使用 $ORIGIN 相对路径
        // 可执行文件在 target/{profile}/，库在 target/{profile}/deps/
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/deps");
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: 使用 @executable_path 相对路径
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path");
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/deps");
    }
}
