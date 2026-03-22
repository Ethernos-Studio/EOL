import * as vscode from 'vscode';
import { exec } from 'child_process';
import { promisify } from 'util';
import * as path from 'path';
import * as fs from 'fs';

const execAsync = promisify(exec);

/**
 * 运行选项
 */
export interface RunOptions {
    debug?: boolean;
    verbose?: boolean;
    outputFile?: string;
    noRun?: boolean;
}

/**
 * Cavvy 代码运行器
 * 支持使用 cay-run 运行 Cavvy 代码
 */
export class CavvyRunner {
    private config: vscode.WorkspaceConfiguration;
    private outputChannel: vscode.OutputChannel;
    private terminal: vscode.Terminal | undefined;

    constructor() {
        this.config = vscode.workspace.getConfiguration('cavvyAnalyzer');
        this.outputChannel = vscode.window.createOutputChannel('Cavvy Runner');
    }

    /**
     * 运行 Cavvy 文件
     * @param filePath 文件路径
     * @param options 运行选项
     */
    async run(filePath: string, options: RunOptions = {}): Promise<void> {
        const runnerPath = this.config.get<string>('runnerPath', 'cay-run');
        const runInTerminal = this.config.get<boolean>('runInTerminal', true);
        const preserveFocus = this.config.get<boolean>('preserveFocus', false);

        // 检查文件是否存在
        if (!fs.existsSync(filePath)) {
            vscode.window.showErrorMessage(`文件不存在: ${filePath}`);
            return;
        }

        // 检查文件类型
        const ext = path.extname(filePath).toLowerCase();
        const supportedExts = ['.cay', '.eol', '.caybc', '.ll'];
        if (!supportedExts.includes(ext)) {
            vscode.window.showErrorMessage(`不支持的文件类型: ${ext}。支持的类型: ${supportedExts.join(', ')}`);
            return;
        }

        try {
            // 构建命令参数
            const args: string[] = [];
            if (options.verbose) {
                args.push('--verbose');
            }
            if (options.noRun) {
                args.push('--no-run');
            }
            if (options.outputFile) {
                args.push('-o', options.outputFile);
            }
            args.push(`"${filePath}"`);

            const command = `"${runnerPath}" ${args.join(' ')}`;

            if (runInTerminal) {
                // 在终端中运行（支持交互式输入）
                await this.runInTerminal(command, filePath, preserveFocus);
            } else {
                // 在输出通道中运行
                await this.runInOutputChannel(command, filePath);
            }
        } catch (error) {
            vscode.window.showErrorMessage(`运行失败: ${error}`);
        }
    }

    /**
     * 在终端中运行
     * @param command 命令
     * @param filePath 文件路径
     * @param preserveFocus 是否保持焦点
     */
    private async runInTerminal(command: string, filePath: string, preserveFocus: boolean): Promise<void> {
        const fileName = path.basename(filePath);

        // 如果终端已存在且未关闭，则复用
        if (this.terminal) {
            try {
                this.terminal.show(preserveFocus);
                this.terminal.sendText(command);
                return;
            } catch {
                // 终端可能已关闭，创建新终端
                this.terminal = undefined;
            }
        }

        // 创建新终端
        this.terminal = vscode.window.createTerminal({
            name: `Cavvy: ${fileName}`,
            cwd: path.dirname(filePath)
        });

        this.terminal.show(preserveFocus);
        this.terminal.sendText(command);

        // 监听终端关闭事件
        const dispose = vscode.window.onDidCloseTerminal((t) => {
            if (t === this.terminal) {
                this.terminal = undefined;
                dispose.dispose();
            }
        });
    }

    /**
     * 在输出通道中运行
     * @param command 命令
     * @param filePath 文件路径
     */
    private async runInOutputChannel(command: string, filePath: string): Promise<void> {
        const fileName = path.basename(filePath);

        this.outputChannel.clear();
        this.outputChannel.show(true);
        this.outputChannel.appendLine(`运行: ${fileName}`);
        this.outputChannel.appendLine(`命令: ${command}`);
        this.outputChannel.appendLine('─'.repeat(50));

        try {
            const { stdout, stderr } = await execAsync(command, {
                timeout: 60000,
                cwd: path.dirname(filePath)
            });

            if (stdout) {
                this.outputChannel.appendLine(stdout);
            }
            if (stderr) {
                this.outputChannel.appendLine('标准错误输出:');
                this.outputChannel.appendLine(stderr);
            }

            this.outputChannel.appendLine('─'.repeat(50));
            this.outputChannel.appendLine('程序执行完成');
        } catch (error: any) {
            this.outputChannel.appendLine('执行出错:');
            this.outputChannel.appendLine(error.message || String(error));

            if (error.stdout) {
                this.outputChannel.appendLine('标准输出:');
                this.outputChannel.appendLine(error.stdout);
            }
            if (error.stderr) {
                this.outputChannel.appendLine('标准错误:');
                this.outputChannel.appendLine(error.stderr);
            }

            vscode.window.showErrorMessage(`运行失败: ${error.message || String(error)}`);
        }
    }

    /**
     * 编译代码（不运行）
     * @param filePath 文件路径
     * @param outputFile 输出文件路径
     */
    async compile(filePath: string, outputFile?: string): Promise<boolean> {
        const runnerPath = this.config.get<string>('runnerPath', 'cay-run');

        try {
            const args: string[] = ['--no-run'];
            if (outputFile) {
                args.push('-o', outputFile);
            }
            args.push(`"${filePath}"`);

            const command = `"${runnerPath}" ${args.join(' ')}`;

            const { stdout, stderr } = await execAsync(command, {
                timeout: 60000,
                cwd: path.dirname(filePath)
            });

            if (stderr) {
                vscode.window.showWarningMessage(`编译警告: ${stderr}`);
            }

            vscode.window.showInformationMessage('编译成功');
            return true;
        } catch (error: any) {
            vscode.window.showErrorMessage(`编译失败: ${error.message || String(error)}`);
            return false;
        }
    }

    /**
     * 检查 cay-run 是否可用
     */
    async checkRunner(): Promise<boolean> {
        const runnerPath = this.config.get<string>('runnerPath', 'cay-run');

        try {
            await execAsync(`"${runnerPath}" --version`, { timeout: 5000 });
            return true;
        } catch {
            return false;
        }
    }

    /**
     * 配置变更时的处理
     */
    onConfigurationChanged(): void {
        this.config = vscode.workspace.getConfiguration('cavvyAnalyzer');
    }

    /**
     * 释放资源
     */
    dispose(): void {
        if (this.terminal) {
            this.terminal.dispose();
            this.terminal = undefined;
        }
        this.outputChannel.dispose();
    }
}
