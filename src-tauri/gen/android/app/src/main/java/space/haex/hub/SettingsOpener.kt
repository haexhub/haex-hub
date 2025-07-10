package space.haex.hub

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.provider.Settings

class SettingsOpener {
    companion object {
        @JvmStatic
        fun openManageAllFilesAccessSettings(context: Context) {
            val intent = Intent(Settings.ACTION_MANAGE_ALL_FILES_ACCESS_PERMISSION)
            intent.data = Uri.parse("package:" + context.packageName)
            context.startActivity(intent)
        }
    }
}