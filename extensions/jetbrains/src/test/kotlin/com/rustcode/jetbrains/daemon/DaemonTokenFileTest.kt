package com.rustcode.jetbrains.daemon

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Assumptions.assumeTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import java.nio.file.Files

class DaemonTokenFileTest {
    @BeforeEach
    fun guardRustcodeHome() {
        // RUSTCODE_HOME cannot be cleared from JVM at runtime; skip the test if a CI-exported
        // value would shadow the user.home fallback that these tests rely on.
        assumeTrue(
            System.getenv("RUSTCODE_HOME").isNullOrEmpty(),
            "RUSTCODE_HOME is set in the environment; skipping user.home-fallback tests to avoid shadowing.",
        )
    }

    @Test
    fun readsTokenFromFile() {
        val home = Files.createTempDirectory("rc-jb").toFile()
        System.setProperty("user.home", home.absolutePath)
        val f = home.resolve(".rustcode").apply { mkdirs() }.resolve("daemon-13456.json")
        f.writeText("""{"pid":1,"port":13456,"token":"jb-tok"}""")
        assertEquals("jb-tok", DaemonTokenFile.read(13456))
    }

    @Test
    fun returnsNullWhenMissing() {
        val home = Files.createTempDirectory("rc-jb2").toFile()
        System.setProperty("user.home", home.absolutePath)
        assertNull(DaemonTokenFile.read(19999))
    }

    @Test
    fun returnsNullOnInvalidJson() {
        val home = Files.createTempDirectory("rc-jb4").toFile()
        System.setProperty("user.home", home.absolutePath)
        val f = home.resolve(".rustcode").apply { mkdirs() }.resolve("daemon-13456.json")
        f.writeText("not-valid-json")
        assertNull(DaemonTokenFile.read(13456))
    }

    @Test
    fun returnsNullWhenTokenFieldMissing() {
        val home = Files.createTempDirectory("rc-jb5").toFile()
        System.setProperty("user.home", home.absolutePath)
        val f = home.resolve(".rustcode").apply { mkdirs() }.resolve("daemon-13456.json")
        f.writeText("""{"pid":1,"port":13456}""")
        assertNull(DaemonTokenFile.read(13456))
    }
}
