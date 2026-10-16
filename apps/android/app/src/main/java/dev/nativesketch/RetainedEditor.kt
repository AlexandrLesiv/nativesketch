package dev.nativesketch

import android.graphics.*
import android.view.*
import android.widget.*
import kotlin.math.*

internal data class Retained(
    val handle: Long,
    val busy: Boolean,
    val bytes: ByteArray?,
    val revision: Long,
    val replace: Int,
    val pen: Boolean,
    val ink: Int,
    val width: Double,
    val zoom: Double,
    val ox: Double,
    val oy: Double,
    val uri: android.net.Uri?,
    val permission: android.view.DragAndDropPermissions?,
)
