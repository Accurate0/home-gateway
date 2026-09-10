package net.infk8s.homegateway

import androidx.annotation.DrawableRes

enum class AppTab(val label: String, @param:DrawableRes val icon: Int) {
    HOME("Home", R.drawable.ic_nav_home),
    NOTIFICATIONS("Notifications", R.drawable.ic_nav_notifications),
}
