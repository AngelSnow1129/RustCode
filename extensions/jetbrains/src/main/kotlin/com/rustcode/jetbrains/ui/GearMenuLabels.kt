package com.rustcode.jetbrains.ui

import com.rustcode.jetbrains.i18n.RustCodeBundle
import java.util.Locale

internal data class GearMenuLabels(
    val connectStart: String,
    val provider: String,
    val createProvider: String,
    val editProvider: String,
    val deleteProvider: String,
    val thinkingSettings: String,
    val login: String,
    val sessionHistory: String,
    val renameSession: String,
    val deleteSession: String,
    val refreshSessions: String,
    val openChanges: String,
    val diagnostics: String,
    val settings: String,
)

internal fun gearMenuLabels(locale: Locale = Locale.getDefault()): GearMenuLabels =
    GearMenuLabels(
        connectStart = RustCodeBundle.message(locale, "gear.connectStart"),
        provider = RustCodeBundle.message(locale, "gear.provider"),
        createProvider = RustCodeBundle.message(locale, "gear.createProvider"),
        editProvider = RustCodeBundle.message(locale, "gear.editProvider"),
        deleteProvider = RustCodeBundle.message(locale, "gear.deleteProvider"),
        thinkingSettings = RustCodeBundle.message(locale, "gear.thinkingSettings"),
        login = RustCodeBundle.message(locale, "gear.login"),
        sessionHistory = RustCodeBundle.message(locale, "gear.sessionHistory"),
        renameSession = RustCodeBundle.message(locale, "gear.renameSession"),
        deleteSession = RustCodeBundle.message(locale, "gear.deleteSession"),
        refreshSessions = RustCodeBundle.message(locale, "gear.refreshSessions"),
        openChanges = RustCodeBundle.message(locale, "gear.openChanges"),
        diagnostics = RustCodeBundle.message(locale, "gear.diagnostics"),
        settings = RustCodeBundle.message(locale, "gear.settings"),
    )
