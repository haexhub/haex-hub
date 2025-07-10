package space.haex.hub
import android.os.Build
import android.os.Bundle
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.provider.Settings

class MainActivity : TauriActivity(){

    /* override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        startActivity(
            Intent(
                Settings.ACTION_MANAGE_APP_ALL_FILES_ACCESS_PERMISSION,
                Uri.parse("package:${BuildConfig.APPLICATION_ID}")
            )
        )
    } */
}