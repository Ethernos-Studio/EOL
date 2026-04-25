# Cavvy FFI 使用指南

FFI（Foreign Function Interface）允许 Cavvy 代码调用 C 语言函数和系统库，实现与底层系统的无缝交互。

## 目录

- [概述](#概述)
- [基础用法](#基础用法)
- [类型映射](#类型映射)
- [标准库函数](#标准库函数)
- [系统调用](#系统调用)
- [Windows API](#windows-api)
- [Linux 系统调用](#linux-系统调用)
- [运行时辅助函数](#运行时辅助函数)
- [最佳实践](#最佳实践)
- [完整示例](#完整示例)

---

## 概述

Cavvy 通过 `extern` 关键字声明外部 C 函数，实现与 C 代码的互操作：

```cay
// 声明外部函数
extern int printf(String fmt, ...);
extern long malloc(int size);
extern void free(long ptr);

public int main() {
    printf("Hello from Cavvy!\n");
    
    long ptr = malloc(100);
    // 使用内存...
    free(ptr);
    
    return 0;
}
```

### 核心特性

- **零开销**：直接调用 C 函数，无额外封装层
- **类型安全**：编译时检查类型匹配
- **可变参数**：支持 C 风格可变参数函数
- **指针操作**：直接操作内存地址

---

## 基础用法

### 声明外部函数

```cay
// 基本声明
extern int functionName(int arg1, String arg2);

// 无参数
extern void initialize();

// 无返回值
extern void cleanup();

// 指针参数
extern int strlen(String s);
extern String strcpy(long dest, String src);

// 可变参数
extern int printf(String fmt, ...);
extern int sprintf(long buf, String fmt, ...);
```

### 调用外部函数

```cay
// 注意：printf, malloc, free 已由运行时库提供

public int main() {
    // 使用运行时提供的 println 替代 printf
    println("Hello from Cavvy!");
    
    // 内存分配测试
    long ptr = malloc(100);
    if (ptr != 0) {
        println("Allocated 100 bytes");
        free(ptr);
        println("Memory freed");
    }
    return 0;
}

extern long malloc(int size);
extern void free(long ptr);
```

### 处理指针

```cay
// 注意：calloc 已由运行时库提供

extern long realloc(long ptr, int size);
extern void memcpy(long dest, long src, int n);
extern void memset(long ptr, int value, int n);
extern int memcmp(long ptr1, long ptr2, int n);

public int main() {
    // malloc - 由运行时提供
    long ptr1 = malloc(100);
    println("malloc(100) = " + ptr1);
    
    // calloc - 由运行时提供，直接使用（不声明 extern）
    // 注意：calloc 在运行时已声明，但 Cavvy 需要知道其签名
    // 使用 malloc + memset 替代
    long ptr2 = malloc(80);
    memset(ptr2, 0, 80);
    println("calloc simulation (malloc + memset) = " + ptr2);
    
    // realloc
    long ptr3 = realloc(ptr1, 200);
    println("realloc(ptr1, 200) = " + ptr3);
    
    // memset
    memset(ptr2, 0, 80);
    println("memset completed");
    
    // free - 由运行时提供
    free(ptr2);
    free(ptr3);
    println("Memory freed");
    
    return 0;
}

extern long malloc(int size);
extern void free(long ptr);
extern long calloc(int num, int size);
```

---

## 类型映射

### 基本类型映射

| Cavvy 类型 | C 类型 | 说明 |
|------------|--------|------|
| `boolean` | `bool` / `int` | 布尔值 |
| `byte` | `char` / `int8_t` | 8位有符号整数 |
| `short` | `short` / `int16_t` | 16位有符号整数 |
| `int` | `int` / `int32_t` | 32位有符号整数 |
| `long` | `long long` / `int64_t` | 64位有符号整数 |
| `float` | `float` | 32位浮点数 |
| `double` | `double` | 64位浮点数 |
| `char` | `wchar_t` / `char16_t` | 16位字符 |
| `String` | `char*` | 以 null 结尾的字符串 |
| `void` | `void` | 无返回值 |

### 指针类型

```cay
// 在 Cavvy 中，指针使用 long 类型表示
long ptr = malloc(100);  // void* 在 Cavvy 中是 long

// 函数指针（使用 long）
extern int atexit(long func);  // int atexit(void (*func)(void));
```

### 字符串处理

```cay
// 注意：strlen, strcpy, strcmp 已由运行时库提供，不要重复声明

extern String strcat(long dest, String src);

public int main() {
    String message = "Hello, Cavvy!";
    
    // strlen - 由运行时提供，但 Cavvy 需要知道其签名
    // 使用 String.length() 替代
    int len = message.length();
    println("Length: " + len);
    
    // strcmp - 由运行时提供，但 Cavvy 需要知道其签名
    // 使用 String.equals() 替代，但避免 boolean 转字符串的问题
    String s1 = "abc";
    String s2 = "def";
    boolean eq = s1.equals(s2);
    if (eq) {
        println("Equals result: true");
    } else {
        println("Equals result: false");
    }
    
    println("String operations completed");
    return 0;
}
```

---

## 标准库函数

### stdio.h - 标准输入输出

```cay
// 文件操作
extern long fopen(String filename, String mode);
extern int fclose(long stream);
extern int fprintf(long stream, String fmt, ...);
extern int fscanf(long stream, String fmt, ...);

// 格式化输出
extern int printf(String fmt, ...);
extern int sprintf(long buf, String fmt, ...);
extern int snprintf(long buf, int size, String fmt, ...);

// 字符 IO
extern int fgetc(long stream);
extern int fputc(int c, long stream);
extern String fgets(long buf, int size, long stream);
extern int fputs(String s, long stream);

// 文件定位
extern int fseek(long stream, long offset, int whence);
extern long ftell(long stream);
extern void rewind(long stream);

public int main() {
    // 写入文件
    long file = fopen("test.txt", "w");
    fprintf(file, "Hello, %s!\n", "World");
    fclose(file);
    
    // 读取文件
    file = fopen("test.txt", "r");
    long buffer = malloc(256);
    fgets(buffer, 256, file);
    String content = __cay_ptr_to_string(buffer);
    println(content);
    free(buffer);
    fclose(file);
    
    return 0;
}
```

### stdlib.h - 标准库

```cay
// 内存分配
extern long malloc(int size);
extern long calloc(int num, int size);
extern void free(long ptr);
extern long realloc(long ptr, int size);

// 字符串转换
extern int atoi(String s);
extern long atol(String s);
extern double atof(String s);

// 随机数
extern int rand();
extern void srand(int seed);

// 程序控制
extern void exit(int status);
extern void abort();
extern int atexit(long func);
extern long getenv(String name);
extern int system(String command);

public int main() {
    // 随机数
    srand(12345);
    int r = rand() % 100;
    println("Random: " + r);
    
    // 环境变量
    long path = getenv("PATH");
    if (path != 0) {
        String pathStr = __cay_ptr_to_string(path);
        println("PATH: " + pathStr);
    }
    
    // 执行命令
    system("echo Hello from system");
    
    return 0;
}
```

### string.h - 字符串操作

```cay
// 字符串操作
extern int strlen(String s);
extern String strcpy(long dest, String src);
extern String strncpy(long dest, String src, int n);
extern String strcat(long dest, String src);
extern String strncat(long dest, String src, int n);
extern int strcmp(String s1, String s2);
extern int strncmp(String s1, String s2, int n);
extern String strchr(String s, int c);
extern String strrchr(String s, int c);
extern String strstr(String haystack, String needle);
extern String strtok(long str, String delim);

// 内存操作
extern void memcpy(long dest, long src, int n);
extern void memmove(long dest, long src, int n);
extern void memset(long ptr, int value, int n);
extern int memcmp(long ptr1, long ptr2, int n);
extern long memchr(long ptr, int value, int n);

public int main() {
    // 字符串操作
    String s1 = "Hello";
    String s2 = "World";
    
    int len = strlen(s1);
    int cmp = strcmp(s1, s2);
    
    // 内存操作
    long buf1 = malloc(100);
    long buf2 = malloc(100);
    
    memset(buf1, 0, 100);
    memcpy(buf2, buf1, 100);
    
    free(buf1);
    free(buf2);
    
    return 0;
}
```

### math.h - 数学函数

```cay
// 三角函数
extern double sin(double x);
extern double cos(double x);
extern double tan(double x);
extern double asin(double x);
extern double acos(double x);
extern double atan(double x);
extern double atan2(double y, double x);

// 双曲函数
extern double sinh(double x);
extern double cosh(double x);
extern double tanh(double x);

// 指数和对数
extern double exp(double x);
extern double log(double x);
extern double log10(double x);
extern double pow(double x, double y);
extern double sqrt(double x);
extern double cbrt(double x);

// 取整函数
extern double ceil(double x);
extern double floor(double x);
extern double round(double x);
extern double trunc(double x);

// 其他
extern double fabs(double x);
extern double fmod(double x, double y);
extern double modf(double x, long iptr);
extern double frexp(double x, long exp);

public int main() {
    double x = 3.14159;
    
    println("sin(" + x + ") = " + sin(x));
    println("sqrt(2) = " + sqrt(2.0));
    println("pow(2, 10) = " + pow(2.0, 10.0));
    println("log(100) = " + log(100.0));
    
    return 0;
}
```

### time.h - 时间函数

```cay
// 时间类型
public class TimeVal {
    public long tv_sec;   // 秒
    public long tv_usec;  // 微秒
}

// 时间函数
extern long time(long ptr);
extern long clock();
extern double difftime(long end, long start);
extern long mktime(long tm_ptr);
extern String ctime(long time_ptr);
extern String asctime(long tm_ptr);
extern long gmtime(long time_ptr);
extern long localtime(long time_ptr);
extern int gettimeofday(long tv, long tz);

// 时间常量
#define CLOCKS_PER_SEC 1000

public int main() {
    // 获取当前时间
    long now = time(0);
    println("Current time: " + now);
    
    // 测量代码执行时间
    long start = clock();
    
    // 执行一些操作...
    for (int i = 0; i < 1000000; i++) {
        // 空循环
    }
    
    long end = clock();
    double elapsed = (end - start) * 1.0 / CLOCKS_PER_SEC;
    println("Elapsed: " + elapsed + " seconds");
    
    return 0;
}
```

---

## 系统调用

### 文件系统操作

```cay
// POSIX 文件操作
extern int open(String pathname, int flags, int mode);
extern int close(int fd);
extern int read(int fd, long buf, int count);
extern int write(int fd, long buf, int count);
extern long lseek(int fd, long offset, int whence);
extern int stat(String pathname, long buf);
extern int fstat(int fd, long buf);
extern int lstat(String pathname, long buf);

// 文件权限
extern int chmod(String pathname, int mode);
extern int chown(String pathname, int owner, int group);

// 目录操作
extern int mkdir(String pathname, int mode);
extern int rmdir(String pathname);
extern int chdir(String path);
extern String getcwd(long buf, int size);
extern long opendir(String name);
extern int closedir(long dirp);
extern long readdir(long dirp);

public int main() {
    // 创建目录
    mkdir("test_dir", 0755);
    
    // 切换目录
    chdir("test_dir");
    
    // 获取当前目录
    long cwd = malloc(256);
    getcwd(cwd, 256);
    println("Current dir: " + __cay_ptr_to_string(cwd));
    free(cwd);
    
    // 返回上级
    chdir("..");
    rmdir("test_dir");
    
    return 0;
}
```

### 进程控制

```cay
// 进程管理
extern int fork();
extern int execve(String filename, long argv, long envp);
extern int execv(String pathname, long argv);
extern int execvp(String file, long argv);
extern void exit(int status);
extern int wait(long status);
extern int waitpid(int pid, long status, int options);
extern int system(String command);

// 进程信息
extern int getpid();
extern int getppid();
extern int getuid();
extern int getgid();

public int main() {
    println("PID: " + getpid());
    println("Parent PID: " + getppid());
    
    // 执行命令
    int status = system("ls -la");
    println("Exit status: " + status);
    
    return 0;
}
```

---

## Windows API

### 基础 Windows API

```cay
#ifdef _WIN32

// Windows 类型定义
#define WINAPI
#define NULL 0

// 内核函数
extern int GetLastError();
extern void Sleep(int dwMilliseconds);
extern int GetTickCount();
extern int GetCurrentProcessId();
extern int GetCurrentThreadId();

// 控制台函数
extern int SetConsoleOutputCP(int wCodePageID);
extern int GetConsoleMode(long hConsoleHandle, long lpMode);
extern int SetConsoleMode(long hConsoleHandle, int dwMode);
extern long GetStdHandle(int nStdHandle);

#define STD_INPUT_HANDLE  -10
#define STD_OUTPUT_HANDLE -11
#define STD_ERROR_HANDLE  -12

// 内存函数
extern long VirtualAlloc(long lpAddress, int dwSize, int flAllocationType, int flProtect);
extern int VirtualFree(long lpAddress, int dwSize, int dwFreeType);
extern void RtlMoveMemory(long Destination, long Source, int Length);
extern void RtlZeroMemory(long Destination, int Length);

// 文件函数
extern long CreateFileA(String lpFileName, int dwDesiredAccess, int dwShareMode, 
                        long lpSecurityAttributes, int dwCreationDisposition, 
                        int dwFlagsAndAttributes, long hTemplateFile);
extern int ReadFile(long hFile, long lpBuffer, int nNumberOfBytesToRead, 
                    long lpNumberOfBytesRead, long lpOverlapped);
extern int WriteFile(long hFile, long lpBuffer, int nNumberOfBytesToWrite,
                     long lpNumberOfBytesWritten, long lpOverlapped);
extern int CloseHandle(long hObject);

public int main() {
    // 设置控制台代码页为 UTF-8
    SetConsoleOutputCP(65001);
    
    // 获取标准输出句柄
    long stdout = GetStdHandle(STD_OUTPUT_HANDLE);
    
    // 写入控制台
    String msg = "Hello from Windows API!\n";
    long written = malloc(4);
    WriteFile(stdout, __cay_string_to_ptr(msg), strlen(msg), written, 0);
    free(written);
    
    // 休眠 1 秒
    Sleep(1000);
    
    return 0;
}

#endif
```

### Windows 套接字

```cay
#ifdef _WIN32

// Winsock 函数
extern int WSAStartup(int wVersionRequired, long lpWSAData);
extern int WSACleanup();
extern long socket(int af, int type, int protocol);
extern int bind(long s, long addr, int namelen);
extern int listen(long s, int backlog);
extern long accept(long s, long addr, long addrlen);
extern int connect(long s, long addr, int namelen);
extern int send(long s, long buf, int len, int flags);
extern int recv(long s, long buf, int len, int flags);
extern int closesocket(long s);
extern int shutdown(long s, int how);

// 地址族
#define AF_INET     2
#define AF_INET6    23

// Socket 类型
#define SOCK_STREAM 1
#define SOCK_DGRAM  2

// 协议
#define IPPROTO_TCP 6
#define IPPROTO_UDP 17

// 错误码
#define SOCKET_ERROR -1
#define INVALID_SOCKET -1

public int main() {
    // 初始化 Winsock
    long wsaData = malloc(400);
    WSAStartup(0x0202, wsaData);
    
    // 创建 socket
    long sock = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    
    // ... 使用 socket ...
    
    // 清理
    closesocket(sock);
    WSACleanup();
    free(wsaData);
    
    return 0;
}

#endif
```

---

## Linux 系统调用

### Linux 特定 API

```cay
#ifndef _WIN32

// Linux 系统调用
extern int syscall(int number, ...);

// 常用系统调用号 (x86_64)
#define SYS_read        0
#define SYS_write       1
#define SYS_open        2
#define SYS_close       3
#define SYS_stat        4
#define SYS_fstat       5
#define SYS_lstat       6
#define SYS_poll        7
#define SYS_lseek       8
#define SYS_mmap        9
#define SYS_mprotect    10
#define SYS_munmap      11

// epoll (Linux 高性能 IO)
extern int epoll_create1(int flags);
extern int epoll_ctl(int epfd, int op, int fd, long event);
extern int epoll_wait(int epfd, long events, int maxevents, int timeout);

// 内存映射
extern long mmap(long addr, int length, int prot, int flags, int fd, long offset);
extern int munmap(long addr, int length);
extern int mprotect(long addr, int len, int prot);

// 线程
extern int pthread_create(long thread, long attr, long start_routine, long arg);
extern int pthread_join(long thread, long retval);
extern int pthread_exit(long retval);

public int main() {
    // 直接使用系统调用
    String msg = "Hello from syscall!\n";
    syscall(SYS_write, 1, __cay_string_to_ptr(msg), strlen(msg));
    
    return 0;
}

#endif
```

---

## 运行时辅助函数

Cavvy 运行时提供了一些辅助函数，用于在 Cavvy 和 C 之间转换数据：

### 内存读写函数

```cay
// 从指针读取值（运行时内置）
extern int __cay_read_int(long ptr);       // 从指定地址读取32位整数
extern long __cay_read_ptr(long ptr);      // 读取指针值

// 向指针写入值（运行时内置）
extern void __cay_write_int(long ptr, int value);   // 向指定地址写入32位整数
extern void __cay_write_byte(long ptr, int value);  // 向指定地址写入8位字节
extern void __cay_write_ptr(long ptr, long value);  // 写入指针值
```

### 字符串转换函数

```cay
// 指针与字符串转换（运行时内置）
extern String __cay_ptr_to_string(long ptr);    // C char* -> Cavvy String
extern long __cay_string_to_ptr(String s);      // Cavvy String -> C char*
```

### 使用示例

```cay
public int main() {
    // 分配内存
    long buffer = malloc(100);
    
    // 写入 int（使用指针运算计算地址）
    for (int i = 0; i < 10; i++) {
        long addr = buffer + i * 4;  // int 占4字节
        __cay_write_int(addr, i * 10);
    }
    
    // 读取值
    for (int i = 0; i < 10; i++) {
        long addr = buffer + i * 4;
        int val = __cay_read_int(addr);
        println("buffer[" + i + "] = " + val);
    }
    
    // 写入字符串
    String msg = "Hello, FFI!";
    long strPtr = __cay_string_to_ptr(msg);
    strcpy(buffer, strPtr);
    
    // 读取字符串
    String result = __cay_ptr_to_string(buffer);
    println("Message: " + result);
    
    free(buffer);
    return 0;
}
```

---

## 最佳实践

### 1. 封装外部函数

```cay
// 不要直接到处使用 extern，而是封装成 Cavvy 类

public class FileUtils {
    // 私有外部声明
    extern static long fopen(String filename, String mode);
    extern static int fclose(long file);
    extern static int fprintf(long file, String fmt, ...);
    extern static String fgets(long buf, int size, long file);
    
    // 公共 Cavvy 接口
    public static boolean writeFile(String filename, String content) {
        long file = fopen(filename, "w");
        if (file == 0) return false;
        
        fprintf(file, "%s", content);
        fclose(file);
        return true;
    }
    
    public static String readFile(String filename) {
        long file = fopen(filename, "r");
        if (file == 0) return null;
        
        long buffer = malloc(4096);
        StringBuilder sb = new StringBuilder();
        
        while (fgets(buffer, 4096, file) != 0) {
            sb.append(__cay_ptr_to_string(buffer));
        }
        
        free(buffer);
        fclose(file);
        
        return sb.toString();
    }
}
```

### 2. 错误处理

```cay
public class SafeMemory {
    extern static long malloc(int size);
    extern static void free(long ptr);
    extern static int errno();
    extern static String strerror(int errnum);
    
    public static long allocate(int size) {
        long ptr = malloc(size);
        if (ptr == 0) {
            int err = errno();
            println("Memory allocation failed: " + strerror(err));
            return 0;
        }
        return ptr;
    }
    
    public static void release(long ptr) {
        if (ptr != 0) {
            free(ptr);
        }
    }
}
```

### 3. 资源管理（RAII）

```cay
public class CFile implements Disposable {
    private long handle;
    
    extern static long fopen(String filename, String mode);
    extern static int fclose(long file);
    extern static int fseek(long file, long offset, int whence);
    extern static long ftell(long file);
    
    public CFile(String filename, String mode) {
        this.handle = fopen(filename, mode);
    }
    
    public boolean isOpen() {
        return handle != 0;
    }
    
    public void seek(long offset) {
        if (handle != 0) {
            fseek(handle, offset, 0);  // SEEK_SET
        }
    }
    
    public long position() {
        if (handle != 0) {
            return ftell(handle);
        }
        return -1;
    }
    
    public void dispose() {
        if (handle != 0) {
            fclose(handle);
            handle = 0;
        }
    }
}

// 使用
public void processFile(String filename) {
    CFile file = new CFile(filename, "r");
    if (!file.isOpen()) {
        println("Failed to open: " + filename);
        return;
    }
    
    // 使用文件...
    
    file.dispose();  // 确保关闭
}
```

### 4. 平台兼容性

```cay
// 使用条件编译处理平台差异

#ifdef _WIN32
    extern int _mkdir(String pathname);
    #define mkdir(path) _mkdir(path)
#else
    extern int mkdir(String pathname, int mode);
#endif

public class CrossPlatformDir {
    public static boolean create(String pathname) {
        #ifdef _WIN32
            return _mkdir(pathname) == 0;
        #else
            return mkdir(pathname, 0755) == 0;
        #endif
    }
}
```

---

## 完整示例

### 示例 1：文件加密工具

```cay
/**
 * 简单的 XOR 文件加密工具
 * 演示 FFI 文件操作
 */

extern long fopen(String filename, String mode);
extern int fclose(long file);
extern int fgetc(long file);
extern int fputc(int c, long file);
extern int feof(long file);
extern int ferror(long file);

public class XorEncryptor {
    private byte key;
    
    public XorEncryptor(byte key) {
        this.key = key;
    }
    
    public boolean encrypt(String inputFile, String outputFile) {
        long in = fopen(inputFile, "rb");
        if (in == 0) {
            println("Error: Cannot open input file");
            return false;
        }
        
        long out = fopen(outputFile, "wb");
        if (out == 0) {
            fclose(in);
            println("Error: Cannot create output file");
            return false;
        }
        
        while (true) {
            int c = fgetc(in);
            if (feof(in) != 0) break;
            if (ferror(in) != 0) {
                println("Error: Read error");
                break;
            }
            
            fputc(c ^ key, out);
        }
        
        fclose(out);
        fclose(in);
        
        println("Encryption complete: " + outputFile);
        return true;
    }
}

public int main(String[] args) {
    if (args.length < 3) {
        println("Usage: xor_encrypt <input> <output> <key>");
        return 1;
    }
    
    String inputFile = args[0];
    String outputFile = args[1];
    byte key = (byte)Integer.parseInt(args[2]);
    
    XorEncryptor encryptor = new XorEncryptor(key);
    encryptor.encrypt(inputFile, outputFile);
    
    return 0;
}
```

### 示例 2：系统信息获取

```cay
/**
 * 获取系统信息
 * 演示跨平台 FFI 使用
 */

#ifdef _WIN32
    // Windows 特定
    extern void GetSystemInfo(long lpSystemInfo);
    extern int GetTickCount();
    extern int GetCurrentProcessId();
    
    public class SystemInfo {
        public static long getUptime() {
            return GetTickCount() / 1000;  // 转换为秒
        }
        
        public static int getProcessId() {
            return GetCurrentProcessId();
        }
    }
#else
    // Linux/Unix 特定
    extern int sysinfo(long info);
    extern int getpid();
    
    public class SystemInfo {
        public static long getUptime() {
            long info = malloc(128);
            sysinfo(info);
            long uptime = __cay_read_long(info, 0);  // uptime 在结构体开头
            free(info);
            return uptime;
        }
        
        public static int getProcessId() {
            return getpid();
        }
    }
#endif

public int main() {
    println("Process ID: " + SystemInfo.getProcessId());
    println("System Uptime: " + SystemInfo.getUptime() + " seconds");
    return 0;
}
```

### 示例 3：高性能网络服务器

```cay
/**
 * 高性能 Echo 服务器
 * 演示 socket FFI 使用
 */

#include <Network.cay>

public class EchoServer {
    private TcpServer server;
    private int port;
    private boolean running;
    
    public EchoServer(int port) {
        this.port = port;
        this.server = new TcpServer();
    }
    
    public boolean start() {
        if (!server.bind("0.0.0.0", port)) {
            println("Failed to bind to port " + port);
            return false;
        }
        
        if (!server.listen(100)) {
            println("Failed to listen");
            return false;
        }
        
        running = true;
        println("Echo server started on port " + port);
        
        while (running) {
            TcpSocket client = server.accept();
            if (client != null && client.isValid()) {
                handleClient(client);
            }
        }
        
        return true;
    }
    
    private void handleClient(TcpSocket client) {
        println("Client connected: " + client.getRemoteAddress());
        
        while (true) {
            String data = client.receive(1024);
            if (data == null || data.length() == 0) {
                break;
            }
            
            println("Received: " + data.trim());
            
            // Echo back
            if (!client.send("Echo: " + data)) {
                break;
            }
        }
        
        println("Client disconnected");
        client.close();
    }
    
    public void stop() {
        running = false;
        server.close();
    }
}

public int main(String[] args) {
    int port = 8080;
    if (args.length > 0) {
        port = Integer.parseInt(args[0]);
    }
    
    EchoServer server = new EchoServer(port);
    
    // 在另一个线程或信号处理中设置停止标志
    server.start();
    
    return 0;
}
```

---

## 相关文档

- [快速开始](quickstart.md) - 5 分钟上手 Cavvy
- [语言文档](language-guide.md) - 深入了解 Cavvy 语言特性
- [语法参考](syntax-reference.md) - 完整的语法规范
