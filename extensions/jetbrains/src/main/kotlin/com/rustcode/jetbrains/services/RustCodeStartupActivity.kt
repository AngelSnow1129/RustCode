package com.rustcode.jetbrains.services

import com.rustcode.jetbrains.settings.RustCodeSettingsState
import com.rustcode.jetbrains.ui.openRustCodeWelcomePage
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.project.Project
import com.intellij.openapi.startup.StartupActivity

class RustCodeStartupActivity : StartupActivity.DumbAware {
    override fun runActivity(project: Project) {
        RustCodeProjectService.getInstance(project).startBackgroundHealthChecks()
        showWelcomePageOnce(project)
    }

    private fun showWelcomePageOnce(project: Project) {
        val settings = RustCodeSettingsState.getInstance()
        if (settings.state.welcomePageShown) return
        settings.update { it.welcomePageShown = true }
        ApplicationManager.getApplication().invokeLater {
            if (!project.isDisposed) {
                openRustCodeWelcomePage(project)
            }
        }
    }
}
