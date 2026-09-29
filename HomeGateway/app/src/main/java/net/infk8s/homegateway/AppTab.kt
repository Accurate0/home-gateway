package net.infk8s.homegateway

import androidx.annotation.DrawableRes

enum class AppTab(val label: String, @param:DrawableRes val icon: Int) {
    HOME("Home", R.drawable.ic_nav_home),
    WORKFLOWS("Workflows", R.drawable.ic_nav_workflows),
    MODES("Modes", R.drawable.ic_nav_modes),
    NOTIFICATIONS("Notifications", R.drawable.ic_nav_notifications),
}
