package net.infk8s.homegateway.fuel

data class FuelSiteUi(
    val siteId: Int,
    val name: String,
    val brand: String,
    val suburb: String,
    val address: String,
    val price: Double,
    val priceTomorrow: Double?,
    val latitude: Double,
    val longitude: Double,
)
