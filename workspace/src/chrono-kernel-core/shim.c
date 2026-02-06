// shim.c
#include <linux/module.h>
#include <linux/kernel.h>

MODULE_LICENSE("GPL");
MODULE_AUTHOR("Bureum Lee");
MODULE_DESCRIPTION("Ghost Shell Module");

// Rust 함수들
extern int init_hook(void);
extern void cleanup_hook(void);
extern void print_stats(void);

// 🔥 Rust에서 부를 출력 래퍼 (이름표를 명확히 함)
void ghost_printk(const char *fmt) {
    printk(KERN_INFO "%s", fmt);
}

static int __init ghost_init(void) {
    printk(KERN_INFO "[GHOST] 🚀 Loading Module...\n");
    return init_hook(); 
}

static void __exit ghost_exit(void) {
    print_stats();
    cleanup_hook();
    printk(KERN_INFO "[GHOST] 💀 Unloading Module...\n");
}

module_init(ghost_init);
module_exit(ghost_exit);