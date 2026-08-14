#![allow(unused)]

use anyhow::Result;

#[tokio::test]
async fn test_client() -> Result<()> {
    let client = reqwest::Client::new();

    let response = client
        .get("http://localhost:8080/greet?name=Cybro&message=Lmao ")
        .send()
        .await?;

    println!("{:?}", response);
    // println!("status: {}", response.status());
    println!("body: {}", response.text().await?);

    // Response { 
        // url: "http://localhost:8080/greet?name=Cybro&message=Lmao",
        // status: 200,
        // headers: {
        // "content-type": "text/html; charset=utf-8",
        // "content-length": "25",
        // "date": "Fri, 14 Aug 2026 04:04:37 GMT"
    // }}
    // body: Lmao<strong>Cybro<strong>

    Ok(())
}
