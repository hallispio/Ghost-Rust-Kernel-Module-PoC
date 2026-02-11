python#!/usr/bin/env python3
# telemetry_export.py

import subprocess
import datetime
import re

def export_telemetry():
    # dmesg에서 GHOST 통계 추출
    result = subprocess.run(['dmesg'], capture_output=True, text=True)
    lines = [l for l in result.stdout.split('\n') if '[GHOST]' in l]
    
    # 타임스탬프
    timestamp = datetime.datetime.now().strftime('%Y%m%d_%H%M%S')
    filename = f'ghost_telemetry_{timestamp}.txt'
    
    with open(filename, 'w', encoding='utf-8') as f:
        f.write('=' * 60 + '\n')
        f.write('GHOST Shell Telemetry Report\n')
        f.write(f'Generated: {timestamp}\n')
        f.write('=' * 60 + '\n\n')
        
        for line in lines:
            f.write(line + '\n')
        
        f.write('\n' + '=' * 60 + '\n')
        f.write('End of Report\n')
        f.write('=' * 60 + '\n')
    
    print(f'✅ Telemetry exported: {filename}')

if __name__ == '__main__':
    export_telemetry()