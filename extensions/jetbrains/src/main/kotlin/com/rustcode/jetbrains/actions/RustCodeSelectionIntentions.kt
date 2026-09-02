package com.rustcode.jetbrains.actions

import com.rustcode.jetbrains.i18n.RustCodeBundle

import com.intellij.codeInsight.intention.IntentionAction
import com.intellij.openapi.editor.Editor
import com.intellij.openapi.project.Project
import com.intellij.psi.PsiFile

abstract class RustCodeSelectionIntention(
    private val title: String,
    private val instruction: String?,
) : IntentionAction {
    override fun getText(): String = title

    override fun getFamilyName(): String = "RustCode"

    override fun isAvailable(project: Project, editor: Editor, file: PsiFile): Boolean =
        if (instruction == null) editor.selectionModel.hasSelection() || editor.document.text.isNotBlank()
        else EditorRustCodeActions.canSendSelectedText(editor)

    override fun invoke(project: Project, editor: Editor, file: PsiFile) {
        if (instruction == null) {
            EditorRustCodeActions.addEditorContext(project, editor)
        } else {
            EditorRustCodeActions.sendSelectionCommand(project, editor, instruction)
        }
    }

    override fun startInWriteAction(): Boolean = false
}

class ExplainSelectionIntention : RustCodeSelectionIntention(
    RustCodeBundle.message("intention.explain.title"),
    RustCodeBundle.message("intention.explain.prompt"),
)

class FixSelectionIntention : RustCodeSelectionIntention(
    RustCodeBundle.message("intention.fix.title"),
    RustCodeBundle.message("intention.fix.prompt"),
)

class OptimizeSelectionIntention : RustCodeSelectionIntention(
    RustCodeBundle.message("intention.optimize.title"),
    RustCodeBundle.message("intention.optimize.prompt"),
)

class AddContextIntention : RustCodeSelectionIntention(
    RustCodeBundle.message("intention.addContext.title"),
    null,
)
