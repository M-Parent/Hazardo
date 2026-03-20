package com.hazardo.app

import android.app.Activity
import android.content.ContentValues
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.Environment
import android.provider.MediaStore
import android.util.Log
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.Plugin

@InvokeArg
class SaveArgs {
  lateinit var path: String
}

@TauriPlugin
class GalleryPlugin(private val activity: Activity) : Plugin(activity) {

  companion object {
    private const val TAG = "GalleryPlugin"
  }

  @Command
  fun saveToGallery(invoke: Invoke) {
    val args = invoke.parseArgs(SaveArgs::class.java)
    val filePath = args.path
    Log.d(TAG, "saveToGallery called: $filePath")

    try {
      val file = java.io.File(filePath)
      if (!file.exists()) {
        Log.e(TAG, "File not found: $filePath")
        invoke.reject("File not found: $filePath")
        return
      }
      Log.d(TAG, "File size: ${file.length()} bytes")

      val mimeType = when {
        filePath.endsWith(".png") -> "image/png"
        filePath.endsWith(".webp") -> "image/webp"
        filePath.endsWith(".gif") -> "image/gif"
        else -> "image/jpeg"
      }

      if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
        // Android 10+ : MediaStore with scoped storage
        val values = ContentValues().apply {
          put(MediaStore.Images.Media.DISPLAY_NAME, file.name)
          put(MediaStore.Images.Media.MIME_TYPE, mimeType)
          put(MediaStore.Images.Media.RELATIVE_PATH, Environment.DIRECTORY_DCIM + "/Hazardo")
          put(MediaStore.Images.Media.IS_PENDING, 1)
        }
        val uri = activity.contentResolver.insert(MediaStore.Images.Media.EXTERNAL_CONTENT_URI, values)
        if (uri != null) {
          activity.contentResolver.openOutputStream(uri)?.use { outputStream ->
            file.inputStream().use { inputStream ->
              inputStream.copyTo(outputStream)
            }
          }
          values.clear()
          values.put(MediaStore.Images.Media.IS_PENDING, 0)
          activity.contentResolver.update(uri, values, null, null)
          file.delete()
          Log.d(TAG, "Image saved to DCIM/Hazardo: ${file.name}")
          invoke.resolve()
        } else {
          Log.e(TAG, "MediaStore insert returned null")
          invoke.reject("MediaStore insert failed")
        }
      } else {
        // Android 9 and below
        @Suppress("DEPRECATION")
        val dcimDir = Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_DCIM)
        val hazardoDir = java.io.File(dcimDir, "Hazardo")
        if (!hazardoDir.exists()) hazardoDir.mkdirs()
        val outFile = java.io.File(hazardoDir, file.name)
        file.copyTo(outFile, overwrite = true)
        file.delete()
        @Suppress("DEPRECATION")
        activity.sendBroadcast(Intent(Intent.ACTION_MEDIA_SCANNER_SCAN_FILE, Uri.fromFile(outFile)))
        Log.d(TAG, "Image saved to ${outFile.absolutePath}")
        invoke.resolve()
      }
    } catch (e: Exception) {
      Log.e(TAG, "Failed to save to gallery", e)
      invoke.reject("Exception: ${e.message}")
    }
  }
}
