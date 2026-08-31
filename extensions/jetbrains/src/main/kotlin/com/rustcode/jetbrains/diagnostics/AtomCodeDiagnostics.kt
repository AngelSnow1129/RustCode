package com.rustcode.jetbrains.diagnostics

import com.rustcode.jetbrains.security.SecretRedactor
import com.intellij.openapi.application.ApplicationInfo
import com.intellij.openapi.project.Project

object AtomCodeDiagnostics {
    fun summary(project: Project, rawDetails: String = ""): String {
        val text = buildString {
            appendLine("RustCode JetBrains diagnostics")
            appendLine("IDE: ${ApplicationInfo.getInstance().fullVersion}")
            appendLine("Project: ${project.name}")
            if (rawDetails.isNotBlank()) {
                appendLine("Details:")
                appendLine(rawDetails)
            }
        }
        return SecretRedactor.redact(text)
    }
}

