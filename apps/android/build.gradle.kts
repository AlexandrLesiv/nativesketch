plugins { id("com.android.application") version "9.1.1" apply false }

// Official Maven artifact; the formatter is local to this project, not an IDE/global setting.
val kotlinFormatter by configurations.creating

repositories { mavenCentral() }

dependencies { kotlinFormatter("com.facebook:ktfmt:0.64:with-dependencies") }

val handwrittenKotlin =
    fileTree(projectDir) {
        include("**/*.kt", "**/*.kts")
        exclude("**/build/**", "**/.gradle/**")
    }

fun registerFormatter(name: String, check: Boolean) =
    tasks.register<JavaExec>(name) {
        group = "verification"
        description =
            if (check) "Check Kotlin formatting without changing files"
            else "Format handwritten Kotlin"
        classpath = kotlinFormatter
        mainClass.set("com.facebook.ktfmt.cli.Main")
        args("--kotlinlang-style")
        if (check) args("--dry-run", "--set-exit-if-changed")
        args(handwrittenKotlin.files.sortedBy { it.path }.map { it.path })
    }

registerFormatter("kotlinFormat", false)

registerFormatter("kotlinFormatCheck", true)

tasks.register("kotlinSourceSizeCheck") {
    group = "verification"
    description = "Enforce 200 physical lines on handwritten Kotlin after formatting"
    doLast {
        val oversized = handwrittenKotlin.files.filter { it.readLines().size > 200 }
        check(oversized.isEmpty()) {
            oversized.joinToString("\n", "Source files exceed 200 lines:\n") {
                it.relativeTo(projectDir).path
            }
        }
    }
}

tasks.register("androidQuality") {
    group = "verification"
    description = "Background Kotlin format/size, Android lint and JVM unit checks (not native E2E)"
    dependsOn(
        "kotlinFormatCheck",
        "kotlinSourceSizeCheck",
        ":app:lintDebug",
        ":app:testDebugUnitTest",
    )
}
