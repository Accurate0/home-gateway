use mlua::{Lua, Table};

use crate::auth::scope::{Action, Resource, Scope};
use crate::lua::{LuaCallContext, LuaClass, LuaField, LuaFunction, LuaModule, LuaParam, LuaType};

const TRACKED_PRICE: LuaClass = LuaClass {
    name: "WoolworthsTrackedPrice",
    fields: &[
        LuaField {
            name: "product_id",
            ty: LuaType::Integer,
        },
        LuaField {
            name: "price",
            ty: LuaType::Optional(&LuaType::Number),
        },
    ],
};

const PRICE: LuaFunction = LuaFunction {
    name: "price",
    params: &[LuaParam {
        name: "product_id",
        ty: LuaType::Integer,
    }],
    returns: Some(LuaType::Optional(&LuaType::Number)),
    scope: Some(Scope::new(Resource::Woolworths, Action::Read)),
};

const TRACKED: LuaFunction = LuaFunction {
    name: "tracked",
    params: &[],
    returns: Some(LuaType::Array(&LuaType::Class(&TRACKED_PRICE))),
    scope: Some(Scope::new(Resource::Woolworths, Action::Read)),
};

const FUNCTIONS: &[LuaFunction] = &[PRICE, TRACKED];

pub struct WoolworthsLua;

impl LuaModule for WoolworthsLua {
    fn namespace(&self) -> &'static str {
        "woolworths"
    }

    fn functions(&self) -> &'static [LuaFunction] {
        FUNCTIONS
    }

    fn register(&self, lua: &Lua, table: &Table, cx: &LuaCallContext) -> mlua::Result<()> {
        let price_cx = cx.clone();
        cx.expose(table, &PRICE, || {
            lua.create_async_function(move |_, product_id: i64| {
                let cx = price_cx.clone();

                async move {
                    let prices = cx
                        .query("woolworths.price", || async {
                            cx.state.repos.woolworths().prices().await
                        })
                        .await?;

                    Ok(prices.get(&product_id).copied())
                }
            })
        })?;

        let tracked_cx = cx.clone();
        cx.expose(table, &TRACKED, || {
            lua.create_async_function(move |lua, ()| {
                let cx = tracked_cx.clone();

                async move {
                    let repo = cx.state.repos.woolworths();

                    let tracked = cx
                        .query("woolworths.tracked", || async {
                            repo.tracked_products().await
                        })
                        .await?;

                    let prices = cx
                        .query("woolworths.tracked", || async { repo.prices().await })
                        .await?;

                    let result = lua.create_table()?;

                    for product in tracked {
                        let row = lua.create_table()?;

                        row.set("product_id", product.product_id)?;
                        row.set("price", prices.get(&product.product_id).copied())?;

                        result.push(row)?;
                    }

                    Ok(result)
                }
            })
        })
    }
}
