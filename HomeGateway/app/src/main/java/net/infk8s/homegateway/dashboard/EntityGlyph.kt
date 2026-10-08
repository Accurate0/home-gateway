package net.infk8s.homegateway.dashboard

import androidx.annotation.DrawableRes
import net.infk8s.homegateway.R

enum class EntityGlyph(@param:DrawableRes val drawable: Int) {
    LIGHT_ON(R.drawable.ic_light_on),
    LIGHT_OFF(R.drawable.ic_light_off),
    DOOR_OPEN(R.drawable.ic_door_open),
    DOOR_CLOSED(R.drawable.ic_door_closed),
    GARAGE(R.drawable.ic_garage),
    FAN(R.drawable.ic_fan),
    PERSON(R.drawable.ic_person),
    PERSON_AWAY(R.drawable.ic_person_away),
    THERMOMETER(R.drawable.ic_thermometer),
    PLANT(R.drawable.ic_plant),
    DISPLAY(R.drawable.ic_display),
    ROBOT(R.drawable.ic_robot),
    TV(R.drawable.ic_tv),
}
