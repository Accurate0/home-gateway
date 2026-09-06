package net.infk8s.homegateway

import android.app.Application
import net.infk8s.homegateway.notifications.Notifications

class HomeGatewayApplication : Application() {
    override fun onCreate() {
        super.onCreate()
        Notifications.createChannels(this)
    }
}
