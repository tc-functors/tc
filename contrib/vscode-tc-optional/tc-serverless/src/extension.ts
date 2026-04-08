import * as vscode from "vscode";

const CONFIG = "tc.serverless";

interface TcTaskDefinition extends vscode.TaskDefinition {
  type: "tc";
  command: string;
  args?: string[];
  cwd?: string;
}

function getExecutable(): string {
  return vscode.workspace.getConfiguration(CONFIG).get<string>("executablePath", "tc");
}

function getTopologyDir(): string {
  return vscode.workspace.getConfiguration(CONFIG).get<string>("topologyDirectory", "").trim();
}

function getDefaultProfile(): string {
  return vscode.workspace.getConfiguration(CONFIG).get<string>("defaultProfile", "").trim();
}

function getDefaultSandbox(): string {
  return vscode.workspace.getConfiguration(CONFIG).get<string>("defaultSandbox", "").trim();
}

function dirFlagArgs(): string[] {
  const d = getTopologyDir();
  if (!d) return [];
  return ["-d", d];
}

function profileArgs(): string[] {
  const e = getDefaultProfile();
  if (!e) return [];
  return ["-e", e];
}

function sandboxArgs(): string[] {
  const s = getDefaultSandbox();
  if (!s) return [];
  return ["-s", s];
}

function workspaceRoot(): string | undefined {
  return vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
}

function quoteIfNeeded(segment: string): string {
  return /\s/.test(segment) ? `"${segment.replace(/"/g, '\\"')}"` : segment;
}

function runInTerminal(parts: string[]): void {
  const cwd = workspaceRoot();
  if (!cwd) {
    void vscode.window.showErrorMessage("tc: open a folder in the workspace first.");
    return;
  }
  const exe = getExecutable();
  const line = [quoteIfNeeded(exe), ...parts.map(quoteIfNeeded)].join(" ");
  const terminal = vscode.window.createTerminal({ name: "tc", cwd });
  terminal.sendText(line, true);
  terminal.show();
}

async function inputEntity(prompt: string): Promise<string | undefined> {
  return vscode.window.showInputBox({
    prompt,
    ignoreFocusOut: true,
  });
}

function register(
  context: vscode.ExtensionContext,
  command: string,
  fn: () => void | Promise<void>
): void {
  context.subscriptions.push(vscode.commands.registerCommand(command, fn));
}

export function activate(context: vscode.ExtensionContext): void {
  register(context, "tc.serverless.compose", () => {
    runInTerminal(["compose", ...dirFlagArgs()]);
  });

  register(context, "tc.serverless.compile", () => {
    runInTerminal(["compile", ...dirFlagArgs()]);
  });

  register(context, "tc.serverless.build", async () => {
    const name = await inputEntity("Entity name (-n), e.g. function or layer name");
    if (name === undefined) return;
    const kind = await vscode.window.showInputBox({
      prompt: "Kind (-k), optional (e.g. function, layer)",
      ignoreFocusOut: true,
    });
    const parts = ["build", ...profileArgs(), ...dirFlagAsBuildDir()];
    if (name) {
      parts.push("-n", name);
    }
    if (kind?.trim()) {
      parts.push("-k", kind.trim());
    }
    runInTerminal(parts);
  });

  register(context, "tc.serverless.deploy", () => {
    const parts = ["deploy", ...deployEnvSandbox(), ...deployDir()];
    runInTerminal(parts);
  });

  register(context, "tc.serverless.resolve", () => {
    runInTerminal(["resolve", ...profileArgs(), ...sandboxArgs()]);
  });

  register(context, "tc.serverless.emulate", async () => {
    const entity = await inputEntity("Entity (-c), optional");
    if (entity === undefined) return;
    const kind = await vscode.window.showInputBox({
      prompt: "Kind (-k), optional",
      ignoreFocusOut: true,
    });
    const parts = ["emulate", ...profileArgs(), ...sandboxArgs()];
    if (entity) {
      parts.push("-c", entity);
    }
    if (kind?.trim()) {
      parts.push("-k", kind.trim());
    }
    runInTerminal(parts);
  });

  register(context, "tc.serverless.test", () => {
    runInTerminal(["test", ...profileArgs(), ...sandboxArgs(), ...dirFlagArgs()]);
  });

  register(context, "tc.serverless.invoke", async () => {
    const entity = await inputEntity("Entity (-c), optional");
    if (entity === undefined) return;
    const parts = ["invoke", ...profileArgs(), ...sandboxArgs(), ...invokeDir()];
    if (entity) {
      parts.push("-c", entity);
    }
    runInTerminal(parts);
  });

  register(context, "tc.serverless.version", () => {
    runInTerminal(["version"]);
  });

  register(context, "tc.serverless.visualize", () => {
    runInTerminal(["visualize", ...dirFlagArgs()]);
  });

  context.subscriptions.push(
    vscode.tasks.registerTaskProvider("tc", {
      provideTasks: () => {
        const folder = vscode.workspace.workspaceFolders?.[0];
        if (!folder) {
          return [];
        }
        const mk = (command: string, label: string, extra: string[] = []) =>
          createTcTask(
            { type: "tc", command, args: extra },
            label,
            folder,
            vscode.TaskScope.Workspace
          );
        return [
          mk("compose", "tc: compose"),
          mk("compile", "tc: compile"),
          mk("version", "tc: version"),
        ];
      },
      resolveTask(task: vscode.Task): vscode.Task | undefined {
        const def = task.definition as TcTaskDefinition;
        if (def.type !== "tc" || !def.command) {
          return undefined;
        }
        const folder = vscode.workspace.workspaceFolders?.[0];
        const scope = task.scope === undefined ? vscode.TaskScope.Workspace : task.scope;
        return createTcTask(def, task.name, folder, scope);
      },
    })
  );
}

/** build uses -n for name; topology path is still -d when topologyDirectory is set */
function dirFlagAsBuildDir(): string[] {
  return dirFlagArgs();
}

/** deploy uses -e for env and -d for dir */
function deployEnvSandbox(): string[] {
  const parts: string[] = [];
  const env = getDefaultProfile();
  if (env) {
    parts.push("-e", env);
  }
  parts.push(...sandboxArgs());
  return parts;
}

function deployDir(): string[] {
  return dirFlagArgs();
}

function invokeDir(): string[] {
  const d = getTopologyDir();
  if (!d) return [];
  return ["-d", d];
}

function createTcTask(
  def: TcTaskDefinition,
  name: string,
  folder: vscode.WorkspaceFolder | undefined,
  scope: vscode.TaskScope.Workspace | vscode.TaskScope.Global | vscode.WorkspaceFolder
): vscode.Task {
  const exe = getExecutable();
  const extra = def.args ?? [];
  const args = [def.command, ...extra];
  const cwdOptions = resolveTaskCwd(def.cwd, folder);

  const execution = new vscode.ShellExecution(exe, args, cwdOptions);

  const task = new vscode.Task(
    def,
    scope,
    name,
    "tc",
    execution,
    []
  );
  task.presentationOptions = {
    reveal: vscode.TaskRevealKind.Always,
    panel: vscode.TaskPanelKind.Dedicated,
    showReuseMessage: false,
    clear: false,
  };
  return task;
}

function resolveTaskCwd(
  cwd: string | undefined,
  folder: vscode.WorkspaceFolder | undefined
): vscode.ShellExecutionOptions {
  if (!cwd?.trim()) {
    return folder ? { cwd: folder.uri.fsPath } : {};
  }
  const c = cwd.trim();
  if (vscode.Uri.file(c).scheme === "file" && (c.startsWith("/") || /^[a-zA-Z]:\\/.test(c))) {
    return { cwd: c };
  }
  if (folder) {
    return { cwd: vscode.Uri.joinPath(folder.uri, c).fsPath };
  }
  return { cwd };
}

export function deactivate(): void {}
