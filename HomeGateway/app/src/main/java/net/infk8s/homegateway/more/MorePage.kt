package net.infk8s.homegateway.more

enum class MorePage(val title: String, val description: String) {
    TRANSPERTH("Transperth", "Next departures for configured routes"),
    FUEL("Fuel prices", "Cheapest nearby unleaded, today and tomorrow"),
    WOOLWORTHS("Woolworths", "Tracked product prices"),
    JELLYFIN("Jellyfin", "What's playing right now"),
    BATTERIES("Batteries", "Battery level history per device"),
    PLANTS("Plants", "Soil moisture history"),
    ADHOC_TASKS("Adhoc tasks", "Scheduled and one-shot maintenance tasks"),
    HOME_ASSISTANT("Home Assistant", "Live entity state feed"),
}
