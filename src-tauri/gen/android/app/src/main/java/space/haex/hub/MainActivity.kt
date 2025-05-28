package space.haex.hub
import android.webkit.WebView
import android.os.Bundle
import android.content.pm.ApplicationInfo

class MainActivity : TauriActivity(){
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        if (0!= (applicationInfo.flags and ApplicationInfo.FLAG_DEBUGGABLE)) {
            WebView.setWebContentsDebuggingEnabled(true)
        }
    }
}