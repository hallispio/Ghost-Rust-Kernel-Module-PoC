#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/kprobes.h>

MODULE_LICENSE("GPL");
MODULE_AUTHOR("Bureum Lee");

// 🔥 [변경] Rust로 딱 하나의 "정답 주소"만 보냅니다.
extern int init_hook(unsigned long sys_write_addr);
extern void cleanup_hook(void);
extern void print_stats(void);

// Rust에서 쓸 출력 함수 (살림)
void ghost_printk(const char *fmt) {
    printk(KERN_INFO "%s", fmt);
}

// 🛠️ [유틸] 주소 찾는 만능 함수 (이건 좋아서 그대로 살림)
static unsigned long find_symbol_addr(const char *symbol) {
    struct kprobe kp = { .symbol_name = symbol };
    unsigned long addr;

    // 잠깐 찔러보고 주소만 따오기
    if (register_kprobe(&kp) < 0) {
        printk(KERN_ERR "[GHOST] ❌ Failed to find symbol: %s\n", symbol);
        return 0;
    }
    
    addr = (unsigned long)kp.addr;
    unregister_kprobe(&kp); // 주소 확보 후 즉시 철수
    
    printk(KERN_INFO "[GHOST] 🎯 Found %s at: %lx\n", symbol, addr);
    return addr;
}

static int __init ghost_init(void) {
    unsigned long sys_write_addr;
    
    printk(KERN_INFO "[GHOST] 🚀 Scanning System Call Entry...\n");

    // 🔥 [핵심] VFS, KSYS 다 필요 없고 "정문"만 찾습니다.
    sys_write_addr = find_symbol_addr("__x64_sys_write");

    // 혹시 커널 버전에 따라 이름이 다를까 봐 예비책 하나만 둠
    if (!sys_write_addr) {
        sys_write_addr = find_symbol_addr("sys_write");
    }

    if (!sys_write_addr) {
        printk(KERN_ERR "[GHOST] FATAL: Cannot find write syscall!\n");
        return -1;
    }

    // 확보한 정문 주소 전송
    return init_hook(sys_write_addr); 
}

static void __exit ghost_exit(void) {
    print_stats();
    cleanup_hook();
    printk(KERN_INFO "[GHOST] 💀 Unloading Module...\n");
}

module_init(ghost_init);
module_exit(ghost_exit);
