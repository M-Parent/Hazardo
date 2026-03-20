package com.hazardo.app

import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.content.res.Configuration
import android.graphics.Color
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.util.Log
import android.view.View
import android.webkit.PermissionRequest
import android.webkit.WebChromeClient
import android.webkit.WebResourceRequest
import android.webkit.WebView
import android.webkit.WebViewClient
import androidx.activity.result.contract.ActivityResultContracts
import androidx.annotation.Keep
import androidx.appcompat.app.AppCompatDelegate
import androidx.core.content.ContextCompat
import androidx.core.view.WindowCompat

class MainActivity : TauriActivity() {

  private var isRecreating = false

  companion object {
    private const val TAG = "Hazardo"
    private const val PREFS_NAME = "hazardo_theme"
    private const val KEY_IS_DARK = "is_dark"
    private const val KEY_PERMISSIONS_REQUESTED = "permissions_requested"
  }

  private val requestMultiplePermissionsLauncher = registerForActivityResult(
    ActivityResultContracts.RequestMultiplePermissions()
  ) { _ -> }

  @Suppress("DEPRECATION")
  override fun onCreate(savedInstanceState: Bundle?) {
    val prefs = applicationContext.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
    val savedIsDark = prefs.getBoolean(KEY_IS_DARK, false)
    val nightMode = if (savedIsDark) AppCompatDelegate.MODE_NIGHT_YES else AppCompatDelegate.MODE_NIGHT_NO
    AppCompatDelegate.setDefaultNightMode(nightMode)

    super.onCreate(savedInstanceState)

    requestAppPermissions()

    // Edge-to-edge
    WindowCompat.setDecorFitsSystemWindows(window, false)
    window.statusBarColor = Color.TRANSPARENT
    window.navigationBarColor = Color.TRANSPARENT
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
      window.isStatusBarContrastEnforced = false
      window.isNavigationBarContrastEnforced = false
    }

    // Post-setup: find WebView and configure
    window.decorView.post {
      val webView = findWebView(window.decorView)

      webView?.let {
        it.addJavascriptInterface(StatusBarBridge(), "HazardoNative")

        it.evaluateJavascript("""
          (function() {
            if (window.__hazardoThemeObserver) return;
            window.__hazardoThemeObserver = new MutationObserver(function(mutations) {
              mutations.forEach(function(m) {
                if (m.attributeName === 'class') {
                  var isDark = document.documentElement.classList.contains('dark');
                  if (window.HazardoNative) {
                    window.HazardoNative.setDarkMode(isDark);
                  }
                }
              });
            });
            window.__hazardoThemeObserver.observe(
              document.documentElement,
              { attributes: true, attributeFilter: ['class'] }
            );
          })();
        """.trimIndent(), null)

        val originalClient = it.webChromeClient
        it.webChromeClient = object : WebChromeClient() {
          override fun onPermissionRequest(request: PermissionRequest) {
            request.grant(request.resources)
          }

          override fun onProgressChanged(view: WebView?, newProgress: Int) {
            originalClient?.onProgressChanged(view, newProgress)
            if (newProgress == 100 && view != null) {
              syncThemeFromWebView(view)
            }
          }
        }

        // Intercept external URL navigation — open in real browser/app instead of WebView
        val originalWebViewClient = it.webViewClient
        it.webViewClient = object : WebViewClient() {
          private fun handleUrl(url: String): Boolean {
            if (url.startsWith("intent://")) {
              try {
                val intent = Intent.parseUri(url, Intent.URI_INTENT_SCHEME)
                intent.addCategory(Intent.CATEGORY_BROWSABLE)
                intent.component = null
                if (intent.resolveActivity(packageManager) != null) {
                  startActivity(intent)
                } else {
                  val fallback = intent.getStringExtra("browser_fallback_url")
                  if (!fallback.isNullOrEmpty()) {
                    startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(fallback)))
                  }
                }
              } catch (e: Exception) {
                Log.e(TAG, "Failed to handle intent URL", e)
              }
              return true
            }

            if (url.startsWith("geo:") || url.startsWith("google.navigation:")) {
              try {
                startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(url)))
              } catch (e: Exception) {
                Log.e(TAG, "Failed to open geo URL", e)
              }
              return true
            }

            if (!url.startsWith("http://") && !url.startsWith("https://")) {
              try {
                startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(url)))
              } catch (e: Exception) {
                Log.e(TAG, "Failed to open scheme URL", e)
              }
              return true
            }

            if (!url.startsWith("http://localhost") && !url.startsWith("https://localhost") &&
                !url.contains("tauri.localhost")) {
              try {
                startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(url)))
              } catch (e: Exception) {
                Log.e(TAG, "Failed to open URL", e)
              }
              return true
            }
            return false
          }

          override fun shouldOverrideUrlLoading(view: WebView?, request: WebResourceRequest?): Boolean {
            val url = request?.url?.toString() ?: return false
            if (handleUrl(url)) return true
            return originalWebViewClient?.shouldOverrideUrlLoading(view, request) ?: false
          }

          @Deprecated("Deprecated in API level 24")
          override fun shouldOverrideUrlLoading(view: WebView?, url: String?): Boolean {
            if (url != null && handleUrl(url)) return true
            @Suppress("DEPRECATION")
            return originalWebViewClient?.shouldOverrideUrlLoading(view, url) ?: false
          }

          override fun onReceivedError(view: WebView?, request: WebResourceRequest?, error: android.webkit.WebResourceError?) {
            val url = request?.url?.toString() ?: ""
            if (url.startsWith("intent://") || (!url.startsWith("http") && url.contains("://"))) {
              handleUrl(url)
              view?.goBack()
              return
            }
            originalWebViewClient?.onReceivedError(view, request, error)
          }
        }

        for (delay in listOf(500L, 1500L, 3000L)) {
          it.postDelayed({ syncThemeFromWebView(it) }, delay)
        }
      }
    }
  }

  private fun applyDarkMode(isDark: Boolean) {
    val prefs = applicationContext.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
    val currentSaved = prefs.getBoolean(KEY_IS_DARK, false)

    if (currentSaved == isDark) return

    prefs.edit().putBoolean(KEY_IS_DARK, isDark).apply()

    val mode = if (isDark) AppCompatDelegate.MODE_NIGHT_YES else AppCompatDelegate.MODE_NIGHT_NO
    AppCompatDelegate.setDefaultNightMode(mode)

    if (!isRecreating) {
      isRecreating = true
      recreate()
    }
  }

  private fun syncThemeFromWebView(webView: WebView) {
    webView.evaluateJavascript(
      "(function(){ return JSON.stringify({isDark: document.documentElement.classList.contains('dark')}); })()"
    ) { result ->
      val isDark = result?.contains("true") == true
      runOnUiThread { applyDarkMode(isDark) }
    }
  }

  private fun findWebView(view: View): WebView? {
    if (view is WebView) return view
    if (view is android.view.ViewGroup) {
      for (i in 0 until view.childCount) {
        findWebView(view.getChildAt(i))?.let { return it }
      }
    }
    return null
  }

  private fun requestAppPermissions() {
    val prefs = applicationContext.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
    if (prefs.getBoolean(KEY_PERMISSIONS_REQUESTED, false)) return

    val permissions = mutableListOf(
      android.Manifest.permission.ACCESS_FINE_LOCATION,
      android.Manifest.permission.ACCESS_COARSE_LOCATION,
      android.Manifest.permission.CAMERA
    )
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
      permissions.add(android.Manifest.permission.READ_MEDIA_IMAGES)
      permissions.add(android.Manifest.permission.READ_MEDIA_VIDEO)
      permissions.add(android.Manifest.permission.READ_MEDIA_VISUAL_USER_SELECTED)
    } else if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
      permissions.add(android.Manifest.permission.READ_MEDIA_IMAGES)
      permissions.add(android.Manifest.permission.READ_MEDIA_VIDEO)
    } else {
      permissions.add(android.Manifest.permission.READ_EXTERNAL_STORAGE)
    }
    val notGranted = permissions.filter {
      ContextCompat.checkSelfPermission(this, it) != PackageManager.PERMISSION_GRANTED
    }
    if (notGranted.isNotEmpty()) {
      requestMultiplePermissionsLauncher.launch(notGranted.toTypedArray())
    }
    prefs.edit().putBoolean(KEY_PERMISSIONS_REQUESTED, true).apply()
  }

  override fun onConfigurationChanged(newConfig: Configuration) {
    super.onConfigurationChanged(newConfig)
  }

  @Keep
  inner class StatusBarBridge {
    @Keep
    @android.webkit.JavascriptInterface
    fun setDarkMode(isDark: Boolean) {
      runOnUiThread { applyDarkMode(isDark) }
    }
  }
}
