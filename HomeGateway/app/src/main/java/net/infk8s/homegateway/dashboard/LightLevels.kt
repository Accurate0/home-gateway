package net.infk8s.homegateway.dashboard

import androidx.compose.runtime.mutableStateMapOf

class LightLevels {
    val brightness = mutableStateMapOf<String, Float>()

    val mireds = mutableStateMapOf<String, Float>()
}
