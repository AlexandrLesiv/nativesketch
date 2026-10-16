package dev.nativesketch

import android.graphics.*
import android.view.*
import android.widget.*
import kotlin.math.*

internal fun acceptsDrawingDrop(description: android.content.ClipDescription?): Boolean =
    description?.let {
        it.hasMimeType("text/uri-list") ||
            it.hasMimeType("application/octet-stream") ||
            it.hasMimeType("application/x-native-sketch")
    } == true
