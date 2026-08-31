package com.rustcode.jetbrains.ui

import com.rustcode.jetbrains.persistence.WorkspaceTabState
import com.rustcode.jetbrains.session.ChatRuntime
import com.rustcode.jetbrains.session.SessionWorkspace
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.project.Project
import com.intellij.openapi.util.Key
import com.intellij.openapi.wm.ToolWindow
import com.intellij.openapi.wm.ToolWindowManager
import com.intellij.ui.content.Content
import com.intellij.ui.content.ContentFactory
import com.intellij.ui.content.ContentManagerEvent
import com.intellij.ui.content.ContentManagerListener
import java.awt.Dimension
import java.awt.event.MouseAdapter
import java.awt.event.MouseEvent
import javax.swing.JMenuItem
import javax.swing.JPopupMenu
import javax.swing.JSeparator
import javax.swing.JTabbedPane
import javax.swing.SwingUtilities

const val RUSTCODE_TOOL_WINDOW_ID = "RustCode"
private val RUSTCODE_TAB_ID_KEY = Key.create<String>("rustcode.tabId")
private val RUSTCODE_TOOL_WINDOW_MIN_SIZE = Dimension(360, 300)

fun createRustCodeChatContent(project: Project, toolWindow: ToolWindow, closeable: Boolean): RustCodeChatPanel {
    val name = nextChatTabName(toolWindow)
    val runtime = SessionWorkspace.getInstance(project).createRuntime(name)
    return createRustCodeChatContent(project, toolWindow, closeable, runtime, name)
}

fun restoreRustCodeChatContent(project: Project, toolWindow: ToolWindow, tab: WorkspaceTabState): RustCodeChatPanel {
    val name = tab.title.ifBlank { nextChatTabName(toolWindow) }
    val runtime = SessionWorkspace.getInstance(project).createRuntimeForRestoredTab(tab)
    return createRustCodeChatContent(project, toolWindow, closeable = true, runtime, name)
}

private fun createRustCodeChatContent(
    project: Project,
    toolWindow: ToolWindow,
    closeable: Boolean,
    runtime: ChatRuntime,
    name: String,
): RustCodeChatPanel {
    val panel = RustCodeChatPanel(project, runtime)
    val content = ContentFactory.getInstance().createContent(panel, name, false).apply {
        isCloseable = closeable
        description = "RustCode Chat"
        putUserData(RUSTCODE_TAB_ID_KEY, runtime.tabId)
        setDisposer(panel)
    }
    toolWindow.contentManager.addContent(content)
    toolWindow.contentManager.setSelectedContent(content)
    toolWindow.component.minimumSize = RUSTCODE_TOOL_WINDOW_MIN_SIZE

    // 给标签栏安装右键菜单
    installTabPopupMenu(toolWindow, project)
    installContentSelectionListener(toolWindow, project)

    return panel
}

fun selectedRustCodeChatPanel(project: Project): RustCodeChatPanel? {
    val toolWindow = ToolWindowManager.getInstance(project).getToolWindow(RUSTCODE_TOOL_WINDOW_ID) ?: return null
    val selected = toolWindow.contentManager.selectedContent?.component as? RustCodeChatPanel
    if (selected != null) return selected
    return toolWindow.contentManager.contents
        .asSequence()
        .mapNotNull { it.component as? RustCodeChatPanel }
        .firstOrNull()
}

fun ensureRustCodeChatContent(project: Project, toolWindow: ToolWindow): RustCodeChatPanel {
    val selected = toolWindow.contentManager.selectedContent?.component as? RustCodeChatPanel
    if (selected != null) return selected
    val existing = toolWindow.contentManager.contents
        .asSequence()
        .mapNotNull { it.component as? RustCodeChatPanel }
        .firstOrNull()
    if (existing != null) return existing
    return createRustCodeChatContent(project, toolWindow, closeable = true)
}

fun openRustCodeChatTab(project: Project, newTab: Boolean = false, focusInput: Boolean = true) {
    ApplicationManager.getApplication().invokeLater {
        val toolWindow = ToolWindowManager.getInstance(project).getToolWindow(RUSTCODE_TOOL_WINDOW_ID) ?: return@invokeLater
        toolWindow.show()
        val panel = if (newTab) {
            createRustCodeChatContent(project, toolWindow, closeable = true)
        } else {
            ensureRustCodeChatContent(project, toolWindow)
        }
        if (focusInput) panel.focusInput()
    }
}

fun openRustCodeWelcomePage(project: Project) {
    ApplicationManager.getApplication().invokeLater {
        val toolWindow = ToolWindowManager.getInstance(project).getToolWindow(RUSTCODE_TOOL_WINDOW_ID) ?: return@invokeLater
        toolWindow.show()
        ensureRustCodeChatContent(project, toolWindow).showWelcomePage()
    }
}

fun openRustCodeSessionHistory(project: Project) {
    ApplicationManager.getApplication().invokeLater {
        val toolWindow = ToolWindowManager.getInstance(project).getToolWindow(RUSTCODE_TOOL_WINDOW_ID) ?: return@invokeLater
        toolWindow.show()
        ensureRustCodeChatContent(project, toolWindow).showSessionHistory()
    }
}

fun closeCurrentChatTab(project: Project) {
    ApplicationManager.getApplication().invokeLater {
        val toolWindow = ToolWindowManager.getInstance(project).getToolWindow(RUSTCODE_TOOL_WINDOW_ID) ?: return@invokeLater
        val contentManager = toolWindow.contentManager
        val selected = contentManager.selectedContent ?: return@invokeLater

        if (contentManager.contentCount <= 1) {
            // 最后一个标签页：清空内容
            val panel = selected.component as? RustCodeChatPanel
            panel?.startNewConversation()
            return@invokeLater
        }
        closeRuntimeForContent(project, selected)
        contentManager.removeContent(selected, true)
    }
}

fun contentTabId(content: Content): String? =
    content.getUserData(RUSTCODE_TAB_ID_KEY)

fun updateRustCodeChatTabTitle(project: Project, panel: RustCodeChatPanel, title: String) {
    val normalizedTitle = title.trim()
    if (normalizedTitle.isEmpty()) return
    val toolWindow = ToolWindowManager.getInstance(project).getToolWindow(RUSTCODE_TOOL_WINDOW_ID) ?: return
    val content = toolWindow.contentManager.getContent(panel) ?: return
    content.displayName = normalizedTitle
    content.description = normalizedTitle
}

private fun nextChatTabName(toolWindow: ToolWindow): String {
    val count = toolWindow.contentManager.contentCount
    return if (count == 0) "Chat" else "Chat ${count + 1}"
}

/**
 * 在标签栏上安装右键弹出菜单，支持关闭/关闭其他/新建。
 */
private fun installTabPopupMenu(toolWindow: ToolWindow, project: Project) {
    SwingUtilities.invokeLater {
        val tabPane = findTabbedPane(toolWindow) ?: return@invokeLater

        // 避免重复安装
        if (tabPane.getClientProperty("rustcode-popup-installed") == true) return@invokeLater
        tabPane.putClientProperty("rustcode-popup-installed", true)

        tabPane.addMouseListener(object : MouseAdapter() {
            override fun mousePressed(e: MouseEvent) = maybeShowPopup(e)
            override fun mouseReleased(e: MouseEvent) = maybeShowPopup(e)

            private fun maybeShowPopup(e: MouseEvent) {
                if (!e.isPopupTrigger) return
                val index = tabPane.indexAtLocation(e.x, e.y)
                if (index < 0) return

                val contentManager = toolWindow.contentManager
                val clickedContent = contentManager.getContent(index) ?: return

                val menu = JPopupMenu()
                menu.add(JMenuItem("关闭标签页").apply {
                    addActionListener {
                        if (contentManager.contentCount <= 1) {
                            val panel = clickedContent.component as? RustCodeChatPanel
                            panel?.startNewConversation()
                        } else {
                            closeRuntimeForContent(project, clickedContent)
                            contentManager.removeContent(clickedContent, true)
                        }
                    }
                })
                menu.add(JMenuItem("关闭其他标签页").apply {
                    isEnabled = contentManager.contentCount > 1
                    addActionListener {
                        val others = contentManager.contents.filter { it != clickedContent }
                        others.forEach {
                            closeRuntimeForContent(project, it)
                            contentManager.removeContent(it, true)
                        }
                    }
                })
                menu.add(JSeparator())
                menu.add(JMenuItem("新建标签页").apply {
                    addActionListener { openRustCodeChatTab(project, newTab = true) }
                })
                menu.show(tabPane, e.x, e.y)
            }
        })
    }
}

private fun closeRuntimeForContent(project: Project, content: Content) {
    contentTabId(content)?.let { SessionWorkspace.getInstance(project).close(it) }
}

private fun installContentSelectionListener(toolWindow: ToolWindow, project: Project) {
    if (toolWindow.component.getClientProperty("rustcode-content-listener-installed") == true) return
    toolWindow.component.putClientProperty("rustcode-content-listener-installed", true)
    toolWindow.contentManager.addContentManagerListener(object : ContentManagerListener {
        override fun selectionChanged(event: ContentManagerEvent) {
            contentTabId(event.content)?.let { SessionWorkspace.getInstance(project).select(it) }
        }

        override fun contentRemoved(event: ContentManagerEvent) {
            closeRuntimeForContent(project, event.content)
        }
    })
}

/**
 * 在 ToolWindow 组件树中查找 JTabbedPane（标签栏）。
 */
private fun findTabbedPane(toolWindow: ToolWindow): JTabbedPane? =
    findComponentOfType(toolWindow.component, JTabbedPane::class.java)

private fun <T> findComponentOfType(root: java.awt.Component, type: Class<T>, maxDepth: Int = 50): T? {
    if (maxDepth <= 0) return null
    if (type.isInstance(root)) {
        @Suppress("UNCHECKED_CAST")
        return root as T
    }
    if (root is java.awt.Container) {
        for (child in root.components) {
            val found = findComponentOfType(child, type, maxDepth - 1)
            if (found != null) return found
        }
    }
    return null
}
