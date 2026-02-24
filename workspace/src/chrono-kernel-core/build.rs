// src/chrono-kernel-core/build.rs
// Modified for Hyung-nim (Force Local Kernel Dir)
use std::env;
use std::path::PathBuf;

fn main() {
    // 1. 경로 수사 및 wrapper.h 확인
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let wrapper_path = PathBuf::from(&manifest_dir).join("wrapper.h");
    
    if !wrapper_path.exists() {
        panic!("\n❌ wrapper.h가 없습니다! 경로: {:?}", wrapper_path);
    }

    // 💀 [기존 코드 삭제함] uname -r 믿다가 망함.
    // 🚀 [수정됨] 앞마당으로 강제 고정!
    let kernel_dir = env::var("KERNEL_DIR").unwrap_or_else(|_| {
        let output = std::process::Command::new("uname").arg("-r").output().unwrap();
        let version = String::from_utf8(output.stdout).unwrap().trim().to_string();
        format!("/lib/modules/{}/build", version)
    });

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
        // 🔥 재배치 에러(Relocation 9)를 일으키는 수학 함수들 싹 다 블랙리스트 처리
        .blocklist_function("__adddf3")
        .blocklist_function("__muldf3")
        .blocklist_function("__divdf3")
        .blocklist_function("__subdf3")
        .blocklist_function("__addsf3")
        .blocklist_function("__mulsf3")
        .blocklist_function("__divsf3")
        .blocklist_function("__subsf3")
        .blocklist_function("__extendsfdf2")
        .blocklist_function("__truncdfsf2")
        .blocklist_function("atan2.*")
        .blocklist_function("sin.*")
        .blocklist_function("cos.*")
        .blocklist_function("tan.*")
        .blocklist_function("__.*") // 모든 내부 언더바 함수 차단
        .blocklist_type("__va_list_tag")
        .blocklist_type(".*float.*")
        .blocklist_type(".*double.*")
        .blocklist_type("__va_list_tag")
        // 🔥 --------------------------------------------------------------------------
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
        // 2. "야! 나 진짜 쓴다고!" (매크로 강제 정의)
        .clang_arg("-DCC_USING_FENTRY")
        
        .generate()
        .expect("❌ Bindgen 생성 실패! (wrapper.h나 커널 헤더 확인 필요)");

    // 5. 보물지도(bindings.rs) 기록
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("❌ 파일 쓰기 실패");
}
