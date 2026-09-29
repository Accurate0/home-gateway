package net.infk8s.homegateway.dashboard

import net.infk8s.homegateway.graphql.CategoryUi
import net.infk8s.homegateway.graphql.EntitySectionUi
import net.infk8s.homegateway.graphql.EntityUi

data class DashboardLayout(
    val mode: DashboardMode,
    val sections: Map<DashboardMode, Map<String, Int>>,
    val tiles: Map<DashboardMode, Map<String, Int>>,
    val customSections: List<CustomSection>,
    val customTiles: Map<String, CustomTile>,
) {
    fun arrange(entities: List<EntityUi>, categories: List<CategoryUi>): List<EntitySectionUi> =
        when (mode) {
            DashboardMode.TYPE -> byType(entities, categories)
            DashboardMode.ROOM -> byRoom(entities)
            DashboardMode.CUSTOM -> custom(entities)
        }

    fun withSectionOrder(order: List<String>) =
        copy(sections = sections + (mode to sections[mode].orEmpty() + order.positions()))

    fun withTileOrder(order: List<String>) =
        copy(tiles = tiles + (mode to tiles[mode].orEmpty() + order.positions()))

    fun withCustomSections(updated: List<CustomSection>): DashboardLayout {
        val ids = updated.map { it.id }.toSet()

        return copy(
            customSections = updated,
            customTiles = customTiles.filterValues { it.sectionId in ids },
        )
    }

    fun withCustomTiles(placed: List<CustomTile>, unplaced: Collection<String>) = copy(
        customTiles = customTiles - unplaced.toSet() + placed.associateBy { it.key },
    )

    private fun byType(entities: List<EntityUi>, categories: List<CategoryUi>): List<EntitySectionUi> = categories
        .mapNotNull { category ->
            val items = entities.filter { it.category == category.category }
            if (items.isEmpty()) {
                null
            } else {
                EntitySectionUi(category.category.rawValue, category.title, items.ordered(), editable = false)
            }
        }
        .ordered()

    private fun byRoom(entities: List<EntityUi>): List<EntitySectionUi> = entities
        .groupBy { it.room.orEmpty() }
        .toList()
        .sortedWith(compareBy({ it.first.isEmpty() }, { it.first.lowercase() }))
        .map { (room, items) ->
            EntitySectionUi(room, room.ifEmpty { NO_ROOM_TITLE }, items.ordered(), editable = false)
        }
        .ordered()

    private fun custom(entities: List<EntityUi>): List<EntitySectionUi> {
        val byKey = entities.associateBy { it.key }
        val placed = customTiles.values.groupBy { it.sectionId }
        val ids = customSections.map { it.id }.toSet()

        val sections = customSections.sortedBy { it.position }.map { section ->
            val items = placed[section.id].orEmpty()
                .sortedBy { it.position }
                .mapNotNull { byKey[it.key] }

            EntitySectionUi(section.id, section.title, items, editable = true)
        }

        val unsorted = entities.filter { customTiles[it.key]?.sectionId !in ids }
        if (unsorted.isEmpty()) {
            return sections
        }

        return sections + EntitySectionUi(UNSORTED_KEY, UNSORTED_TITLE, unsorted.ordered(), editable = false)
    }

    private fun List<EntityUi>.ordered(): List<EntityUi> {
        val positions = tiles[mode].orEmpty()

        return sortedBy { it.name.lowercase() }.sortedBy { positions[it.key] ?: Int.MAX_VALUE }
    }

    @JvmName("orderedSections")
    private fun List<EntitySectionUi>.ordered(): List<EntitySectionUi> {
        val positions = sections[mode].orEmpty()

        return sortedBy { positions[it.key] ?: Int.MAX_VALUE }
    }

    companion object {
        const val UNSORTED_KEY = "unsorted"

        private const val UNSORTED_TITLE = "Unsorted"
        private const val NO_ROOM_TITLE = "No room"
        private const val HEADER_PREFIX = "section:"

        fun headerKey(section: String) = "$HEADER_PREFIX$section"

        fun isHeaderKey(key: String) = key.startsWith(HEADER_PREFIX)

        fun sectionOfHeader(key: String) = key.removePrefix(HEADER_PREFIX)

        fun from(
            mode: DashboardMode,
            sections: List<LayoutSection>,
            tiles: List<LayoutTile>,
            customSections: List<CustomSection>,
            customTiles: List<CustomTile>,
        ) = DashboardLayout(
            mode = mode,
            sections = sections.groupBy { it.mode }.mapValues { (_, rows) -> rows.associate { it.key to it.position } },
            tiles = tiles.groupBy { it.mode }.mapValues { (_, rows) -> rows.associate { it.key to it.position } },
            customSections = customSections,
            customTiles = customTiles.associateBy { it.key },
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
