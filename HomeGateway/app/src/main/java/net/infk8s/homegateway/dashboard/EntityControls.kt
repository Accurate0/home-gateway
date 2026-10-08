package net.infk8s.homegateway.dashboard

import net.infk8s.homegateway.graphql.type.AirPurifierMode

/// The mutations a card can invoke, passed down so the composables stay
/// previewable and free of a ViewModel dependency.
data class EntityControls(
    val run: (String, EntityCommand) -> Unit,
    val setLight: (String, Boolean) -> Unit,
    val setBrightness: (String, Int) -> Unit,
    val setColourTemperature: (String, Int) -> Unit,
    val colourTemperatureMove: (String, Int) -> Unit,
    val setColour: (String, String) -> Unit,
    val mediaPlayPause: (String) -> Unit,
    val mediaStop: (String) -> Unit,
    val vacuumStart: (String) -> Unit,
    val vacuumStop: (String) -> Unit,
    val vacuumDock: (String) -> Unit,
    val garageDoorOpen: (String) -> Unit,
    val garageDoorClose: (String) -> Unit,
    val setAirPurifier: (String, Boolean) -> Unit,
    val setAirPurifierMode: (String, AirPurifierMode) -> Unit,
    val setAirPurifierSpeed: (String, Int) -> Unit,
    val setAirPurifierDisplay: (String, Boolean) -> Unit,
    val takeScreenshot: (String) -> Unit,
    val loadEinkConfig: suspend (String) -> EinkConfigUi?,
)
