package com.rustcode.jetbrains.settings

import com.intellij.openapi.options.Configurable
import com.rustcode.jetbrains.i18n.RustCodeBundle
import java.awt.BorderLayout
import java.awt.GridBagConstraints
import java.awt.GridBagLayout
import javax.swing.JCheckBox
import javax.swing.JComboBox
import javax.swing.JComponent
import javax.swing.JLabel
import javax.swing.JPanel
import javax.swing.JSpinner
import javax.swing.JTextField
import javax.swing.SpinnerNumberModel

class RustCodeConfigurable : Configurable {
    private var panel: JPanel? = null
    private val settings = RustCodeSettingsState.getInstance()

    private lateinit var daemonPath: JTextField
    private lateinit var host: JTextField
    private lateinit var port: JSpinner
    private lateinit var autoStart: JCheckBox
    private lateinit var autoSaveBeforeRead: JCheckBox
    private lateinit var timeout: JSpinner
    private lateinit var contextLevel: JComboBox<RustCodeContextLevel>
    private lateinit var allowSelection: JCheckBox
    private lateinit var sendRelativePath: JCheckBox
    private lateinit var sendWithCtrlEnter: JCheckBox
    private lateinit var chatFontSize: JSpinner

    override fun getDisplayName(): String = "RustCode"

    override fun createComponent(): JComponent {
        val form = JPanel(GridBagLayout())
        var row = 0

        daemonPath = JTextField()
        host = JTextField()
        port = JSpinner(SpinnerNumberModel(13456, 1, 65535, 1))
        autoStart = JCheckBox(RustCodeBundle.message("settings.autoStart"))
        autoSaveBeforeRead = JCheckBox(RustCodeBundle.message("settings.autoSave"))
        timeout = JSpinner(SpinnerNumberModel(30_000, 1_000, 300_000, 1_000))
        contextLevel = JComboBox(RustCodeContextLevel.entries.toTypedArray())
        allowSelection = JCheckBox(RustCodeBundle.message("settings.allowSelection"))
        sendRelativePath = JCheckBox(RustCodeBundle.message("settings.sendRelativePath"))
        sendWithCtrlEnter = JCheckBox(RustCodeBundle.message("settings.ctrlEnter"))
        chatFontSize = JSpinner(SpinnerNumberModel(13, 9, 30, 1))

        form.addRow(row++, RustCodeBundle.message("settings.daemonPath"), daemonPath)
        form.addRow(row++, RustCodeBundle.message("settings.host"), host)
        form.addRow(row++, RustCodeBundle.message("settings.port"), port)
        form.addRow(row++, RustCodeBundle.message("settings.timeout"), timeout)
        form.addRow(row++, RustCodeBundle.message("settings.fontSize"), chatFontSize)
        form.addRow(row++, RustCodeBundle.message("settings.contextLevel"), contextLevel)
        form.addFullRow(row++, autoStart)
        form.addFullRow(row++, autoSaveBeforeRead)
        form.addFullRow(row++, allowSelection)
        form.addFullRow(row++, sendRelativePath)
        form.addFullRow(row++, sendWithCtrlEnter)

        panel = JPanel(BorderLayout()).apply {
            add(form, BorderLayout.NORTH)
        }
        reset()
        return panel!!
    }

    override fun isModified(): Boolean {
        val current = settings.state
        return daemonPath.text != current.daemonBinaryPath ||
            host.text != current.host ||
            port.value as Int != current.port ||
            autoStart.isSelected != current.autoStart ||
            autoSaveBeforeRead.isSelected != current.autoSaveBeforeRead ||
            timeout.value as Int != current.requestTimeoutMs ||
            chatFontSize.value as Int != current.chatFontSize ||
            contextLevel.selectedItem != current.contextLevel ||
            allowSelection.isSelected != current.allowSelectedTextContext ||
            sendRelativePath.isSelected != current.sendRelativePathWithSelection ||
            sendWithCtrlEnter.isSelected != current.sendWithCtrlEnter
    }

    override fun apply() {
        settings.update {
            it.daemonBinaryPath = daemonPath.text.trim()
            it.host = host.text.trim()
            it.port = port.value as Int
            it.autoStart = autoStart.isSelected
            it.autoSaveBeforeRead = autoSaveBeforeRead.isSelected
            it.requestTimeoutMs = timeout.value as Int
            it.chatFontSize = chatFontSize.value as Int
            it.contextLevel = contextLevel.selectedItem as RustCodeContextLevel
            it.allowSelectedTextContext = allowSelection.isSelected
            it.sendRelativePathWithSelection = sendRelativePath.isSelected
            it.sendWithCtrlEnter = sendWithCtrlEnter.isSelected
        }
    }

    override fun reset() {
        val current = settings.state
        daemonPath.text = current.daemonBinaryPath
        host.text = current.host
        port.value = current.port
        autoStart.isSelected = current.autoStart
        autoSaveBeforeRead.isSelected = current.autoSaveBeforeRead
        timeout.value = current.requestTimeoutMs
        chatFontSize.value = current.chatFontSize
        contextLevel.selectedItem = current.contextLevel
        allowSelection.isSelected = current.allowSelectedTextContext
        sendRelativePath.isSelected = current.sendRelativePathWithSelection
        sendWithCtrlEnter.isSelected = current.sendWithCtrlEnter
    }

    private fun JPanel.addRow(row: Int, label: String, component: JComponent) {
        add(JLabel(label), GridBagConstraints().apply {
            gridx = 0
            gridy = row
            anchor = GridBagConstraints.WEST
            insets.set(4, 4, 4, 8)
        })
        add(component, GridBagConstraints().apply {
            gridx = 1
            gridy = row
            weightx = 1.0
            fill = GridBagConstraints.HORIZONTAL
            insets.set(4, 0, 4, 4)
        })
    }

    private fun JPanel.addFullRow(row: Int, component: JComponent) {
        add(component, GridBagConstraints().apply {
            gridx = 0
            gridy = row
            gridwidth = 2
            anchor = GridBagConstraints.WEST
            insets.set(4, 4, 4, 4)
        })
    }
}
