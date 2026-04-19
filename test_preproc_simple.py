#!/usr/bin/env python3
"""测试预处理器是否正确处理条件编译"""

import subprocess
import sys

# 创建测试文件
test_content = '''#ifdef _WIN32
/* Windows */
int windows_func();
#else
/* Linux */
int linux_func();
#endif

public class Test {
    public static void main() {
    }
}
'''

with open('test_preproc.cay', 'w') as f:
    f.write(test_content)

# 运行 cay-check 进行完整检查
result = subprocess.run(
    ['cargo', 'run', '--release', '--bin', 'cay-check', '--', 'test_preproc.cay'],
    capture_output=True,
    text=True
)

print("STDOUT:")
print(result.stdout)
print("\nSTDERR:")
print(result.stderr)
print("\nReturn code:", result.returncode)
