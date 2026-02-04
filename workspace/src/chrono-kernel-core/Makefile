# Makefile (Nightly + Traffic Control Edition)

KDIR ?= /lib/modules/$(shell uname -r)/build
PWD := $(shell pwd)

# 설정 파일(.cargo/config.toml) 믿고 갑니다.
TARGET := x86_64-unknown-linux-gnu
RUST_TARGET_DIR := target/$(TARGET)/release
RUST_LIB := $(RUST_TARGET_DIR)/libchrono_kernel_core.a

obj-m += ghost_shell.o
ghost_shell-y := shim.o $(RUST_LIB)

all:
	@echo "🔥 [GHOST] Cleaning legacy artifacts..."
	# 찌꺼기 때문에 안 될 수 있으니 target 폴더 과감히 삭제 (이번엔 필수)
	rm -rf target/
	
	@echo "🔥 [GHOST] Rust Re-build (Config Driven)..."
	# Nightly 사용 (설정 파일이 옵션 다 가지고 있음)
	cargo +nightly build --release
	
	@echo "🔥 [GHOST] Dummy Receipt..."
	@mkdir -p $(RUST_TARGET_DIR)
	touch $(RUST_TARGET_DIR)/.libchrono_kernel_core.a.cmd
	
	@echo "🔥 [GHOST] Linking..."
	$(MAKE) -C $(KDIR) M=$(PWD) modules KBUILD_MODPOST_WARN=1
	
	@echo "✅ Build Complete."

clean:
	cargo clean
	$(MAKE) -C $(KDIR) M=$(PWD) clean