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

    // 2. 현재 커널 버전 동적 추출 (6.12.8 등 자동 대응)
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
    let mut builder = bindgen::Builder::default()
        .header(wrapper_path.to_str().unwrap())
        .use_core(); // 커널이니까 libstd 안 쓰게 설정

    // 5. [루프 폭격] 모든 경로를 자동으로 주입
    for path in include_paths {
        builder = builder.clang_arg(format!("-I{}/{}", kernel_dir, path));
    }

    // 6. 커널 설정(kconfig.h) 강제 포함 및 매크로 설정
    let bindings = bindgen::Builder::default()
        .header(wrapper_path.to_str().unwrap())
        .use_core()
        // 💡 [자동 정렬 해결책 1] 레이아웃 테스트 생성을 끕니다. 
        // (E0588 에러의 주원인인 정렬 확인 코드를 안 만듦)
        .layout_tests(false)
        // 💡 [자동 정렬 해결책 2] 문제가 되는 정렬 속성을 러스트가 이해할 수 있게 변환
        .rustified_enum(".*") // 모든 열거형을 러스트 스타일로 강제 변환
        .derive_default(true)
        .derive_debug(true)
        // 💡 특정 구조체에서 터지는 걸 막기 위한 최후의 수단 (정렬 무시)
        .no_copy(".*") 
        
        // --- 아까 만든 자동 매핑 루프 시작 ---
        .clang_args(include_paths.iter().map(|path| format!("-I{}/{}", kernel_dir, path)))
        .clang_arg("-include")
        .clang_arg(format!("{}/include/linux/kconfig.h", kernel_dir))
        .clang_arg("-D__KERNEL__")
        // --- 루프 끝 ---
        
        .generate()
        .expect("❌ 그래도 안 되면 이건 커널이 형님 거부하는 겁니다 ㅋㅋㅋ");

    // 7. 보물지도(bindings.rs) 기록
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("❌ 파일 쓰기 실패");
}