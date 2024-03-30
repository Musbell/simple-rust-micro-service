pub async fn on_response(res: Result<reqwest::Response, reqwest::Error>) -> Result<(), String> {
    res.map_err(|err| format!("HTTP error: {:?}", err.status()))
        .and_then(|res| {
            if res.status().is_success() {
                Ok(())
            } else {
                Err(format!("Unexpected response status: {:?}", res.status()))
            }
        })
}
