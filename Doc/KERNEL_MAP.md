# 🛡️ Ghost Shell - Kernel API Map
## 1. Available Kernel Functions
    pub fn get_bit(&self, index: usize) -> bool {
    pub fn set_bit(&mut self, index: usize, val: bool) {
    pub fn get(&self, bit_offset: usize, bit_width: u8) -> u64 {
    pub fn set(&mut self, bit_offset: usize, bit_width: u8, val: u64) {
    pub fn as_ptr(&self) -> *const T {
    pub fn as_mut_ptr(&mut self) -> *mut T {
    pub fn ftrace_likely_update(
    pub fn ibt_save(disable: bool_) -> u64_;
    pub fn ibt_restore(save: u64_);
    pub fn __sw_hweight8(w: ::core::ffi::c_uint) -> ::core::ffi::c_uint;
    pub fn __sw_hweight16(w: ::core::ffi::c_uint) -> ::core::ffi::c_uint;
    pub fn __sw_hweight32(w: ::core::ffi::c_uint) -> ::core::ffi::c_uint;
    pub fn __sw_hweight64(w: __u64) -> ::core::ffi::c_ulong;
    pub fn cpuid(&self) -> u32_ {
    pub fn set_cpuid(&mut self, val: u32_) {
    pub fn flags(&self) -> u32_ {
    pub fn set_flags(&mut self, val: u32_) {
    pub fn new_bitfield_1(cpuid: u32_, flags: u32_) -> __BindgenBitfieldUnit<[u8; 4usize]> {
    pub fn alternative_instructions();
    pub fn apply_alternatives(start: *mut alt_instr, end: *mut alt_instr);
    pub fn apply_retpolines(start: *mut s32, end: *mut s32);
    pub fn apply_returns(start: *mut s32, end: *mut s32);
    pub fn apply_seal_endbr(start: *mut s32, end: *mut s32);
    pub fn apply_fineibt(
    pub fn callthunks_patch_builtin_calls();
    pub fn callthunks_patch_module_calls(sites: *mut callthunk_sites, mod_: *mut module);
    pub fn callthunks_translate_call_dest(
    pub fn x86_call_depth_emit_accounting(
    pub fn alternatives_smp_module_add(
    pub fn alternatives_smp_module_del(mod_: *mut module);
    pub fn alternatives_enable_smp();
    pub fn alternatives_text_reserved(
    pub fn hex_to_bin(ch: ::core::ffi::c_uchar) -> ::core::ffi::c_int;
    pub fn hex2bin(
    pub fn bin2hex(
    pub fn mac_pton(s: *const ::core::ffi::c_char, mac: *mut u8_) -> bool_;
    pub fn _kstrtoul(
    pub fn _kstrtol(
    pub fn kstrtoull(
    pub fn kstrtoll(
    pub fn kstrtouint(
    pub fn kstrtoint(
    pub fn kstrtou16(
    pub fn kstrtos16(
    pub fn kstrtou8(
    pub fn kstrtos8(
    pub fn kstrtobool(s: *const ::core::ffi::c_char, res: *mut bool_) -> ::core::ffi::c_int;
    pub fn kstrtoull_from_user(
    pub fn kstrtoll_from_user(
    pub fn kstrtoul_from_user(
    pub fn kstrtol_from_user(
    pub fn kstrtouint_from_user(
    pub fn kstrtoint_from_user(
    pub fn kstrtou16_from_user(
    pub fn kstrtos16_from_user(
    pub fn kstrtou8_from_user(
    pub fn kstrtos8_from_user(
    pub fn kstrtobool_from_user(
    pub fn simple_strtoul(
    pub fn simple_strtol(
    pub fn simple_strtoull(
    pub fn simple_strtoll(
    pub fn int_pow(base: u64_, exp: ::core::ffi::c_uint) -> u64_;
    pub fn int_sqrt(arg1: ::core::ffi::c_ulong) -> ::core::ffi::c_ulong;
    pub fn panic(fmt: *const ::core::ffi::c_char, ...) -> !;
    pub fn nmi_panic(regs: *mut pt_regs, msg: *const ::core::ffi::c_char);
    pub fn check_panic_on_warn(origin: *const ::core::ffi::c_char);
    pub fn oops_enter();
    pub fn oops_exit();
    pub fn oops_may_print() -> bool_;
    pub fn __stack_chk_fail();
    pub fn abort() -> !;
    pub fn print_tainted() -> *const ::core::ffi::c_char;
    pub fn add_taint(flag: ::core::ffi::c_uint, arg1: lockdep_ok);
    pub fn test_taint(flag: ::core::ffi::c_uint) -> ::core::ffi::c_int;
    pub fn get_taint() -> ::core::ffi::c_ulong;
    pub fn do_one_initcall(fn_: initcall_t) -> ::core::ffi::c_int;
    pub fn setup_arch(arg1: *mut *mut ::core::ffi::c_char);
    pub fn prepare_namespace();
    pub fn init_rootfs();
    pub fn init_IRQ();
    pub fn time_init();
    pub fn poking_init();
    pub fn pgtable_cache_init();
    pub fn mark_rodata_ro();
    pub fn parse_early_param();
    pub fn parse_early_options(cmdline: *mut ::core::ffi::c_char);
    pub fn ___ratelimit(
    pub fn console_verbose();
    pub fn early_printk(fmt: *const ::core::ffi::c_char, ...);
    pub fn vprintk_emit(
    pub fn vprintk(fmt: *const ::core::ffi::c_char, args: *mut __va_list_tag)
    pub fn _printk(fmt: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    pub fn _printk_deferred(fmt: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    pub fn __printk_safe_enter();
    pub fn __printk_safe_exit();
    pub fn __printk_ratelimit(func: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    pub fn printk_timed_ratelimit(
    pub fn wake_up_klogd();
    pub fn log_buf_addr_get() -> *mut ::core::ffi::c_char;
    pub fn log_buf_len_get() -> u32_;
    pub fn log_buf_vmcoreinfo_setup();
    pub fn setup_log_buf(early: ::core::ffi::c_int);
    pub fn dump_stack_set_arch_desc(fmt: *const ::core::ffi::c_char, ...);
    pub fn dump_stack_print_info(log_lvl: *const ::core::ffi::c_char);
    pub fn show_regs_print_info(log_lvl: *const ::core::ffi::c_char);
    pub fn dump_stack_lvl(log_lvl: *const ::core::ffi::c_char);
    pub fn dump_stack();
    pub fn printk_trigger_flush();
    pub fn __printk_cpu_sync_try_get() -> ::core::ffi::c_int;
    pub fn __printk_cpu_sync_wait();
    pub fn __printk_cpu_sync_put();
    pub fn arch_jump_entry_size(entry: *mut jump_entry) -> ::core::ffi::c_int;
    pub fn jump_label_init();
    pub fn jump_label_lock();
    pub fn jump_label_unlock();
    pub fn arch_jump_label_transform(entry: *mut jump_entry, type_: jump_label_type);
    pub fn arch_jump_label_transform_queue(entry: *mut jump_entry, type_: jump_label_type)
    pub fn arch_jump_label_transform_apply();
    pub fn jump_label_text_reserved(
    pub fn static_key_slow_inc(key: *mut static_key) -> bool_;
    pub fn static_key_fast_inc_not_disabled(key: *mut static_key) -> bool_;
    pub fn static_key_slow_dec(key: *mut static_key);
    pub fn static_key_slow_inc_cpuslocked(key: *mut static_key) -> bool_;
    pub fn static_key_slow_dec_cpuslocked(key: *mut static_key);
    pub fn static_key_count(key: *mut static_key) -> ::core::ffi::c_int;
    pub fn static_key_enable(key: *mut static_key);
    pub fn static_key_disable(key: *mut static_key);
    pub fn static_key_enable_cpuslocked(key: *mut static_key);
    pub fn static_key_disable_cpuslocked(key: *mut static_key);
    pub fn jump_label_init_type(entry: *mut jump_entry) -> jump_label_type;
    pub fn ____wrong_branch_error() -> bool_;
    pub fn lineno(&self) -> ::core::ffi::c_uint {
    pub fn set_lineno(&mut self, val: ::core::ffi::c_uint) {
    pub fn class_id(&self) -> ::core::ffi::c_uint {
    pub fn set_class_id(&mut self, val: ::core::ffi::c_uint) {
    pub fn flags(&self) -> ::core::ffi::c_uint {
    pub fn set_flags(&mut self, val: ::core::ffi::c_uint) {
    pub fn new_bitfield_1(
    pub fn __dynamic_pr_debug(descriptor: *mut _ddebug, fmt: *const ::core::ffi::c_char, ...);
    pub fn __dynamic_dev_dbg(
    pub fn __dynamic_netdev_dbg(
    pub fn __dynamic_ibdev_dbg(
    pub fn ddebug_dyndbg_module_param_cb(
    pub fn param_set_dyndbg_classes(
    pub fn param_get_dyndbg_classes(
    pub fn hex_dump_to_buffer(
    pub fn print_hex_dump(
    pub fn num_to_str(
    pub fn sprintf(
    pub fn vsprintf(
    pub fn snprintf(
    pub fn vsnprintf(
    pub fn scnprintf(
    pub fn vscnprintf(
    pub fn kasprintf(gfp: gfp_t, fmt: *const ::core::ffi::c_char, ...) -> *mut ::core::ffi::c_char;
    pub fn kvasprintf(
    pub fn kvasprintf_const(
    pub fn sscanf(
    pub fn vsscanf(
    pub fn no_hash_pointers_enable(str_: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
    pub fn __cond_resched() -> ::core::ffi::c_int;
    pub fn __SCT__might_resched() -> ::core::ffi::c_int;
    pub fn do_exit(error_code: ::core::ffi::c_long) -> !;
    pub fn get_option(
    pub fn get_options(
    pub fn memparse(
    pub fn parse_option_str(
    pub fn next_arg(
    pub fn core_kernel_text(addr: ::core::ffi::c_ulong) -> ::core::ffi::c_int;
    pub fn __kernel_text_address(addr: ::core::ffi::c_ulong) -> ::core::ffi::c_int;
    pub fn kernel_text_address(addr: ::core::ffi::c_ulong) -> ::core::ffi::c_int;
    pub fn func_ptr_is_kernel_text(ptr: *mut ::core::ffi::c_void) -> ::core::ffi::c_int;
    pub fn bust_spinlocks(yes: ::core::ffi::c_int);
    pub fn tracing_on();
    pub fn tracing_off();
    pub fn tracing_is_on() -> ::core::ffi::c_int;
    pub fn tracing_snapshot();
    pub fn tracing_snapshot_alloc();
    pub fn tracing_start();
    pub fn tracing_stop();
    pub fn __trace_bprintk(
    pub fn __trace_printk(
    pub fn __trace_bputs(
    pub fn __trace_puts(
    pub fn trace_dump_stack(skip: ::core::ffi::c_int);
    pub fn __ftrace_vbprintk(
    pub fn __ftrace_vprintk(
    pub fn ftrace_dump(oops_dump_mode: ftrace_dump_mode);

## 2. Available Kernel Structs
pub struct __BindgenBitfieldUnit<Storage> {
pub struct __IncompleteArrayField<T>(::core::marker::PhantomData<T>, [T; 0]);
pub struct __BindgenUnionField<T>(::core::marker::PhantomData<T>);
pub struct ftrace_branch_data {
pub struct ftrace_branch_data__bindgen_ty_1 {
pub struct ftrace_branch_data__bindgen_ty_1__bindgen_ty_1 {
pub struct ftrace_branch_data__bindgen_ty_1__bindgen_ty_2 {
pub struct ftrace_likely_data {
pub struct __kernel_fd_set {
pub struct __kernel_fsid_t {
pub struct atomic_t {
pub struct atomic64_t {
pub struct rcuref_t {
pub struct list_head {
pub struct hlist_head {
pub struct hlist_node {
pub struct ustat {
pub struct callback_head {
pub struct kcsan_scoped_access {}
pub struct sysinfo {
pub struct alt_instr {
pub struct alt_instr__bindgen_ty_1 {
pub struct alt_instr__bindgen_ty_1__bindgen_ty_1 {
pub struct module {
pub struct paravirt_patch_site {
pub struct callthunk_sites {
pub struct s16_fract {
pub struct u16_fract {
pub struct s32_fract {
pub struct u32_fract {
pub struct pt_regs {
pub struct taint_flag {
pub struct file_system_type {
pub struct obs_kernel_param {
pub struct qspinlock {
pub struct qspinlock__bindgen_ty_1 {
pub struct qspinlock__bindgen_ty_1__bindgen_ty_1 {
pub struct qspinlock__bindgen_ty_1__bindgen_ty_2 {
pub struct qrwlock {
pub struct qrwlock__bindgen_ty_1 {
pub struct qrwlock__bindgen_ty_1__bindgen_ty_1 {
pub struct lock_class_key {}
pub struct lockdep_map {}
pub struct pin_cookie {}
pub struct raw_spinlock {
pub struct ratelimit_state {
pub struct ctl_table {
pub struct va_format {
pub struct dev_printk_info {
pub struct static_key {
pub struct static_key__bindgen_ty_1 {
pub struct jump_entry {
pub struct static_key_true {
pub struct static_key_false {
pub struct _ddebug {
pub struct _ddebug__bindgen_ty_1 {
pub struct ddebug_class_map {
pub struct _ddebug_info {
pub struct ddebug_class_param {
pub struct ddebug_class_param__bindgen_ty_1 {
pub struct device {
pub struct net_device {
pub struct ib_device {
pub struct kernel_param {
pub struct kernel_param_ops {
pub struct file_operations {
pub struct static_call_site {
pub struct static_call_key {
pub struct static_call_key__bindgen_ty_1 {
pub struct completion {
pub struct user {
pub struct __va_list_tag {
pub struct static_key_mod {
pub struct static_call_mod {
