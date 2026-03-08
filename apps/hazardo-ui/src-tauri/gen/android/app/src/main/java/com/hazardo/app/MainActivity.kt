package com.hazardo.app

import android.content.res.Configuration
import android.os.Build
import android.os.Bundle
import android.view.View
import androidx.activity.enableEdgeToEdge
import androidx.core.content.ContextCompat
import androidx.core.graphics.ColorUtils
import androidx.core.view.WindowCompat

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
    updateStatusBarAppearance()
  }

  override fun onResume() {
    super.onResume()
    updateStatusBarAppearance()
  }

  override fun onConfigurationChanged(newConfig: Configuration) {
    super.onConfigurationChanged(newConfig)
    updateStatusBarAppearance()
  }

  private fun updateStatusBarAppearance() {
    // Determine background color for status bar. Prefer explicit resource, fall back to current statusBarColor
    val bgColor = try {
      ContextCompat.getColor(this, com.hazardo.app.R.color.hazardo_background)
    } catch (e: Exception) {
      window.statusBarColor
    }

    // Calculate luminance: >0.5 => light background => use dark icons
    val luminance = ColorUtils.calculateLuminance(bgColor)
    val lightStatusIcons = luminance > 0.5

    // Apply via WindowInsetsController (AndroidX)
    try {
      WindowCompat.getInsetsController(window, window.decorView)?.isAppearanceLightStatusBars = lightStatusIcons
    } catch (e: NoSuchMethodError) {
      // ignore and fallback
    }

    // Fallback for older APIs using systemUiVisibility flag
    if (Build.VERSION.SDK_INT < Build.VERSION_CODES.R) {
      val decor = window.decorView
      var flags = decor.systemUiVisibility
      flags = if (lightStatusIcons) {
        flags or View.SYSTEM_UI_FLAG_LIGHT_STATUS_BAR
      } else {
        flags and View.SYSTEM_UI_FLAG_LIGHT_STATUS_BAR.inv()
      }
      decor.systemUiVisibility = flags
    }
  }
}
