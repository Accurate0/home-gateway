use mlua::{Lua, Table};

use crate::auth::scope::{Action, Resource, Scope};
use crate::lua::{LuaCallContext, LuaClass, LuaField, LuaFunction, LuaModule, LuaParam, LuaType};

use super::types::FuelSite;

const SITE: LuaClass = LuaClass {
    name: "FuelSite",
    fields: &[
        LuaField {
            name: "site_id",
            ty: LuaType::Integer,
        },
        LuaField {
            name: "name",
            ty: LuaType::String,
        },
        LuaField {
            name: "brand",
            ty: LuaType::String,
        },
        LuaField {
            name: "suburb",
            ty: LuaType::String,
        },
        LuaField {
            name: "postcode",
            ty: LuaType::Integer,
        },
        LuaField {
            name: "address",
            ty: LuaType::String,
        },
        LuaField {
            name: "price",
            ty: LuaType::Number,
        },
        LuaField {
            name: "price_tomorrow",
            ty: LuaType::Optional(&LuaType::Number),
        },
        LuaField {
            name: "latitude",
            ty: LuaType::Number,
        },
        LuaField {
            name: "longitude",
            ty: LuaType::Number,
        },
    ],
};

const CHEAPEST: LuaFunction = LuaFunction {
    name: "cheapest",
    params: &[
        LuaParam {
            name: "postcode",
            ty: LuaType::Optional(&LuaType::Integer),
        },
        LuaParam {
            name: "limit",
            ty: LuaType::Optional(&LuaType::Integer),
        },
    ],
    returns: Some(LuaType::Array(&LuaType::Class(&SITE))),
    scope: Some(Scope::new(Resource::FuelWatch, Action::Read)),
};

const FUNCTIONS: &[LuaFunction] = &[CHEAPEST];

pub struct FuelLua;

impl LuaModule for FuelLua {
    fn namespace(&self) -> &'static str {
        "fuel"
    }

    fn functions(&self) -> &'static [LuaFunction] {
        FUNCTIONS
    }

    fn register(&self, lua: &Lua, table: &Table, cx: &LuaCallContext) -> mlua::Result<()> {
        let cheapest_cx = cx.clone();

        cx.expose(table, &CHEAPEST, || {
            lua.create_async_function(
                move |lua, (postcode, limit): (Option<i32>, Option<i64>)| {
                    let cx = cheapest_cx.clone();

                    async move {
                        let postcode = match postcode.or_else(|| {
                            cx.state
                                .settings
                                .fuelwatch
                                .as_ref()
                                .map(|settings| settings.postcode)
                        }) {
                            Some(postcode) => postcode,
                            None => {
                                return Err(mlua::Error::external(
                                    "fuel.cheapest needs a postcode; none was given and fuelwatch is not configured",
                                ));
                            }
                        };

                        let sites = cx
                            .query("fuel.cheapest", || async {
                                cx.state
                                    .repos
                                    .fuelwatch()
                                    .sites_for_postcode(postcode, limit)
                                    .await
                            })
                            .await?;

                        let result = lua.create_table()?;

                        for site in &sites {
                            result.push(site_table(&lua, site)?)?;
                        }

                        Ok(result)
                    }
                },
            )
        })
    }
}

fn site_table(lua: &Lua, site: &FuelSite) -> mlua::Result<Table> {
    let table = lua.create_table()?;

    table.set("site_id", site.site_id)?;
    table.set("name", site.name.as_str())?;
    table.set("brand", site.brand.as_str())?;
    table.set("suburb", site.suburb.as_str())?;
    table.set("postcode", site.postcode)?;
    table.set("address", site.address.as_str())?;
    table.set("price", site.price)?;
    table.set("price_tomorrow", site.price_tomorrow)?;
    table.set("latitude", site.latitude)?;
    table.set("longitude", site.longitude)?;

    Ok(table)
}
