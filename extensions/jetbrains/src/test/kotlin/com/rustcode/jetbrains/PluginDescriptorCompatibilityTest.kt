package com.rustcode.jetbrains

import kotlin.test.Test
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

class PluginDescriptorCompatibilityTest {
    @Test
    fun jcefDependencyIsOptionalForCrossVersionCompatibility() {
        val pluginXml = assertNotNull(javaClass.getResource("/META-INF/plugin.xml")).readText()

        assertTrue(
            pluginXml.contains(
                """<depends optional="true" config-file="rustcode-jcef.xml">com.intellij.modules.jcef</depends>""",
            ),
            "JCEF must be optional so pre-2025.3.1 IDEs can still load the plugin",
        )
        assertNotNull(
            javaClass.getResource("/META-INF/rustcode-jcef.xml"),
            "the optional JCEF dependency descriptor must be packaged",
        )
    }
}
