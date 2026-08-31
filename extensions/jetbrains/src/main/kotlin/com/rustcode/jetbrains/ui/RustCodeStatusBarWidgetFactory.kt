package com.rustcode.jetbrains.ui

import com.rustcode.jetbrains.daemon.ConnectionState
import com.rustcode.jetbrains.services.RustCodeProjectService
import com.intellij.openapi.project.Project
import com.intellij.openapi.wm.StatusBar
import com.intellij.openapi.wm.StatusBarWidget
import com.intellij.openapi.wm.StatusBarWidgetFactory
import com.intellij.util.Consumer
import java.awt.event.MouseEvent
import java.beans.PropertyChangeListener
import javax.swing.SwingUtilities

class RustCodeStatusBarWidgetFactory : StatusBarWidgetFactory {
    override fun getId(): String = RustCodeStatusBarWidget.ID

    override fun getDisplayName(): String = "RustCode"

    override fun isAvailable(project: Project): Boolean = true

    override fun createWidget(project: Project): StatusBarWidget = RustCodeStatusBarWidget(project)

    override fun disposeWidget(widget: StatusBarWidget) {
        widget.dispose()
    }

    override fun canBeEnabledOn(statusBar: StatusBar): Boolean = true
}

private class RustCodeStatusBarWidget(private val project: Project) : StatusBarWidget, StatusBarWidget.TextPresentation {
    private val service = RustCodeProjectService.getInstance(project)
    private var statusBar: StatusBar? = null
    private val listener = PropertyChangeListener {
        SwingUtilities.invokeLater {
            statusBar?.updateWidget(ID)
        }
    }

    init {
        service.addConnectionListener(listener)
    }

    override fun ID(): String = ID

    override fun install(statusBar: StatusBar) {
        this.statusBar = statusBar
    }

    override fun dispose() {
        service.removeConnectionListener(listener)
        statusBar = null
    }

    override fun getPresentation(): StatusBarWidget.WidgetPresentation = this

    override fun getText(): String =
        when (service.connectionState) {
            is ConnectionState.Ready -> "RustCode"
            ConnectionState.Idle -> "RustCode ○"
            ConnectionState.CheckingDaemon,
            ConnectionState.StartingDaemon,
            ConnectionState.Connecting,
            ConnectionState.SyncingProject,
            ConnectionState.CheckingProvider -> "RustCode ..."
            is ConnectionState.SetupRequired,
            is ConnectionState.ProviderMissing,
            is ConnectionState.Error -> "RustCode !"
        }

    override fun getTooltipText(): String =
        when (val state = service.connectionState) {
            is ConnectionState.Ready -> "RustCode: Connected (${state.daemonVersion}). Click to open chat."
            ConnectionState.Idle -> "RustCode: Not connected. Click to open chat."
            ConnectionState.CheckingDaemon -> "RustCode: Checking daemon..."
            ConnectionState.StartingDaemon -> "RustCode: Starting daemon..."
            ConnectionState.Connecting -> "RustCode: Connecting..."
            ConnectionState.SyncingProject -> "RustCode: Syncing project..."
            ConnectionState.CheckingProvider -> "RustCode: Checking provider..."
            is ConnectionState.SetupRequired -> "RustCode: Setup required - ${state.reason}"
            is ConnectionState.ProviderMissing -> "RustCode: Provider missing"
            is ConnectionState.Error -> "RustCode: ${state.message}"
        }

    override fun getAlignment(): Float = 0.5f

    override fun getClickConsumer(): Consumer<MouseEvent>? =
        Consumer {
            openRustCodeChatTab(project)
        }

    companion object {
        const val ID = "RustCodeStatus"
    }
}
