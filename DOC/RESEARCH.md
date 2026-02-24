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
