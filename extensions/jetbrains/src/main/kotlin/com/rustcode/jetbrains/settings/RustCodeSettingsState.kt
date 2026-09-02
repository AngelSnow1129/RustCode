package com.rustcode.jetbrains.settings

import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.components.PersistentStateComponent
import com.intellij.openapi.components.Service
import com.intellij.openapi.components.State
import com.intellij.openapi.components.Storage

enum class RustCodeContextLevel {
    Minimal,
    CurrentFile,
    ProjectContext,
    ;

    // Combo-box label; persistence still uses the enum constant name.
    override fun toString(): String = when (this) {
        Minimal -> com.rustcode.jetbrains.i18n.RustCodeBundle.message("context.level.minimal")
        CurrentFile -> com.rustcode.jetbrains.i18n.RustCodeBundle.message("context.level.currentFile")
        ProjectContext -> com.rustcode.jetbrains.i18n.RustCodeBundle.message("context.level.project")
    }
}

data class RustCodeSettings(
    var daemonBinaryPath: String = "",
    var host: String = "127.0.0.1",
    var port: Int = 13456,
    var autoStart: Boolean = true,
    var requestTimeoutMs: Int = 30_000,
    var autoSaveBeforeRead: Boolean = true,
    var contextLevel: RustCodeContextLevel = RustCodeContextLevel.Minimal,
    var allowSelectedTextContext: Boolean = true,
    var sendRelativePathWithSelection: Boolean = true,
    var sendWithCtrlEnter: Boolean = false,
    var chatFontSize: Int = 13,
    var welcomePageShown: Boolean = false,
)

@Service(Service.Level.APP)
@State(name = "RustCodeSettings", storages = [Storage("rustcode.xml")])
class RustCodeSettingsState : PersistentStateComponent<RustCodeSettings> {
    private var state = RustCodeSettings()

    override fun getState(): RustCodeSettings = state

    override fun loadState(state: RustCodeSettings) {
        this.state = state.normalized()
    }

    fun update(block: (RustCodeSettings) -> Unit) {
        val next = state.copy()
        block(next)
        state = next.normalized()
    }

    companion object {
        fun getInstance(): RustCodeSettingsState =
            ApplicationManager.getApplication().getService(RustCodeSettingsState::class.java)
    }
}

internal fun RustCodeSettings.normalized(): RustCodeSettings {
    if (host.isBlank()) host = "127.0.0.1"
    if (port <= 0) port = 13456
    if (requestTimeoutMs <= 0) requestTimeoutMs = 30_000
    if (chatFontSize <= 0) chatFontSize = 13
    return this
}
