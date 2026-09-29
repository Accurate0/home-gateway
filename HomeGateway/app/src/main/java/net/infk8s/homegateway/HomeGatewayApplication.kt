package net.infk8s.homegateway

import android.app.Application
import android.content.Context
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import net.infk8s.homegateway.auth.AuthSession
import net.infk8s.homegateway.dashboard.DashboardDatabase
import net.infk8s.homegateway.dashboard.DashboardPreferences
import net.infk8s.homegateway.graphql.ApolloProvider
import net.infk8s.homegateway.graphql.EntitySource
import net.infk8s.homegateway.notifications.Notifications
import net.infk8s.homegateway.quicktiles.QuickTileAssignments

class HomeGatewayApplication : Application() {
    val auth by lazy { AuthSession(this) }

    val apollo by lazy { ApolloProvider.build(auth) }

    val entities by lazy { EntitySource(apollo) }

    val database by lazy { DashboardDatabase.build(this) }

    val dashboardPreferences by lazy { DashboardPreferences(this) }

    val quickTileAssignments by lazy { QuickTileAssignments(this) }

    val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main.immediate)

    override fun onCreate() {
        super.onCreate()
        Notifications.createChannels(this)
    }
}

val Context.gateway: HomeGatewayApplication
    get() = applicationContext as HomeGatewayApplication
