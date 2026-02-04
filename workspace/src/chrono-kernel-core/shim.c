// shim.c - 커널 문지기 (최종)
#include <linux/module.h>
#include <linux/kernel.h>

// 🔥 [필수] 이거 없으면 'module_layout' 에러 뜨고 난리 납니다.
MODULE_LICENSE("GPL");
MODULE_AUTHOR("Bureum Lee");
MODULE_DESCRIPTION("Ghost Shell Module");

// 1. Rust에 있는 함수들을 쓰겠다고 선언 (이름 맞춰야 함!)
// (lib.rs에서 #[no_mangle] extern "C" fn init_hook() ... 이렇게 돼있어야 함)
extern int init_hook(void);
extern void cleanup_hook(void);
extern void print_stats(void);

// 2. 모듈 꽂을 때 실행 (insmod)
static int __init ghost_init(void) {
    printk(KERN_INFO "[GHOST] 🚀 Loading Module...\n");
    
    // 바로 Rust한테 토스!
    return init_hook(); 
}

// 3. 모듈 뺄 때 실행 (rmmod)
static void __exit ghost_exit(void) {
    // 통계 출력하고
    print_stats();
    // 청소하고
    cleanup_hook();
    
    printk(KERN_INFO "[GHOST] 💀 Unloading Module...\n");
}

// 커널한테 진입점 알려주기
module_init(ghost_init);
module_exit(ghost_exit);