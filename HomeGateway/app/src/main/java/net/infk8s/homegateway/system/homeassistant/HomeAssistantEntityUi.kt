package net.infk8s.homegateway.system.homeassistant

import java.time.Instant

data class HomeAssistantEntityUi(val entityId: String, val state: String, val time: Instant)
