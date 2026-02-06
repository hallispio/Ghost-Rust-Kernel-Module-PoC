# /workspaces/rust/workspace/src/chrono-kernel-core/Makefile

KDIR := /workspaces/rust/workspace/local-kernel
PWD  := $(shell pwd)

TARGET := x86_64-unknown-none
RUST_OUT := target/$(TARGET)/release/libghost_shell.a
LOCAL_LIB := libghost_shell_static.a

# 🔥 [수정] ghost_shell-objs 대신 -y를 쓰고, 라이브러리를 직접 결합
obj-m += ghost_shell.o
ghost_shell-y := shim.o $(LOCAL_LIB)

all:
	@echo "🔥 [Step 1] Rust 빌드 중..."
	cargo build --release --target $(TARGET) -Z build-std
	
	@echo "🔥 [Step 2] 라이브러리 복사 및 링크..."
	cp $(RUST_OUT) ./$(LOCAL_LIB)
	# 커널 빌드 시스템을 속이는 .cmd 파일 생성
	@touch .$(LOCAL_LIB).cmd
	
	@echo "🔥 [Step 3] 커널 모듈 빌드..."
	# KBUILD_MODPOST_WARN=1로 심볼 체크 유연하게 통과
	$(MAKE) -C $(KDIR) M=$(PWD) KBUILD_MODPOST_WARN=1 modules

clean:
	cargo clean
	$(MAKE) -C $(KDIR) M=$(PWD) clean
	rm -f $(LOCAL_LIB) .$(LOCAL_LIB).cmd