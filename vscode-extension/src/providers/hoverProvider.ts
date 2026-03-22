import * as vscode from 'vscode';

/**
 * Hover 提供器
 * 提供 Cavvy 语言的悬停提示信息
 */
export class CavvyHoverProvider implements vscode.HoverProvider {

    // 关键字文档 - 更新到最新语法
    private keywordDocs: Map<string, string> = new Map([
        // 访问修饰符
        ['public', '**public** - 访问修饰符，表示公开的，任何地方都可以访问。\n\n```cavvy\npublic class MyClass {\n    public int value;\n}\n```'],
        ['private', '**private** - 访问修饰符，表示私有的，只能在类内部访问。\n\n```cavvy\npublic class MyClass {\n    private int secret;\n}\n```'],
        ['protected', '**protected** - 访问修饰符，表示受保护的，可在类内部和子类中访问。'],

        // 修饰符
        ['static', '**static** - 静态修饰符，表示属于类而不是实例。\n\n```cavvy\npublic static void main() {\n    // 静态方法\n}\n```'],
        ['final', '**final** - 最终修饰符，表示不可修改（常量）。\n\n```cavvy\nfinal int MAX_SIZE = 100;\n```'],
        ['abstract', '**abstract** - 抽象修饰符，用于抽象类和抽象方法。\n\n```cavvy\npublic abstract class Shape {\n    public abstract void draw();\n}\n```'],
        ['native', '**native** - 本地方法修饰符，表示由外部实现。'],
        ['Override', '**@Override** - 注解，表示方法重写。编译器会检查父类是否存在该方法。'],

        // 类型声明
        ['class', '**class** - 用于声明类。\n\n```cavvy\npublic class MyClass {\n    // 类体\n}\n```'],
        ['interface', '**interface** - 用于声明接口。\n\n```cavvy\npublic interface Drawable {\n    void draw();\n}\n```'],
        ['enum', '**enum** - 用于声明枚举类型。'],
        ['extends', '**extends** - 继承父类。\n\n```cavvy\npublic class Child extends Parent {\n    // ...\n}\n```'],
        ['implements', '**implements** - 实现接口。\n\n```cavvy\npublic class MyClass implements MyInterface {\n    // ...\n}\n```'],
        ['namespace', '**namespace** - 命名空间，用于组织代码。'],

        // 基本类型
        ['void', '**void** - 表示无返回值。\n\n```cavvy\npublic void doSomething() {\n    // 无返回值\n}\n```'],
        ['int', '**int** - 32位整数类型。\n\n范围: -2,147,483,648 到 2,147,483,647\n\n```cavvy\nint count = 10;\n```'],
        ['long', '**long** - 64位整数类型。\n\n范围: -9,223,372,036,854,775,808 到 9,223,372,036,854,775,807\n\n```cavvy\nlong bigNumber = 10000000000L;\n```'],
        ['float', '**float** - 32位单精度浮点数。\n\n```cavvy\nfloat price = 19.99f;\n```'],
        ['double', '**double** - 64位双精度浮点数。\n\n```cavvy\ndouble precise = 3.14159265359;\n```'],
        ['bool', '**bool** - 布尔类型，值为 `true` 或 `false`。\n\n```cavvy\nbool isReady = true;\n```'],
        ['boolean', '**boolean** - 布尔类型的别名，等同于 `bool`。'],
        ['char', '**char** - 16位 Unicode 字符。\n\n```cavvy\nchar letter = \'A\';\n```'],
        ['string', '**string** - 字符串类型。\n\n```cavvy\nstring message = "Hello, World!";\n```'],

        // 现代类型声明
        ['var', '**var** - 变量声明关键字，类型后置（Cavvy 0.4.3+）。\n\n```cavvy\nvar x: int = 10;\nvar name: String = "Cavvy";\n```'],
        ['let', '**let** - 变量声明关键字，与 var 相同（Cavvy 0.4.3+）。\n\n```cavvy\nlet y: int = 20;\n```'],
        ['auto', '**auto** - 自动类型推断（Cavvy 0.4.3+）。\n\n```cavvy\nauto x = 10;        // 推断为 int\nauto d = 3.14;      // 推断为 double\nauto s = "hello";   // 推断为 string\n```'],

        // 控制流
        ['if', '**if** - 条件语句。\n\n```cavvy\nif (condition) {\n    // 条件为真时执行\n}\n```'],
        ['else', '**else** - 与 if 配合使用，条件为假时执行。\n\n```cavvy\nif (condition) {\n    // 条件为真\n} else {\n    // 条件为假\n}\n```'],
        ['while', '**while** - 循环语句，条件为真时重复执行。\n\n```cavvy\nwhile (condition) {\n    // 循环体\n}\n```'],
        ['for', '**for** - 循环语句，用于已知次数的循环。\n\n```cavvy\nfor (int i = 0; i < 10; i++) {\n    // 循环体\n}\n```'],
        ['do', '**do** - do-while 循环的开头。\n\n```cavvy\ndo {\n    // 循环体\n} while (condition);\n```'],
        ['switch', '**switch** - 多分支选择语句。\n\n```cavvy\nswitch (value) {\n    case 1:\n        // ...\n        break;\n    default:\n        // ...\n}\n```'],
        ['case', '**case** - switch 语句中的分支标签。\n\n```cavvy\ncase 1:\n    println("One");\n    break;\n```'],
        ['default', '**default** - switch 语句中的默认分支。\n\n```cavvy\ndefault:\n    println("Other");\n    break;\n```'],
        ['break', '**break** - 跳出循环或 switch 语句。\n\n```cavvy\nwhile (true) {\n    if (done) break;\n}\n```'],
        ['continue', '**continue** - 跳过当前循环迭代，继续下一次。\n\n```cavvy\nfor (int i = 0; i < 10; i++) {\n    if (i == 5) continue;\n    println(i);\n}\n```'],
        ['return', '**return** - 从方法返回，可带返回值。\n\n```cavvy\nreturn 42;\nreturn;  // 无返回值\n```'],

        // 其他关键字
        ['new', '**new** - 创建新对象或数组。\n\n```cavvy\nint[] arr = new int[10];\n```'],
        ['null', '**null** - 空引用。\n\n```cavvy\nstring s = null;\n```'],
        ['true', '**true** - 布尔真值。'],
        ['false', '**false** - 布尔假值。'],
        ['this', '**this** - 引用当前对象实例。'],
        ['super', '**super** - 引用父类。\n\n```cavvy\npublic Child() {\n    super();  // 调用父类构造函数\n}\n```'],
        ['instanceof', '**instanceof** - 类型检查运算符。\n\n```cavvy\nif (obj instanceof String) {\n    // obj 是 String 类型\n}\n```'],
        ['extern', '**extern** - 声明外部 C 函数（FFI）。\n\n```cavvy\nextern {\n    c_int printf(c_int fmt, ...);\n}\n```'],
        ['scope', '**scope** - 栈作用域块（Cavvy 0.5.0+）。\n\n```cavvy\nscope {\n    int x = 10;\n    // x 只在这个作用域内有效\n}\n```']
    ]);

    // 预处理器指令文档
    private preprocessorDocs: Map<string, { signature: string; description: string; example: string }> = new Map([
        ['#define', {
            signature: '#define MACRO [value]',
            description: '定义一个预处理器宏。可以用于条件编译或简单的文本替换。',
            example: '#define DEBUG\n#define VERSION "0.5.0.0"\n#define MAX_SIZE 100'
        }],
        ['#ifdef', {
            signature: '#ifdef MACRO',
            description: '条件编译：如果指定的宏已定义，则包含后续代码块。必须以 #endif 结束。',
            example: '#define DEBUG\n\n#ifdef DEBUG\n    println("Debug mode enabled");\n    // 调试代码\n#endif'
        }],
        ['#ifndef', {
            signature: '#ifndef MACRO',
            description: '条件编译：如果指定的宏未定义，则包含后续代码块。必须以 #endif 结束。',
            example: '#ifndef RELEASE\n    println("Development mode");\n    // 开发环境代码\n#endif'
        }],
        ['#if', {
            signature: '#if expression',
            description: '条件编译：基于常量表达式。',
            example: '#define VERSION 500\n\n#if VERSION > 100\n    // 版本大于 1.0.0\n#endif'
        }],
        ['#elif', {
            signature: '#elif expression',
            description: '条件编译：else if 分支。',
            example: '#ifdef DEBUG\n    // 调试代码\n#elif defined(LOG_LEVEL)\n    // 日志代码\n#endif'
        }],
        ['#else', {
            signature: '#else',
            description: '条件编译：else 分支。',
            example: '#ifdef DEBUG\n    // 调试代码\n#else\n    // 发布代码\n#endif'
        }],
        ['#endif', {
            signature: '#endif',
            description: '结束条件编译块。与 #ifdef、#ifndef、#if 配对使用。',
            example: '#ifdef DEBUG\n    // 调试代码\n#endif  // 结束条件编译'
        }],
        ['#undef', {
            signature: '#undef MACRO',
            description: '取消定义一个已定义的宏。',
            example: '#define DEBUG\n// ... 使用 DEBUG ...\n#undef DEBUG  // 取消定义'
        }],
        ['#include', {
            signature: '#include <header> or #include "header"',
            description: '包含头文件或 Cavvy 源文件。',
            example: '#include <stdio.h>\n#include "MyClass.cay"'
        }]
    ]);

    // 内置方法文档 - 更新到最新语法
    private methodDocs: Map<string, { signature: string; description: string; example: string }> = new Map([
        // 输出函数
        ['print', {
            signature: 'print(value: any) -> void',
            description: '打印值到控制台，不换行。',
            example: 'print("Hello");\nprint(42);'
        }],
        ['println', {
            signature: 'println(value: any) -> void',
            description: '打印值到控制台，并在末尾添加换行符。',
            example: 'println("Hello, World!");\nprintln(123);'
        }],
        ['printf', {
            signature: 'printf(format: string, ...) -> void',
            description: '格式化输出，类似 C 语言 printf。',
            example: 'printf("Name: %s, Age: %d", name, age);'
        }],

        // 输入函数
        ['readInt', {
            signature: 'readInt() -> int',
            description: '从标准输入读取一个整数，返回 int 类型。',
            example: 'int num = readInt();\nprintln("You entered: " + num);'
        }],
        ['readLong', {
            signature: 'readLong() -> long',
            description: '从标准输入读取一个长整数，返回 long 类型。',
            example: 'long num = readLong();\nprintln("You entered: " + num);'
        }],
        ['readFloat', {
            signature: 'readFloat() -> float',
            description: '从标准输入读取一个浮点数，返回 float 类型。',
            example: 'float val = readFloat();\nprintln("Value: " + val);'
        }],
        ['readDouble', {
            signature: 'readDouble() -> double',
            description: '从标准输入读取一个双精度浮点数，返回 double 类型。',
            example: 'double val = readDouble();\nprintln("Value: " + val);'
        }],
        ['readLine', {
            signature: 'readLine() -> string',
            description: '从标准输入读取一行字符串。',
            example: 'string name = readLine();\nprintln("Hello, " + name);'
        }],
        ['readChar', {
            signature: 'readChar() -> char',
            description: '从标准输入读取一个字符。',
            example: 'char c = readChar();\nprintln("Char: " + c);'
        }],

        // 类型转换函数
        ['parseInt', {
            signature: 'parseInt(s: string) -> int',
            description: '将字符串转换为整数。',
            example: 'int num = parseInt("42");'
        }],
        ['parseLong', {
            signature: 'parseLong(s: string) -> long',
            description: '将字符串转换为长整数。',
            example: 'long num = parseLong("9999999999");'
        }],
        ['parseFloat', {
            signature: 'parseFloat(s: string) -> float',
            description: '将字符串转换为浮点数。',
            example: 'float f = parseFloat("3.14");'
        }],
        ['parseDouble', {
            signature: 'parseDouble(s: string) -> double',
            description: '将字符串转换为双精度浮点数。',
            example: 'double d = parseDouble("3.14159265359");'
        }],

        // 字符串方法
        ['length', {
            signature: 'length() -> int',
            description: '返回字符串或数组的长度。',
            example: 'string s = "hello";\nint len = s.length();  // 5\nint[] arr = new int[10];\nint arrLen = arr.length;  // 10'
        }],
        ['charAt', {
            signature: 'charAt(index: int) -> char',
            description: '返回字符串指定位置的字符。索引从 0 开始。',
            example: 'string s = "hello";\nchar c = s.charAt(1);  // \'e\''
        }],
        ['indexOf', {
            signature: 'indexOf(str: string) -> int',
            description: '查找子字符串在字符串中的位置。如果未找到返回 -1。',
            example: 'string s = "hello world";\nint pos = s.indexOf("world");  // 6\nint notFound = s.indexOf("xyz");  // -1'
        }],
        ['substring', {
            signature: 'substring(start: int, end?: int) -> string',
            description: '返回从 start（包含）到 end（不包含）的子字符串。如果省略 end，则返回到字符串末尾。',
            example: 'string s = "hello world";\nstring sub1 = s.substring(0, 5);   // "hello"\nstring sub2 = s.substring(6);      // "world"'
        }],
        ['concat', {
            signature: 'concat(str: string) -> string',
            description: '将指定字符串连接到当前字符串末尾。',
            example: 'string s = "hello";\nstring result = s.concat(" world");  // "hello world"'
        }],
        ['replace', {
            signature: 'replace(old: string, new: string) -> string',
            description: '替换字符串中所有匹配的子字符串。',
            example: 'string s = "hello world";\nstring result = s.replace("world", "Cavvy");  // "hello Cavvy"'
        }],
        ['toLowerCase', {
            signature: 'toLowerCase() -> string',
            description: '将字符串转换为小写。',
            example: 'string s = "HELLO";\nstring lower = s.toLowerCase();  // "hello"'
        }],
        ['toUpperCase', {
            signature: 'toUpperCase() -> string',
            description: '将字符串转换为大写。',
            example: 'string s = "hello";\nstring upper = s.toUpperCase();  // "HELLO"'
        }],
        ['trim', {
            signature: 'trim() -> string',
            description: '去除字符串首尾的空白字符。',
            example: 'string s = "  hello  ";\nstring trimmed = s.trim();  // "hello"'
        }],
        ['startsWith', {
            signature: 'startsWith(prefix: string) -> bool',
            description: '检查字符串是否以指定前缀开头。',
            example: 'string s = "hello world";\nbool result = s.startsWith("hello");  // true'
        }],
        ['endsWith', {
            signature: 'endsWith(suffix: string) -> bool',
            description: '检查字符串是否以指定后缀结尾。',
            example: 'string s = "hello world";\nbool result = s.endsWith("world");  // true'
        }],
        ['contains', {
            signature: 'contains(str: string) -> bool',
            description: '检查字符串是否包含指定子串。',
            example: 'string s = "hello world";\nbool result = s.contains("lo wo");  // true'
        }],
        ['equals', {
            signature: 'equals(str: string) -> bool',
            description: '比较两个字符串是否相等。',
            example: 'string s1 = "hello";\nstring s2 = "hello";\nbool result = s1.equals(s2);  // true'
        }],
        ['isEmpty', {
            signature: 'isEmpty() -> bool',
            description: '检查字符串是否为空。',
            example: 'string s = "";\nbool result = s.isEmpty();  // true'
        }],
        ['compareTo', {
            signature: 'compareTo(str: string) -> int',
            description: '按字典序比较两个字符串。',
            example: 'string s1 = "apple";\nstring s2 = "banana";\nint result = s1.compareTo(s2);  // 负数'
        }],
        ['valueOf', {
            signature: 'valueOf(value: any) -> string',
            description: '将值转换为字符串表示（静态方法）。',
            example: 'int num = 42;\nstring s = String.valueOf(num);  // "42"'
        }],
        ['toString', {
            signature: 'toString() -> string',
            description: '将值转换为字符串表示。',
            example: 'int num = 42;\nstring s = num.toString();  // "42"'
        }]
    ]);

    // FFI 类型文档
    private ffiTypeDocs: Map<string, string> = new Map([
        ['c_int', '**c_int** - C 语言 int 类型'],
        ['c_long', '**c_long** - C 语言 long 类型'],
        ['c_short', '**c_short** - C 语言 short 类型'],
        ['c_char', '**c_char** - C 语言 char 类型'],
        ['c_byte', '**c_byte** - C 语言 signed char 类型'],
        ['c_float', '**c_float** - C 语言 float 类型'],
        ['c_double', '**c_double** - C 语言 double 类型'],
        ['c_bool', '**c_bool** - C 语言 _Bool 类型 (C99)'],
        ['c_void', '**c_void** - C 语言 void 类型'],
        ['size_t', '**size_t** - C 语言 size_t 类型'],
        ['ssize_t', '**ssize_t** - C 语言 ssize_t 类型'],
        ['uintptr_t', '**uintptr_t** - C 语言 uintptr_t 类型'],
        ['intptr_t', '**intptr_t** - C 语言 intptr_t 类型'],
        ['uint8_t', '**uint8_t** - 无符号 8 位整数'],
        ['uint16_t', '**uint16_t** - 无符号 16 位整数'],
        ['uint32_t', '**uint32_t** - 无符号 32 位整数'],
        ['uint64_t', '**uint64_t** - 无符号 64 位整数'],
        ['int8_t', '**int8_t** - 有符号 8 位整数'],
        ['int16_t', '**int16_t** - 有符号 16 位整数'],
        ['int32_t', '**int32_t** - 有符号 32 位整数'],
        ['int64_t', '**int64_t** - 有符号 64 位整数']
    ]);

    /**
     * 提供悬停信息
     */
    provideHover(
        document: vscode.TextDocument,
        position: vscode.Position,
        token: vscode.CancellationToken
    ): vscode.ProviderResult<vscode.Hover> {
        const wordRange = document.getWordRangeAtPosition(position);
        if (!wordRange) {
            return undefined;
        }

        const word = document.getText(wordRange);

        // 检查预处理器指令（需要检查行首）
        const lineText = document.lineAt(position).text;
        const trimmedLine = lineText.trim();
        if (trimmedLine.startsWith('#')) {
            const directiveMatch = trimmedLine.match(/^#(\w+)/);
            if (directiveMatch) {
                const directive = '#' + directiveMatch[1];
                if (this.preprocessorDocs.has(directive)) {
                    const doc = this.preprocessorDocs.get(directive);
                    if (doc) {
                        const content = new vscode.MarkdownString();
                        content.appendCodeblock(doc.signature, 'cavvy');
                        content.appendMarkdown(`\n${doc.description}\n\n**示例：**\n`);
                        content.appendCodeblock(doc.example, 'cavvy');
                        return new vscode.Hover(content);
                    }
                }
            }
        }

        // 检查关键字
        if (this.keywordDocs.has(word)) {
            const content = this.keywordDocs.get(word);
            if (content) {
                return new vscode.Hover(new vscode.MarkdownString(content));
            }
        }

        // 检查 FFI 类型
        if (this.ffiTypeDocs.has(word)) {
            const content = this.ffiTypeDocs.get(word);
            if (content) {
                return new vscode.Hover(new vscode.MarkdownString(content));
            }
        }

        // 检查内置方法
        if (this.methodDocs.has(word)) {
            const doc = this.methodDocs.get(word);
            if (doc) {
                const content = new vscode.MarkdownString();
                content.appendCodeblock(doc.signature, 'cavvy');
                content.appendMarkdown(`\n${doc.description}\n\n**示例：**\n`);
                content.appendCodeblock(doc.example, 'cavvy');
                return new vscode.Hover(content);
            }
        }

        return undefined;
    }
}
