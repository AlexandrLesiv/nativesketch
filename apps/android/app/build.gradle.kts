import java.time.Duration

plugins { id("com.android.application") }

android {
    namespace = "dev.nativesketch"
    compileSdk = 37
    // Additive checkpoint: preserve exact inspected JNI bytes in the provisional APK.
    packaging.jniLibs.keepDebugSymbols += "**/libsketch_app_jni.so"
    defaultConfig {
        applicationId = "dev.nativesketch"
        minSdk = 26
        targetSdk = 37
        versionCode = 1
        versionName = "0.1"
        ndk { abiFilters += listOf("arm64-v8a", "x86_64") }
    }
}

dependencies {
    implementation("com.google.android.material:material:1.13.0")
    implementation("androidx.activity:activity:1.13.0")
}

// Local JVM compatibility lane; no device, emulator, Android runtime or Robolectric.
dependencies { testImplementation("junit:junit:4.13.2") }

afterEvaluate {
    val ordinaryUnitTests = tasks.named<Test>("testDebugUnitTest")
    ordinaryUnitTests.configure { exclude("**/*JniHostTest.class") }

    fun registerHostLane(taskName: String, probeOnly: Boolean) =
        tasks.register<Test>(taskName) {
            group = "verification"
            description =
                "Required real Rust JNI host-JVM tests; fails if the host dylib is missing"
            val original = ordinaryUnitTests.get()
            dependsOn(original.taskDependencies.getDependencies(original))
            testClassesDirs = original.testClassesDirs
            classpath = original.classpath
            if (probeOnly) include("**/PendingExceptionJniHostTest.class")
            else {
                include("**/*JniHostTest.class")
                exclude("**/PendingExceptionJniHostTest.class")
            }
            maxParallelForks = 1
            jvmArgs("--enable-native-access=ALL-UNNAMED")
            timeout.set(Duration.ofMinutes(2))
            outputs.upToDateWhen { false }
            doFirst {
                layout.buildDirectory
                    .file("jni-host-classpath.txt")
                    .get()
                    .asFile
                    .writeText(classpath.asPath)
                val property = if (probeOnly) "portableJniProbeLibrary" else "portableJniLibrary"
                val path = providers.gradleProperty(property).orNull
                check(path != null && file(path).isAbsolute && file(path).isFile) {
                    "Pass -PportableJniLibrary=/absolute/path/to/the/actual/host/library"
                }
                systemProperty("nativesketch.jni.path", path)
                if (probeOnly) systemProperty("nativesketch.jni.probe.path", path)
                systemProperty(
                    "nativesketch.wire.fixtures",
                    rootProject.projectDir
                        .resolve("../../crates/sketch-app-wire/tests/fixtures/v2.json")
                        .canonicalPath,
                )
            }
        }
    tasks.register<JavaExec>("portableJniDiscover") {
        val original = ordinaryUnitTests.get()
        dependsOn(original.taskDependencies.getDependencies(original))
        classpath = original.classpath
        mainClass.set("dev.nativesketch.portable.JniDiscoveryKt")
        args(
            "InputJniHostTest",
            "LifetimeJniHostTest",
            "PreflightJniHostTest",
            "PreparedJniHostTest",
            "WorkerJniHostTest",
            "PendingExceptionJniHostTest",
        )
        systemProperty(
            "nativesketch.discovery.output",
            layout.buildDirectory.file("jni-discovery.txt").get().asFile.absolutePath,
        )
        doFirst {
            layout.buildDirectory
                .file("jni-host-classpath.txt")
                .get()
                .asFile
                .writeText(classpath.asPath)
        }
    }
    registerHostLane("portableJniHostTest", false)
    registerHostLane("portableJniPendingExceptionTest", true)
}
