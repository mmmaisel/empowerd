#[macro_use] extern crate lazy_static;
extern crate reqwest;
extern crate regex;

use regex::Regex;

pub struct DachsClient
{
    client: reqwest::Client,
    url: String,
    password: String
}

lazy_static!
{
  static ref EXTR_TOTAL_ENERGY: regex::Regex = Regex::new(
      r"BD3112\.Hka_Bd\.ulArbeitElektr=([0-9]+\.[0-9]+)\s").unwrap();
  static ref EXTR_RUNTIME: regex::Regex = Regex::new(
      r"Hka_Bd\.ulBetriebssekunden=([0-9]+\.[0-9]+)\s").unwrap();
}

// TODO: dont panic or unwrap or expect

impl DachsClient
{
    const USERNAME: &'static str = "glt";
    const KEY_TOTAL_ENERGY: &'static str = "BD3112.Hka_Bd.ulArbeitElektr";
    const KEY_RUNTIME: &'static str = "Hka_Bd.ulBetriebssekunden";

    pub fn new(url: String, password: String)
        -> DachsClient
    {
        let client = reqwest::Client::new();
        return DachsClient
        {
            client: client,
            url: format!("http://{}:8080", url),
            password: password
        };
    }

    fn query_glt(&self, key: &str) -> Result<String, String>
    {
        let result = self.client.get(
            &format!("{url}/getKey?k={key}", url = self.url, key = key)).
            basic_auth(DachsClient::USERNAME, Some(&self.password)).
            send();

        let text = match result
        {
            Ok(mut result) =>
            {
                match result.status()
                {
                    reqwest::StatusCode::OK =>
                    {
                        result.text()
                    },
                    reqwest::StatusCode::UNAUTHORIZED =>
                    {
                        return Err("💩️ Unauthorized".to_string());
                    },
                    _ =>
                    {
                        return Err("💩️ Dachs GLT API returned an error".to_string());
                    }
                }
            }
            Err(_) => return Err("💩️ Querying Dachs GLT API failed.".to_string())
        };

        return match text
        {
            Ok(text) =>
            {
                if cfg!(debug_assertions)
                {
                    println!("query_glt {}: {}", key, &text);
                }
                Ok(text)
            }
            Err(_) => return
                Err("💩️ Decoding Dachs GLT API response failed.".to_string())
        }
    }

    pub fn get_total_energy(&self) -> Result<i32, String>
    {
        let result = self.query_glt(DachsClient::KEY_TOTAL_ENERGY)?;
        let extracted_val = match EXTR_TOTAL_ENERGY.captures(&result)
        {
            Some(x) => x,
            None => return Err("💩️ Parsing Dachs total energy failed.".to_string())
        };

        let energy: f64 = extracted_val.get(1).unwrap().
            as_str().parse().unwrap();

        if cfg!(debug_assertions)
        {
            println!("Energy f64: {}", &energy);
        }
        return Ok(energy as i32);
    }

    pub fn get_runtime(&self) -> Result<i32, String>
    {
        let result = self.query_glt(DachsClient::KEY_RUNTIME)?;
        let extracted_val = match EXTR_RUNTIME.captures(&result)
        {
            Some(x) => x,
            None => return Err("💩️ Parsing Dachs runtime failed.".to_string())
        };

        let runtime_h: f64 = extracted_val.get(1).unwrap().
            as_str().parse().unwrap();

        if cfg!(debug_assertions)
        {
            println!("Runtime h: {}", &runtime_h);
        }
        return Ok((runtime_h * 3600.0) as i32);
    }
}
