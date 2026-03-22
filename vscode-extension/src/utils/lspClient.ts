import * as vscode from 'vscode';
import { LanguageClient, LanguageClientOptions, ServerOptions, TransportKind } from 'vscode-languageclient/node';

/**
 * Cavvy LSP 客户端
 * 用于与 cay-lsp 语言服务器通信
 */
export class CavvyLSPClient {
    private client: LanguageClient | undefined;
    private config: vscode.WorkspaceConfiguration;
    private isActive: boolean = false;

    constructor() {
        this.config = vscode.workspace.getConfiguration('cavvyAnalyzer');
    }

    /**
     * 激活 LSP 客户端
     * @param context 插件上下文
     */
    async activate(context: vscode.ExtensionContext): Promise<void> {
        const lspServerPath = this.config.get<string>('lspServerPath', 'cay-lsp');
        const enableLSP = this.config.get<boolean>('enableLSP', true);

        if (!enableLSP) {
            console.log('LSP 被禁用');
            return;
        }

        // 检查 cay-lsp 是否可用
        const isAvailable = await this.checkLspServer(lspServerPath);
        if (!isAvailable) {
            console.log(`cay-lsp 不可用: ${lspServerPath}`);
            vscode.window.showWarningMessage(
                `Cavvy LSP 服务器 (${lspServerPath}) 不可用。某些功能可能受限。`,
                '禁用 LSP', '查看设置'
            ).then(selection => {
                if (selection === '禁用 LSP') {
                    this.config.update('enableLSP', false, true);
                } else if (selection === '查看设置') {
                    vscode.commands.executeCommand('workbench.action.openSettings', 'cavvyAnalyzer.lspServerPath');
                }
            });
            return;
        }

        try {
            // 配置服务器选项
            const serverOptions: ServerOptions = {
                command: lspServerPath,
                args: [],
                transport: TransportKind.stdio
            };

            // 配置客户端选项
            const clientOptions: LanguageClientOptions = {
                documentSelector: [
                    { scheme: 'file', language: 'cavvy' },
                    { scheme: 'file', pattern: '**/*.cay' },
                    { scheme: 'file', pattern: '**/*.eol' },
                    { scheme: 'file', pattern: '**/*.caybc' },
                    { scheme: 'file', pattern: '**/*.ll' }
                ],
                synchronize: {
                    fileEvents: vscode.workspace.createFileSystemWatcher('**/*.cay')
                },
                outputChannelName: 'Cavvy LSP',
                revealOutputChannelOn: 4 // never
            };

            // 创建语言客户端
            this.client = new LanguageClient(
                'cavvyLSP',
                'Cavvy Language Server',
                serverOptions,
                clientOptions
            );

            // 启动客户端
            await this.client.start();
            this.isActive = true;

            console.log('Cavvy LSP 客户端已启动');

            // 注册到上下文
            context.subscriptions.push(this.client);

        } catch (error) {
            console.error('启动 LSP 客户端失败:', error);
            vscode.window.showErrorMessage(`启动 Cavvy LSP 失败: ${error}`);
            this.isActive = false;
        }
    }

    /**
     * 检查 LSP 服务器是否可用
     * @param serverPath 服务器路径
     */
    private async checkLspServer(serverPath: string): Promise<boolean> {
        const { exec } = require('child_process');
        const { promisify } = require('util');
        const execAsync = promisify(exec);

        try {
            // 尝试运行 cay-lsp --version
            await execAsync(`"${serverPath}" --version`, { timeout: 5000 });
            return true;
        } catch {
            return false;
        }
    }

    /**
     * 触发文档诊断
     * @param document 文档
     */
    async triggerDiagnostics(document: vscode.TextDocument): Promise<void> {
        if (!this.client || !this.isActive) {
            return;
        }

        try {
            // 发送文档内容变更通知以触发诊断
            await this.client.sendNotification('textDocument/didChange', {
                textDocument: {
                    uri: document.uri.toString(),
                    version: document.version
                },
                contentChanges: [
                    {
                        text: document.getText()
                    }
                ]
            });
        } catch (error) {
            console.error('触发诊断失败:', error);
        }
    }

    /**
     * 重启 LSP 服务器
     */
    async restart(context?: vscode.ExtensionContext): Promise<void> {
        if (this.client) {
            await this.client.stop();
            this.isActive = false;
        }

        // 重新创建配置
        this.config = vscode.workspace.getConfiguration('cavvyAnalyzer');

        // 重新激活
        if (context) {
            await this.activate(context);
        }
    }

    /**
     * 停止 LSP 服务器
     */
    async stop(): Promise<void> {
        if (this.client) {
            await this.client.stop();
            this.isActive = false;
            console.log('Cavvy LSP 客户端已停止');
        }
    }

    /**
     * 检查 LSP 是否正在运行
     */
    isRunning(): boolean {
        return this.isActive && this.client !== undefined;
    }

    /**
     * 获取语言客户端
     */
    getClient(): LanguageClient | undefined {
        return this.client;
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
        if (this.client) {
            this.client.stop();
            this.client = undefined;
        }
        this.isActive = false;
    }
}
