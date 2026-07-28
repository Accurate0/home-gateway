-- Collapse legacy graphql scopes on existing API keys into their replacements:
--   graphql:roborock:*  / graphql:valetudo:*  -> graphql:robot_vacuum:*
--   graphql:events:read                       -> graphql:home_assistant:read
-- Rewrites matching array elements and de-duplicates; keys without any legacy
-- scope are left untouched.
UPDATE api_keys
SET scopes = (
    SELECT array_agg(DISTINCT mapped)
    FROM unnest(scopes) AS original
    CROSS JOIN LATERAL (
        SELECT CASE
            WHEN original IN ('graphql:roborock:read', 'graphql:valetudo:read')
                THEN 'graphql:robot_vacuum:read'
            WHEN original IN ('graphql:roborock:write', 'graphql:valetudo:write')
                THEN 'graphql:robot_vacuum:write'
            WHEN original = 'graphql:events:read'
                THEN 'graphql:home_assistant:read'
            ELSE original
        END AS mapped
    ) AS m
)
WHERE scopes && ARRAY[
    'graphql:roborock:read',
    'graphql:valetudo:read',
    'graphql:roborock:write',
    'graphql:valetudo:write',
    'graphql:events:read'
];
