KDIR := /workspaces/rust/workspace/local-kernel
PWD  := $(shell pwd)
MODULE_NAME := ghost_driver
TARGET := x86_64-unknown-none
GHOST_LIB := ghost_shell_core.o

obj-m += $(MODULE_NAME).o
$(MODULE_NAME)-y := shim.o $(GHOST_LIB)

export RUSTFLAGS := -C code-model=kernel \
	-C relocation-model=static \
	-C no-redzone \
	-C panic=abort \
	-C target-feature=-mmx,-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-avx,-avx2,-x87 \
	-C link-self-contained=yes

all:
	@echo "🔥 [Step 1] Rust 빌드..."
	@rm -f .*.cmd *.o
	@RUSTC_BOOTSTRAP=1 cargo rustc --release --target $(TARGET) -- \
		--emit=obj=$(GHOST_LIB) \
		-C relocation-model=static \
		-C lto=fat \
		-C opt-level=z
	@echo "🔥 [Step 2] 바이너리 장부 생성..."
	@echo "savedcmd_$(PWD)/$(GHOST_LIB) := true" > .$(GHOST_LIB).cmd
	@echo "🔥 [Step 3] 커널 모듈 빌드..."
	@$(MAKE) -C $(KDIR) M=$(PWD) KBUILD_MODPOST_WARN=1 modules > /dev/null 2>&1 || (echo "❌ [KERNEL ERROR] Kbuild 실패"; exit 1)
	@echo "\n===================================================="
	@echo "        🚀 [GHOST] 최종 전술 보고서"
	@echo "===================================================="
	@# GOT 검사
	@if objdump -r $(MODULE_NAME).o | grep -q "GOTPCREL"; then \
		echo "❌ [결과] GOTPCREL 검출! (Relocation 9 잔존)"; \
		objdump -r $(MODULE_NAME).o | grep "GOTPCREL" | head -n 5; \
		RESULT=1; \
	else \
		echo "✅ [결과] GOTPCREL 없음 (바이너리 클린)"; \
		RESULT=0; \
	fi
	@# 심볼 검사
	@if nm $(GHOST_LIB) | grep -q "T init_hook"; then \
		echo "✅ [결과] init_hook 심볼 활성화 완료"; \
	else \
		echo "❌ [결과] init_hook 실종"; \
		RESULT=1; \
	fi
	@# 최종 상태 출력
	@if [ "$$RESULT" = "0" ]; then \
		echo "----------------------------------------------------"; \
		echo "🔥 [상태] 침투 준비 완료! 즉시 insmod 하십시오."; \
		echo "👉 sudo insmod $(MODULE_NAME).ko"; \
		echo "----------------------------------------------------"; \
	else \
		echo "⚠️  [상태] 보수 필요 (적재 금지)"; \
	fi
	@echo "===================================================="

clean:
	cargo clean
	$(MAKE) -C $(KDIR) M=$(PWD) clean
	rm -f $(GHOST_LIB) .*.cmd *.o