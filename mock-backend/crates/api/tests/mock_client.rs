#![allow(unused)]

use anyhow::Result;
use serde_json::json;

#[tokio::test]
async fn test_client() -> Result<()> {
    let client = reqwest::Client::new();
    let base_url = "http://localhost:8080";

    // ? is only when i want to explicitly set what variable
    // will get what value, which can be extracted by the server
    // by .name, .message etc (variables of a struct)
    let response1 = client
        .get(format!("{base_url}/greet?name=Cybro&message=Lmao "))
        .send()
        .await?;

    // a value placed directly after / is a path parameter
    // server can define it as path/{parameter} and extract
    // with Path(parameter)
    let response2 = client
        .get(format!("{base_url}/greet2/cybro"))
        .send()
        .await?;

    let response3 = client
        .get(format!("{base_url}/service_file.html"))
        .send()
        .await?;

    let login_response = client.post(format!("{base_url}/login"))
        .json(&json!({
            "username": "cybro",
            "password": "cybro123"
        }))
        .send()
        .await?;
    
    println!("From response1:");
    println!("{:?}", response1);
    // println!("status: {}", response.status());
    println!("body: {}", response1.text().await?);

    println!("\nFrom response2:");
    println!("{:?}", response2);
    println!("Body: {}", response2.text().await?);

    println!("\nFrom response3");
    println!("{:?}", response3);
    println!("Body: {}", response3.text().await?);

    println!("\nFrom login:");
    println!("{:?}", login_response);
    println!("Body: {}", login_response.text().await?);
    
    // From response1:
    // Response {
    // url: "http://localhost:8080/greet?name=Cybro&message=Lmao",
    // status: 200,
    // headers: {
    // "content-type": "text/html; charset=utf-8",
    // "content-length": "25", "date": "Sun, 16 Aug 2026 13:28:58 GMT"
    // }}
    // body: Lmao<strong>Cybro<strong>
    
    // From response2:
    // Response { 
    // url: "http://localhost:8080/greet2/cybro",
    // status: 200,
    // headers: {
    // "content-type": "text/html; charset=utf-8",
    // "content-length": "41", "date": "Sun, 16 Aug 2026 13:28:58 GMT"
    // }}
    // Body: I am using your name 'cybro' to greet you
    
    Ok(())
}
