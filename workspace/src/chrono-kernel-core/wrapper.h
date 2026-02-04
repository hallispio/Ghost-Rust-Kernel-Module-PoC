// wrapper.h
// 기본 타입
#include <linux/types.h>

// 🔥 [핵심 추가] 이 2개를 넣어야 족보에 내용이 채워짐!
#include <linux/kprobes.h> // kprobe 구조체용
#include <linux/ptrace.h>  // pt_regs (레지스터)용