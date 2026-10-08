package net.infk8s.homegateway.graphql

import net.infk8s.homegateway.graphql.type.Capability
import net.infk8s.homegateway.ui.parseInstant

fun EntitiesQuery.Entity.toUi(): EntityUi? = when {
    onLightEntity != null ->
        EntityUi.Light(
            id = onLightEntity.id,
            name = onLightEntity.name,
            room = onLightEntity.room,
            category = onLightEntity.category,
            on = onLightEntity.on,
            dimmable = onLightEntity.capabilities.any { it == Capability.BRIGHTNESS },
            tunable = onLightEntity.capabilities.any { it == Capability.COLOUR_TEMP },
            colour = onLightEntity.capabilities.any { it == Capability.RGB },
        )

    onDoorEntity != null ->
        EntityUi.Door(
            id = onDoorEntity.id,
            name = onDoorEntity.name,
            room = onDoorEntity.room,
            category = onDoorEntity.category,
            open = onDoorEntity.open,
        )

    onGarageDoorEntity != null ->
        EntityUi.GarageDoor(
            id = onGarageDoorEntity.id,
            name = onGarageDoorEntity.name,
            room = onGarageDoorEntity.room,
            category = onGarageDoorEntity.category,
            state = onGarageDoorEntity.garageState,
            batteryPercentage = onGarageDoorEntity.battery?.percentage,
        )

    onAirPurifierEntity != null ->
        EntityUi.AirPurifier(
            id = onAirPurifierEntity.id,
            name = onAirPurifierEntity.name,
            room = onAirPurifierEntity.room,
            category = onAirPurifierEntity.category,
            on = onAirPurifierEntity.on,
            mode = onAirPurifierEntity.purifierMode,
            speed = onAirPurifierEntity.purifierSpeed,
            pm25 = onAirPurifierEntity.purifierPm25,
            aqi = onAirPurifierEntity.aqi,
            cadr = onAirPurifierEntity.cadr,
            filterLife = onAirPurifierEntity.filterLife,
            displayOn = onAirPurifierEntity.displayOn,
        )

    onPresenceEntity != null ->
        EntityUi.Presence(
            id = onPresenceEntity.id,
            name = onPresenceEntity.name,
            room = onPresenceEntity.room,
            category = onPresenceEntity.category,
            present = onPresenceEntity.present,
        )

    onEnvironmentEntity != null ->
        EntityUi.Environment(
            id = onEnvironmentEntity.id,
            name = onEnvironmentEntity.name,
            room = onEnvironmentEntity.room,
            category = onEnvironmentEntity.category,
            temperature = onEnvironmentEntity.temperature,
            humidity = onEnvironmentEntity.humidity,
            pressure = onEnvironmentEntity.pressure,
            lux = onEnvironmentEntity.lux,
            uvIndex = onEnvironmentEntity.uvIndex,
            pm25 = onEnvironmentEntity.pm25,
            vocIndex = onEnvironmentEntity.vocIndex,
            lastSeen = parseInstant(onEnvironmentEntity.lastSeen),
        )

    onPlantEntity != null ->
        EntityUi.Plant(
            id = onPlantEntity.id,
            name = onPlantEntity.name,
            room = onPlantEntity.room,
            category = onPlantEntity.category,
            soilMoisture = onPlantEntity.soilMoisture,
            batteryPercentage = onPlantEntity.battery?.percentage,
        )

    onEinkDisplayEntity != null ->
        EntityUi.EinkDisplay(
            id = onEinkDisplayEntity.id,
            name = onEinkDisplayEntity.name,
            room = onEinkDisplayEntity.room,
            category = onEinkDisplayEntity.category,
            batteryPercentage = onEinkDisplayEntity.batteryPercentage,
            isCharging = onEinkDisplayEntity.isCharging,
        )

    onRobotVacuumEntity != null ->
        EntityUi.RobotVacuum(
            id = onRobotVacuumEntity.id,
            name = onRobotVacuumEntity.name,
            room = onRobotVacuumEntity.room,
            category = onRobotVacuumEntity.category,
            status = onRobotVacuumEntity.status,
            batteryPercentage = onRobotVacuumEntity.batteryPercentage,
            currentRoom = onRobotVacuumEntity.currentRoom,
        )

    onMediaPlayerEntity != null ->
        EntityUi.MediaPlayer(
            id = onMediaPlayerEntity.id,
            name = onMediaPlayerEntity.name,
            room = onMediaPlayerEntity.room,
            category = onMediaPlayerEntity.category,
            playing = onMediaPlayerEntity.playing,
            appName = onMediaPlayerEntity.appName,
            mediaTitle = onMediaPlayerEntity.mediaTitle,
            mediaSeriesTitle = onMediaPlayerEntity.mediaSeriesTitle,
        )

    else -> null
}
