package net.infk8s.homegateway

import android.app.NotificationChannel
import android.app.NotificationManager
import android.content.Context
import androidx.core.app.NotificationCompat
import androidx.core.content.getSystemService
import com.google.firebase.messaging.FirebaseMessagingService
import com.google.firebase.messaging.RemoteMessage

class PushService : FirebaseMessagingService() {
    override fun onNewToken(token: String) {
        // Fired on background thread by the SDK; safe to do network directly.
        PushTokenRegistrar.register(token)
    }

    override fun onMessageReceived(message: RemoteMessage) {
        val notification = message.notification ?: return
        val manager = getSystemService<NotificationManager>() ?: return

        // Channels are required since Android 8; creating an existing one is a no-op.
        manager.createNotificationChannel(
            NotificationChannel(
                CHANNEL_ID,
                "Home Gateway",
                NotificationManager.IMPORTANCE_HIGH,
            )
        )

        val builder = NotificationCompat.Builder(this, CHANNEL_ID)
            .setSmallIcon(android.R.drawable.ic_dialog_info)
            .setContentTitle(notification.title)
            .setContentText(notification.body)
            .setPriority(NotificationCompat.PRIORITY_HIGH)
            .setAutoCancel(true)

        manager.notify(System.currentTimeMillis().toInt(), builder.build())
    }

    companion object {
        private const val CHANNEL_ID = "home_gateway_default"
    }
}
