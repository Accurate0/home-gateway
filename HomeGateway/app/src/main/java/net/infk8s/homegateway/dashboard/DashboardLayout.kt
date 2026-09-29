package net.infk8s.homegateway.dashboard

import net.infk8s.homegateway.graphql.EntitySectionUi

data class DashboardLayout(
    val sections: Map<String, Int>,
    val tiles: Map<String, Int>,
) {
    fun arrange(unordered: List<EntitySectionUi>): List<EntitySectionUi> = unordered
        .sortedBy { sections[it.category.rawValue] ?: Int.MAX_VALUE }
        .map { section ->
            section.copy(items = section.items.sortedBy { tiles[it.key] ?: Int.MAX_VALUE })
        }

    fun withSections(order: List<String>) = copy(sections = sections + order.positions())

    fun withTiles(order: List<String>) = copy(tiles = tiles + order.positions())

    companion object {
        fun from(sections: List<SectionPosition>, tiles: List<TilePosition>) = DashboardLayout(
            sections = sections.associate { it.category to it.position },
            tiles = tiles.associate { it.key to it.position },
        )
    }
}

fun <T> MutableList<T>.move(from: T, to: T): Boolean {
    val fromIndex = indexOf(from)
    val toIndex = indexOf(to)
    if (fromIndex < 0 || toIndex < 0 || fromIndex == toIndex) {
        return false
    }

    add(toIndex, removeAt(fromIndex))
    return true
}

private fun List<String>.positions(): Map<String, Int> =
    withIndex().associate { it.value to it.index }
