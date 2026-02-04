// src/chrono-kernel-core/build.rs
use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // 1. 경로 수사 및 wrapper.h 확인
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let wrapper_path = PathBuf::from(&manifest_dir).join("wrapper.h");
    
    if !wrapper_path.exists() {
        panic!("\n❌ wrapper.h가 없습니다! 경로: {:?}", wrapper_path);
    }

    // 2. 현재 커널 버전 동적 추출
    let output = Command::new("uname").arg("-r").output().expect("uname 실행 실패");
    let kernel_version = String::from_utf8(output.stdout).unwrap().trim().to_string();
    let kernel_dir = format!("/lib/modules/{}/build", kernel_version);

    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:warning=🚀 Using Kernel Dir: {}", kernel_dir);

    // 3. [자동 매핑] 커널 소스의 모든 통로를 배열로 정의
    let include_paths = [
        "include",
        "arch/x86/include",
        "include/generated",
        "arch/x86/include/generated",
        "include/uapi",
        "arch/x86/include/uapi",
    ];

    // 4. 빌더 시동
    let bindings = bindgen::Builder::default()
        .header(wrapper_path.to_str().unwrap())
        .use_core()
        
        // 💡 [설정]
        .layout_tests(false)
        .rustified_enum(".*")
        .derive_default(true)
        .derive_debug(false)
        .no_copy(".*") 
        
        // --- 경로 주입 ---
        .clang_args(include_paths.iter().map(|path| format!("-I{}/{}", kernel_dir, path)))
        
        // --- 필수 매크로 설정 ---
        .clang_arg("-include")
        .clang_arg(format!("{}/include/linux/kconfig.h", kernel_dir))
        .clang_arg("-D__KERNEL__")
        
        // 🔥🔥🔥 [여기가 핵심 수정] 🔥🔥🔥
        // 1. 컴파일러 플래그
        .clang_arg("-mfentry")
        // 2. "야! 나 진짜 쓴다고!" (매크로 강제 정의) -> 이게 없어서 아까 에러 난 거임
        .clang_arg("-DCC_USING_FENTRY")
        
        .generate()
        .expect("❌ Bindgen 생성 실패! (wrapper.h나 커널 헤더 확인 필요)");

    // 5. 보물지도(bindings.rs) 기록
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("❌ 파일 쓰기 실패");
}