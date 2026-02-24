# Docker Binary Signal Analysis

## Discovery
Date: 2025-02-24
Method: bpftrace + kprobe

## Signal Pattern
```
COMM:libuv-worker | ADDR:0xffff9e4ac4aabbb0 | HEX: 05 00 00 01 01 00 00 00
COMM:libuv-worker | ADDR:0xffff9e4ac4aa3bd8 | HEX: 05 00 00 01 01 00 00 00
(... 100+ occurrences)
```

## Analysis
- Consistent 8-byte pattern
- Always from libuv-worker
- Zero-copy evidence
- High frequency (100+ times)

### 📸 Evidence: Kernel Level Trace

<img width="899" height="439" alt="스크린샷 2026-02-21 034409" src="https://github.com/user-attachments/assets/7bf2d5c8-1be9-46cc-9d7d-906900e92267" />
<img width="903" height="573" alt="스크린샷 2026-02-21 034352" src="https://github.com/user-attachments/assets/2df00c89-b36a-436b-8be0-b9237a108a62" />
<img width="913" height="590" alt="스크린샷 2026-02-21 034343" src="https://github.com/user-attachments/assets/78a46412-f9fd-4a07-b215-c1bf272edf39" />
<img width="926" height="599" alt="스크린샷 2026-02-21 034333" src="https://github.com/user-attachments/assets/bb2dd4ca-be2c-4ef7-a54e-f13d7b38d4c1" />



## Implications
- Docker internal communication
- Performance optimization opportunity
- Further research needed

## Reproduction
```bash
sudo bpftrace -e 'kprobe:tty_write /comm == "libuv-worker"/ {
    printf("COMM:%s | ADDR:%p | HEX:", comm, arg1);
    printf(" %02x %02x %02x %02x %02x %02x %02x %02x\n",
        *(uint8*)(arg1), *(uint8*)(arg1+1), ...);
}'
