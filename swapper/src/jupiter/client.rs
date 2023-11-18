
use primitives::{SwapQuote, SwapQuoteProtocolRequest, ChainType, SwapProvider, SwapQuoteEthereumData};

use super::model::{QuoteRequest, QuoteResponse, QuoteDataResponse, QuoteDataRequest};

const NATIVE_ADDRESS: &str = "So11111111111111111111111111111111111111112";

pub struct JupiterClient {
    api_url: String,
    fee: f64,
    fee_referral_address: String,
    client: reqwest::Client,
}

impl JupiterClient {
    pub fn new(api_url: String, fee: f64, fee_referral_address: String) -> Self {
        let client = reqwest::Client::builder()
            .build().unwrap();

        Self {
            client,
            api_url,
            fee,
            fee_referral_address,
        }
    }

    pub fn provider(&self) -> SwapProvider {
        SwapProvider { name: "Jupiter".to_string() }
    }

    pub async fn get_quote(&self, quote: SwapQuoteProtocolRequest) -> Result<SwapQuote, Box<dyn std::error::Error + Send + Sync>> {
        let input_mint = if quote.from_asset.clone().is_native() { NATIVE_ADDRESS.to_string() } else { quote.from_asset.clone().token_id.unwrap() };
        let output_mint = if quote.to_asset.clone().is_native() { NATIVE_ADDRESS.to_string() } else { quote.to_asset.clone().token_id.unwrap() };

        let quote_request: QuoteRequest = QuoteRequest{
            input_mint,
            output_mint,
            amount: quote.amount.clone(),
            platform_fee_bps: (self.fee * 10.0) as i32,
        };
        let swap_quote = self.get_swap_quote(quote_request).await?;
        let data = if quote.include_data {
            let data = self.get_data(quote.clone(), swap_quote.clone()).await?;
            Some(data)
        } else {
            None
        };

        print!("fee: {}", self.fee);

        let quote = SwapQuote {
            chain_type: ChainType::Solana, 
            from_amount: quote.amount.clone(),
            to_amount: swap_quote.out_amount.clone(),
            fee_percent: self.fee as f32,
            provider: self.provider(),
            data,
        };
        return Ok(quote)
    }

    pub async fn get_data(&self, quote: SwapQuoteProtocolRequest, quote_response: QuoteResponse) -> Result<SwapQuoteEthereumData, Box<dyn std::error::Error + Send + Sync>> {
        let request = QuoteDataRequest{
            user_public_key: quote.wallet_address, 
            fee_account: self.fee_referral_address.clone(),
            quote_response: quote_response.clone()
        };
        let quote_data = self.get_swap_quote_data(request).await?;
        let data = SwapQuoteEthereumData{
            data: quote_data.swap_transaction,
            to: "".to_string(),
            value: "".to_string(),
            gas_limit: 0,
        };
        return Ok(data)
    }

    pub async fn get_swap_quote(&self, request: QuoteRequest) -> Result<QuoteResponse, Box<dyn std::error::Error + Send + Sync>>   {
        let params = serde_urlencoded::to_string(&request)?;
        let url = format!("{}/v6/quote?{}", self.api_url, params);
        return Ok(self.client
            .get(&url)
            .send()
            .await?
            .json::<QuoteResponse>()
            .await?);
    }

    pub async fn get_swap_quote_data(&self, request: QuoteDataRequest) -> Result<QuoteDataResponse, Box<dyn std::error::Error + Send + Sync>>   {
        let url = format!("{}/v6/swap", self.api_url);
        return Ok(self.client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .json::<QuoteDataResponse>()
            .await?);
    }
}