fn main() {
    // 使用正确的路径编译Slint文件
    slint_build::compile("src/main.slint").unwrap();
}