use rusoto_core::Region;
use rusoto_dynamodb::{DynamoDb, DynamoDbClient, PutItemInput, AttributeValue};
use lambda_http::{handler, lambda, Context, IntoResponse, Request};
use serde_json::json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

type Error = Box<dyn std::error::Error + Sync + Send + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda::run(handler(hello)).await?;
    Ok(())
}

async fn hello(req: Request, _: Context) -> Result<impl IntoResponse, Error> {
    // `serde_json::Values` impl `IntoResponse` by default
    // creating an application/json response
    let (_, body) = req.into_parts();
    match serde_json::from_slice(&body) { 
        Ok(user) =>  {
            add_member(user).await?;
            Ok(json!({
                "message": "ok"
            }))

        },
        Err(err) => {
            Ok(json!({
                "error": err.to_string()
            }))
        }
    }
}


fn string_attr(string: &String) -> AttributeValue {
    let mut attr: AttributeValue = Default::default();
    attr.s = Some(string.to_string());
    attr
}


#[derive(Serialize, Deserialize)]
struct User {
  student_number : String,
  first_name : String, 
  last_name : String,
  preferred_name : String
}

async fn add_member(user: User) -> Result<User, String> {
    let client = DynamoDbClient::new(Region::UsEast1);
    let mut put_item_input : PutItemInput = Default::default();
    let mut new_item: HashMap<String, AttributeValue> = HashMap::new();

    new_item.insert("student_number".to_string(), string_attr(&user.student_number.to_string()));
    new_item.insert("first_name".to_string(),  string_attr(&user.first_name.to_string()));
    new_item.insert("last_name".to_string(), string_attr(&user.last_name.to_string()));
    new_item.insert("preferred_name".to_string(), string_attr(&user.preferred_name.to_string()));
    put_item_input.item = new_item;
    put_item_input.table_name = "TPCMembers".to_string();

    match client.put_item(put_item_input).await {
        Ok(_) => {
            Ok(user)
        },
        Err(err) =>
            Err(err.to_string())
        
    }
}
