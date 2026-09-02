package com.rustcode.jetbrains.actions

import com.rustcode.jetbrains.i18n.RustCodeBundle

class FixSelectionAction : EditorSelectionCommandAction(
    RustCodeBundle.message("intention.fix.prompt"),
)
