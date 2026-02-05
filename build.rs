use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // 获取输出目录
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir);
    
    // 获取 profile (debug/release)
    let profile = env::var("PROFILE").unwrap();
    
    // 计算目标目录 (target/debug 或 target/release)
    let target_dir = out_path
        .ancestors()
        .find(|p| p.ends_with(&profile))
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| out_path.clone());
    
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=llvm-minimal/");
    println!("cargo:rerun-if-changed=lib/");
    println!("cargo:rerun-if-changed=mingw-minimal/");
    println!("cargo:rerun-if-changed=third-party/");
    
    // 复制 llvm-minimal 目录
    copy_dir_all("llvm-minimal", &target_dir.join("llvm-minimal"))
        .expect("Failed to copy llvm-minimal directory");
    
    // 复制 lib 目录
    copy_dir_all("lib", &target_dir.join("lib"))
        .expect("Failed to copy lib directory");
    
    // 复制 mingw-minimal 目录
    copy_dir_all("mingw-minimal", &target_dir.join("mingw-minimal"))
        .expect("Failed to copy mingw-minimal directory");
    
    // 复制 third-party 目录 (许可证文件)
    copy_dir_all("third-party", &target_dir.join("third-party"))
        .expect("Failed to copy third-party directory");
    
    println!("cargo:warning=Copied toolchain and license directories to {}", target_dir.display());
}

fn copy_dir_all(src: impl AsRef<std::path::Path>, dst: impl AsRef<std::path::Path>) -> std::io::Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();
    
    if !src.exists() {
        return Ok(());
    }
    
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let dest_path = dst.join(&file_name);
        
        if path.is_dir() {
            copy_dir_all(&path, &dest_path)?;
        } else {
            // 只在文件不存在或源文件更新时才复制
            let should_copy = if dest_path.exists() {
                let src_meta = fs::metadata(&path)?;
                let dst_meta = fs::metadata(&dest_path)?;
                src_meta.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH) > 
                    dst_meta.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH)
            } else {
                true
            };
            
            if should_copy {
                fs::copy(&path, &dest_path)?;
            }
        }
    }
    
    Ok(())
}
