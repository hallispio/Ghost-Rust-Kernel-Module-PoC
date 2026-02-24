#include <linux/module.h>
#include <linux/kernel.h>

// Rust 함수 선언 (Rust야, 나와라!)
extern int init_hook(void);
extern void cleanup_hook(void);
extern void print_stats(void);

// 모듈 로딩 시 실행 (insmod)
static int __init ghost_init(void) {
    // Rust 깨움
    return init_hook();
}

// 모듈 제거 시 실행 (rmmod)
static void __exit ghost_exit(void) {
    print_stats();   // 마지막 통계 출력
    cleanup_hook();  // Rust 정리
}

module_init(ghost_init);
module_exit(ghost_exit);

MODULE_LICENSE("GPL");
MODULE_AUTHOR("Ghost Shell");
MODULE_DESCRIPTION("Rust Kernel Module");
