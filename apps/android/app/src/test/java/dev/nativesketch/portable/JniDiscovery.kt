package dev.nativesketch.portable

import java.io.File
import org.junit.Test

/** Enumerates compiled @Test methods without initializing native libraries or test instances. */
fun main(args: Array<String>) {
    val ids =
        args
            .flatMap { simple ->
                val name = "dev.nativesketch.portable.$simple"
                Class.forName(name, false, Thread.currentThread().contextClassLoader)
                    .declaredMethods
                    .filter { it.getAnnotation(Test::class.java) != null }
                    .map { "$name:${it.name}" }
            }
            .sorted()
    check(ids.isNotEmpty() && ids.distinct().size == ids.size)
    File(requireNotNull(System.getProperty("nativesketch.discovery.output")))
        .writeText(ids.joinToString("\n") + "\n")
}
