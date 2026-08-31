import * as vscode from 'vscode';

export class RustCodeActionProvider implements vscode.CodeActionProvider {
  static readonly providedCodeActionKinds = [vscode.CodeActionKind.QuickFix, vscode.CodeActionKind.Refactor];

  provideCodeActions(
    _document: vscode.TextDocument,
    range: vscode.Range | vscode.Selection,
  ): vscode.CodeAction[] {
    if (range.isEmpty) return [];

    const actions: vscode.CodeAction[] = [];

    const explainAction = new vscode.CodeAction(vscode.l10n.t('RustCode: Explain'), vscode.CodeActionKind.Empty);
    explainAction.command = { command: 'rustcode.explain', title: vscode.l10n.t('Explain Selection') };
    actions.push(explainAction);

    const fixAction = new vscode.CodeAction(vscode.l10n.t('RustCode: Fix'), vscode.CodeActionKind.QuickFix);
    fixAction.command = { command: 'rustcode.fix', title: vscode.l10n.t('Fix Selection') };
    actions.push(fixAction);

    const optimizeAction = new vscode.CodeAction(vscode.l10n.t('RustCode: Optimize'), vscode.CodeActionKind.Refactor);
    optimizeAction.command = { command: 'rustcode.optimize', title: vscode.l10n.t('Optimize Selection') };
    actions.push(optimizeAction);

    const addToChatAction = new vscode.CodeAction(vscode.l10n.t('RustCode: Add to Chat'), vscode.CodeActionKind.Empty);
    addToChatAction.command = { command: 'rustcode.addToChat', title: vscode.l10n.t('Add to Chat') };
    actions.push(addToChatAction);

    return actions;
  }
}
