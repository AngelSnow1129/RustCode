package com.rustcode.jetbrains.actions

import com.rustcode.jetbrains.i18n.RustCodeBundle

class ExplainSelectionAction : EditorSelectionCommandAction(
    RustCodeBundle.message("intention.explain.prompt"),
)
