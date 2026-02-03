# Universal i18n Layer for Linux Kernel

Universal multilingual translation layer for Linux using Rust & kprobe hook. 
Translates terminal messages on-the-fly with minimal overhead (~11ns/call).

## 🛠 Installation
1. Rust nightly + cargo 설치
2. 커널 헤더 설치: `sudo apt install linux-headers-$(uname -r)`
3. `make` && `sudo insmod ko-terminal-translator.ko`
4. `export LANG=ko_KR.UTF-8` (이미 설정돼 있으면 생략)

## ✨ Features in Detail
- **Early Return Filter**: 99% 호출 10ns 이내 패스
- **Multilingual Support**: 매핑 테이블로 확장 가능 (한국어 기본)
- **Compatibility**: 바이너리/네트워크/ANSI escape 완전 무시
- **Overhead**: ~11ns/call 평균 (테스트 기준)

## 📊 Status
- PoC 단계
- 개발 중 (Private)

## ⚖️ License
MIT License
