use crate::model::{SearchTrending, SearchTrendingItemCoin};

use super::model::{Coin, CoinInfo, CoinMarket, ExchangeRates, MarketChart};
use primitives::FiatRate;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::Error;
use std::collections::HashMap;

const COINGECKO_API_URL: &str = "https://api.coingecko.com";
const COINGECKO_API_PRO_URL: &str = "https://pro-api.coingecko.com";
static USER_AGENT_VALUE: HeaderValue = HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36");

#[derive(Clone)]
pub struct CoinGeckoClient {
    client: reqwest::Client,
    url: String,
    api_key: String,
}

impl CoinGeckoClient {
    pub fn new(api_key: String) -> Self {
        let client = reqwest::Client::new();
        let url = Self::url_for_api_key(api_key.clone());
        Self {
            client,
            url,
            api_key,
        }
    }

    fn url_for_api_key(api_key: String) -> String {
        if !api_key.is_empty() {
            return COINGECKO_API_PRO_URL.to_string();
        }
        COINGECKO_API_URL.to_string()
    }

    pub fn convert_coin_vec_to_map(coins: Vec<Coin>) -> HashMap<String, Coin> {
        coins
            .into_iter()
            .map(|coin| (coin.id.clone(), coin))
            .collect()
    }

    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, USER_AGENT_VALUE.clone());
        headers
    }

    pub async fn get_search_trending(&self) -> Result<Vec<SearchTrendingItemCoin>, Error> {
        let url = format!(
            "{}/api/v3/search/trending?x_cg_pro_api_key={}",
            self.url, self.api_key
        );
        let response = self.client.get(&url).headers(self.headers()).send().await?;
        let coins: SearchTrending = response.json().await?;
        Ok(coins.coins.into_iter().map(|x| x.item).collect())
    }

    pub async fn get_coin_list(&self) -> Result<Vec<Coin>, Error> {
        let url = format!(
            "{}/api/v3/coins/list?include_platform=true&x_cg_pro_api_key={}",
            self.url, self.api_key
        );
        let response = self.client.get(&url).headers(self.headers()).send().await?;
        let coins: Vec<Coin> = response.json().await?;
        Ok(coins)
    }

    pub async fn get_coin_markets(
        &self,
        page: u32,
        per_page: u32,
    ) -> Result<Vec<CoinMarket>, Error> {
        let url = format!(
            "{}/api/v3/coins/markets?vs_currency=usd&order=market_cap_desc&per_page={}&page={}&sparkline=false&locale=en&x_cg_pro_api_key={}",
            self.url, per_page, page, self.api_key
        );
        let response = self.client.get(&url).headers(self.headers()).send().await?;
        response.json().await
    }

    pub async fn get_coin_markets_id(
        &self,
        id: &str,
    ) -> Result<CoinMarket, Box<dyn std::error::Error>> {
        let url = format!(
            "{}/api/v3/coins/markets?vs_currency=usd&ids={}&order=market_cap_desc&sparkline=false&locale=en&x_cg_pro_api_key={}",
            self.url, id, self.api_key
        );
        let response = self
            .client
            .get(&url)
            .headers(self.headers())
            .send()
            .await?
            .json::<Vec<CoinMarket>>()
            .await?;

        if let Some(market) = response.first() {
            Ok(market.clone())
        } else {
            Err("market not found".into())
        }
    }

    pub async fn get_coin(&self, coin: &str) -> Result<CoinInfo, Error> {
        let url = format!("{}/api/v3/coins/{}?x_cg_pro_api_key={}&market_data=false&community_data=true&tickers=false&localization=false&developer_data=true", self.url, coin, self.api_key);
        let response = self.client.get(&url).headers(self.headers()).send().await?;
        response.json().await
    }

    pub async fn get_fiat_rates(&self) -> Result<Vec<FiatRate>, Error> {
        let url = format!(
            "{}/api/v3/exchange_rates?x_cg_pro_api_key={}",
            self.url, self.api_key
        );
        let response = self.client.get(&url).headers(self.headers()).send().await?;

        let rates: ExchangeRates = response.json().await?;

        let fiat_rates: Vec<FiatRate> = rates
            .rates
            .into_iter()
            .filter(|x| x.1.rate_type == "fiat")
            .map(|x| FiatRate {
                symbol: x.0.to_uppercase(),
                name: x.1.name,
                rate: x.1.value,
            })
            .collect();
        Ok(fiat_rates)
    }

    pub async fn get_all_coin_markets(
        &self,
        per_page: u32,
        pages: u32,
    ) -> Result<Vec<CoinMarket>, Error> {
        let mut all_coin_markets = Vec::new();
        let mut page = 1;

        loop {
            let coin_markets = self.get_coin_markets(page, per_page).await?;

            all_coin_markets.extend(coin_markets.clone());

            if coin_markets.is_empty() || page == pages {
                break;
            }

            page += 1;
        }

        Ok(all_coin_markets)
    }

    pub async fn get_market_chart(&self, coin_id: &str) -> Result<MarketChart, Error> {
        let url = format!("{}/api/v3/coins/{}/market_chart?vs_currency=usd&days=max&interval=daily&precision=full&x_cg_pro_api_key={}",self.url, coin_id, self.api_key);
        let response = self
            .client
            .get(url)
            .send()
            .await?
            .json::<MarketChart>()
            .await?;

        Ok(response)
    }
}
