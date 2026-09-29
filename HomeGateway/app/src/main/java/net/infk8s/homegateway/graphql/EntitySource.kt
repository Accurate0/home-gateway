package net.infk8s.homegateway.graphql

import android.util.Log
import com.apollographql.apollo.ApolloClient

data class EntitySnapshot(val entities: List<EntityUi>, val categories: List<CategoryUi>)

class EntitySource(private val apollo: ApolloClient) {
    @Volatile
    private var cached: CachedSnapshot? = null

    suspend fun fetch(): EntitySnapshot {
        val response = apollo.query(EntitiesQuery()).execute()
        val data = response.data
            ?: throw IllegalStateException(
                response.errors?.firstOrNull()?.message ?: "Failed to load entities",
            )
        // Partial errors (e.g. an unreachable actor) only null individual fields,
        // not the entity — keep rendering the list and just log them.
        if (response.hasErrors()) {
            Log.w(TAG, "entities query returned field errors: ${response.errors}")
        }

        val entities = data.entities.mapNotNull { it.toUi() }
        // Section order and titles are a backend concern: render whatever it returns,
        // then append any category it didn't describe so nothing silently disappears.
        val described = data.entitySections.map { CategoryUi(it.category, it.title) }
        val undescribed = entities
            .map { it.category }
            .distinct()
            .filterNot { category -> described.any { it.category == category } }
            .map { CategoryUi(it, it.rawValue.lowercase()) }

        return EntitySnapshot(entities, described + undescribed).also(::update)
    }

    suspend fun current(): EntitySnapshot {
        val snapshot = cached
        if (snapshot != null && System.currentTimeMillis() - snapshot.at <= MAX_AGE_MS) {
            return snapshot.snapshot
        }

        return fetch()
    }

    fun update(snapshot: EntitySnapshot) {
        cached = CachedSnapshot(snapshot, System.currentTimeMillis())
    }

    fun invalidate() {
        cached = null
    }

    private data class CachedSnapshot(val snapshot: EntitySnapshot, val at: Long)

    private companion object {
        const val TAG = "EntitySource"
        const val MAX_AGE_MS = 30_000L
    }
}
