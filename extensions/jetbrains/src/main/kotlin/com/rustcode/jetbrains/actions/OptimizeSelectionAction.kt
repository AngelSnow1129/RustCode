package com.rustcode.jetbrains.actions

import com.rustcode.jetbrains.i18n.RustCodeBundle

class OptimizeSelectionAction : EditorSelectionCommandAction(
    RustCodeBundle.message("intention.optimize.prompt"),
)
