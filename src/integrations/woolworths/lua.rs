use crate::lua::{LuaCallContext, LuaClass, lua_module};

#[derive(LuaClass)]
#[lua(output)]
pub struct WoolworthsTrackedPrice {
    product_id: i64,
    price: Option<f64>,
}

pub struct WoolworthsLua;

#[lua_module(namespace = "woolworths")]
impl WoolworthsLua {
    #[lua(scope = Woolworths::Read)]
    async fn price(cx: &LuaCallContext, product_id: i64) -> mlua::Result<Option<f64>> {
        let prices = cx
            .query(Self::PRICE, || async {
                cx.state.repos.woolworths().prices().await
            })
            .await?;

        Ok(prices.get(&product_id).copied())
    }

    #[lua(scope = Woolworths::Read)]
    async fn tracked(cx: &LuaCallContext) -> mlua::Result<Vec<WoolworthsTrackedPrice>> {
        let repo = cx.state.repos.woolworths();

        let tracked = cx
            .query(Self::TRACKED, || async { repo.tracked_products().await })
            .await?;

        let prices = cx
            .query(Self::TRACKED, || async { repo.prices().await })
            .await?;

        Ok(tracked
            .into_iter()
            .map(|product| WoolworthsTrackedPrice {
                product_id: product.product_id,
                price: prices.get(&product.product_id).copied(),
            })
            .collect())
    }
}
