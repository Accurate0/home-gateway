package net.infk8s.homegateway

import com.google.firebase.messaging.FirebaseMessagingService
import com.google.firebase.messaging.RemoteMessage
import net.infk8s.homegateway.notifications.Notifications
import net.infk8s.homegateway.notifications.PushPayload

class PushService : FirebaseMessagingService() {
    override fun onNewToken(token: String) {
        // Fired on background thread by the SDK; safe to do network directly.
        PushTokenRegistrar.register(token)
    }

    override fun onMessageReceived(message: RemoteMessage) {
        val payload = PushPayload.from(message.data) ?: return

        Notifications.createChannels(this)
        Notifications.show(this, payload)
    }
}
