#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/kprobes.h>
#include <linux/sched.h>
#include <linux/nsproxy.h>
#include <linux/pid_namespace.h>

MODULE_LICENSE("GPL");
MODULE_AUTHOR("GHOST");

extern int init_hook(unsigned long sys_write_addr);
extern void cleanup_hook(void);
extern void print_stats(void);
extern void set_danger_zones(unsigned long text_start, unsigned long text_end,
                             unsigned long data_start, unsigned long data_end);
// ============================================================
// 🔥 [핵심] 신원 조회 + FD 추적 (전수조사용)
// ============================================================
void ghost_inspect_task(unsigned long fd) {
    struct task_struct *task = current;
    unsigned int ns_id = 0;
    
    // Namespace ID 추출 (도커 판별)
    if (task->nsproxy && task->nsproxy->pid_ns_for_children) {
        ns_id = task->nsproxy->pid_ns_for_children->ns.inum;
    }
    
    // 🔥 생사부 박제 + FD(목적지) 추가
    printk(KERN_INFO "[GHOST-SCAN] Comm: %s | PID: %d | NS: %u | FD: %lu | CPU: %d\n", 
           task->comm, task->pid, ns_id, fd, smp_processor_id());
}

// ============================================================
// 기본 헬퍼 함수들
// ============================================================
void ghost_printk(const char *fmt) {
    printk(KERN_INFO "%s", fmt);
}

int ghost_register_kprobe(struct kprobe *kp) {
    return register_kprobe(kp);
}

void ghost_unregister_kprobe(struct kprobe *kp) {
    unregister_kprobe(kp);
}

unsigned long ghost_copy_from_user(void *to, const void *from, unsigned long n) {
    return copy_from_user(to, from, n);
}

unsigned long ghost_copy_to_user(void *to, const void *from, unsigned long n) {
    return copy_to_user(to, from, n);
}

// ============================================================
// 심볼 찾기
// ============================================================
static unsigned long find_symbol_addr(const char *symbol) {
    struct kprobe kp = { .symbol_name = symbol };
    unsigned long addr;

    if (register_kprobe(&kp) < 0) {
        printk(KERN_ERR "[GHOST] Failed to find: %s\n", symbol);
        return 0;
    }
    
    addr = (unsigned long)kp.addr;
    unregister_kprobe(&kp);
    
    printk(KERN_INFO "[GHOST] Found %s: %lx\n", symbol, addr);
    return addr;
}

// ============================================================
// 위험 지역 설정 (더미 - 에러 방지용)
// ============================================================

// ============================================================
// 초기화
// ============================================================
static int __init ghost_init(void) {
    unsigned long addr;
    
    printk(KERN_INFO "[GHOST] Initializing...\n");
    
    addr = find_symbol_addr("__x64_sys_write");
    if (!addr) {
        addr = find_symbol_addr("sys_write");
    }
    
    if (!addr) {
        printk(KERN_ERR "[GHOST] FATAL: sys_write not found!\n");
        return -1;
    }
    
    return init_hook(addr);
}

static void __exit ghost_exit(void) {
    print_stats();
    cleanup_hook();
    printk(KERN_INFO "[GHOST] Unloaded.\n");
}

module_init(ghost_init);
module_exit(ghost_exit);

// ============================================================
// 심볼 익스포트
// ============================================================
EXPORT_SYMBOL(ghost_printk);
EXPORT_SYMBOL(ghost_inspect_task);
EXPORT_SYMBOL(ghost_register_kprobe);
EXPORT_SYMBOL(ghost_unregister_kprobe);
EXPORT_SYMBOL(ghost_copy_from_user);
EXPORT_SYMBOL(ghost_copy_to_user);
