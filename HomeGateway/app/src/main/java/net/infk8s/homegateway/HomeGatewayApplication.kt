package net.infk8s.homegateway

import android.app.Application
import android.content.Context
import net.infk8s.homegateway.auth.AuthSession
import net.infk8s.homegateway.dashboard.DashboardDatabase
import net.infk8s.homegateway.graphql.ApolloProvider
import net.infk8s.homegateway.notifications.Notifications

class HomeGatewayApplication : Application() {
    val auth by lazy { AuthSession(this) }

    val apollo by lazy { ApolloProvider.build(auth) }

    val database by lazy { DashboardDatabase.build(this) }

    override fun onCreate() {
        super.onCreate()
        Notifications.createChannels(this)
    }
}

val Context.gateway: HomeGatewayApplication
    get() = applicationContext as HomeGatewayApplication
