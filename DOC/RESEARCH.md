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




### Evidence: Kernel Level Trace
<img width="899" height="439" alt="docker_signal_trace_04" src="https://github.com/user-attachments/assets/7286d4e6-f8f4-42dc-9955-19ca4a2bfb0d" />
<img width="903" height="573" alt="docker_signal_trace_03" src="https://github.com/user-attachments/assets/fa70ce0c-f76c-4850-8c68-ae253eae2ec2" />
<img width="913" height="590" alt="docker_signal_trace_02" src="https://github.com/user-attachments/assets/eb4d3121-5b85-4454-8a94-dde1fddd3cd4" />
<img width="926" height="599" alt="docker_signal_trace_01" src="https://github.com/user-attachments/assets/bc0207e2-6484-4d33-ac5c-64dfbd06259a" />





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
