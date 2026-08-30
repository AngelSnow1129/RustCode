package com.rustcode.jetbrains.services

import com.rustcode.jetbrains.settings.AtomCodeSettingsState
import com.rustcode.jetbrains.ui.openAtomCodeWelcomePage
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.project.Project
import com.intellij.openapi.startup.StartupActivity

class AtomCodeStartupActivity : StartupActivity.DumbAware {
    override fun runActivity(project: Project) {
        AtomCodeProjectService.getInstance(project).startBackgroundHealthChecks()
        showWelcomePageOnce(project)
    }

    private fun showWelcomePageOnce(project: Project) {
        val settings = AtomCodeSettingsState.getInstance()
        if (settings.state.welcomePageShown) return
        settings.update { it.welcomePageShown = true }
        ApplicationManager.getApplication().invokeLater {
            if (!project.isDisposed) {
                openAtomCodeWelcomePage(project)
            }
        }
    }
}
