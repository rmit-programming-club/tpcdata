use rusoto_core::Region;
use rusoto_dynamodb::{DynamoDb, DynamoDbClient, PutItemInput, ScanInput, DeleteItemInput};
use lambda_http::{handler, lambda, Context, Request, Response, Body};
use serde_json::json;
use serde::{Deserialize, Serialize};
use std::env;
use async_trait::async_trait;

type Error = Box<dyn std::error::Error + Sync + Send + 'static>;

#[async_trait]
trait TpcDatastore {
    async fn add_member(&mut self, user: &User) -> Result<(), String>;
    async fn delete_member(&mut self, student_number: &str) -> Result<(), String>;
    async fn list_members(&self) -> Result<Vec<User>, String>;
}

pub struct DynamoDBTPCDatastore;

#[async_trait]
impl TpcDatastore for DynamoDBTPCDatastore {
    async fn add_member(&mut self, user: &User) -> Result<(), String> {
        let client = get_dynamo_client();
        let mut put_item_input : PutItemInput = Default::default();

        put_item_input.item = serde_dynamodb::to_hashmap(user).unwrap();
        put_item_input.table_name = "TPCMembers".to_string();

        match client.put_item(put_item_input).await {
            Ok(_) => {
                Ok(())
            },
            Err(err) =>
                Err(err.to_string())
            
        }
    }
    async fn delete_member(&mut self, student_number: &str) -> Result<(), String> {
        let client = get_dynamo_client();
        let mut delete_item_input : DeleteItemInput = Default::default();

        delete_item_input.key = serde_dynamodb::to_hashmap(student_number).unwrap();
        delete_item_input.table_name = String::from("TPCMembers");

        match client.delete_item(delete_item_input).await {
            Ok(_) => {
                Ok(())
            },
            Err(err) =>
                Err(err.to_string())
            
        }
    }
    async fn list_members(&self) -> Result<Vec<User>,String> {
        let client = get_dynamo_client();
        let mut scan_input: ScanInput = Default::default();

        scan_input.table_name = "TPCMembers".to_string();

        match client.scan(scan_input).await {
            Ok(output) => 
                match output.items {
                    Some(user_maps) => {
                        let users : Result<Vec<User>, serde_dynamodb::error::Error> = user_maps.iter().map(|item| serde_dynamodb::from_hashmap(item.to_owned())).collect();
                        users.map_err(|err| err.to_string())
                    }
                    None => {
                        Ok(Vec::new())
                    }
            },
            Err(err) =>
                Err(err.to_string())
        }
    }

}

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda::run(handler(prod_handler)).await?;
    Ok(())
}
async fn prod_handler(req: Request, context: Context) -> Result<Response<Body>, Error> {
    let mut datastore = DynamoDBTPCDatastore {};
    main_handler(&mut datastore, req, context).await
}

async fn main_handler(datastore: &mut impl TpcDatastore, req: Request, _: Context) -> Result<Response<Body>, Error> {
    // `serde_json::Values` impl `IntoResponse` by default
    // creating an application/json response
    match req.uri().path() {
        "/deregister" => 
            deregister(datastore, req).await,
        "/register" => 
            register(datastore, req).await,
        "/members" =>
            get_members(datastore, req).await,
        _ => {
            let error = json!({"error": "not found"});
            Ok(Response::builder()
               .status(404)
               .body(Body::from(serde_json::to_string(&error).unwrap()))
               .expect("failed to render response")
               )
        }
    }

    
}

async fn deregister(datastore: &mut impl TpcDatastore, req: Request) -> Result<Response<Body>, Error> {
    let (_, body) = req.into_parts();
    match body { 
        Body::Text(student_number) =>  {
            let user = datastore.delete_member(&student_number).await?;
            Ok(Response::builder()
               .status(200)
               .body(Body::from(serde_json::to_string(&user).unwrap()))
               .expect("failed to render response")
               )
        },
        _ => {
            let error = json!({"error": String::from("Not a valid body")});
            Ok(Response::builder()
               .status(401)
               .body(Body::from(serde_json::to_string(&error).unwrap()))
               .expect("failed to render response")
               )
        }
    }
}

async fn register(datastore: &mut impl TpcDatastore, req: Request) -> Result<Response<Body>, Error> {
    let (_, body) = req.into_parts();
    match serde_json::from_slice(&body) { 
        Ok(user) =>  {
            let user = datastore.add_member(&user).await?;
            Ok(Response::builder()
               .status(200)
               .body(Body::from(serde_json::to_string(&user).unwrap()))
               .expect("failed to render response")
               )

        },
        Err(err) => {
            let error = json!({"error": err.to_string()});
            Ok(Response::builder()
               .status(401)
               .body(Body::from(serde_json::to_string(&error).unwrap()))
               .expect("failed to render response")
               )
        }
    }
}

async fn get_members(datastore: &impl TpcDatastore, _ : Request) -> Result<Response<Body>, Error> {
    let members = datastore.list_members().await?;

    // Make sure only members that want to be seen are seen
    let filtered_members : Vec<&User> = members.iter().filter(|user| user.show_user).collect();

    match serde_json::to_string(&filtered_members).map_err(|err| err.to_string()) {
        Ok(member_json) => 
            {
                Ok(Response::builder()
                   .status(400)
                   .body(Body::from(member_json))
                   .expect("fail to create response")
                   )
            }
     
        Err(err) =>
            Ok(Response::builder()
               .status(500)
               .body(Body::from(err.to_string()))
               .expect("fail to pass body")
               )

    }

}

#[derive(Serialize, Deserialize,Debug,PartialEq,Clone)]
struct User {
  id : String,
  student_number : String,
  first_name : String, 
  last_name : String,
  preferred_name : String,
  preferred_email : String,
  discord_id : Option<String>,
  point_record : Option<PointRecord>,
  show_user : bool
}

#[derive(Serialize, Deserialize,Debug,PartialEq,Clone)]
struct UserResponse {
  id : String,
  email_md5 : Option<String>,
  first_name : String, 
  last_name : String,
  preferred_name : String,
  point_record : Option<PointRecord>,
  discord_id : Option<String>
}

#[derive(Serialize, Deserialize,Debug,PartialEq,Clone)]
struct PointRecord {
    points: i64,
    gems : i64
}


fn get_dynamo_client() -> DynamoDbClient {
    DynamoDbClient::new(
        match env::var("LOCAL_DYNAMODB") {
            Ok(_) => 
                Region::Custom {
                    name: String::from("local-region"),
                    endpoint: String::from("http://localhost:8000")
                },
            Err(_) => 
               Region::UsEast1 
        }
    )
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    pub struct LocalTpcDataStore  { users : Vec<User> }

    #[async_trait]
    impl TpcDatastore for LocalTpcDataStore {
      async fn add_member(&mut self, user: &User) -> Result<(), String> {
          self.users.push(user.clone());
          Ok(())
      }
      async fn delete_member(&mut self, student_number: &str) -> Result<(), String> {
          self.users = self.users.iter().filter(|user| user.student_number != student_number).map(|x| x.clone()).collect();
          Ok(())
      }
      async fn list_members(&self) -> Result<Vec<User>, String> {
          Ok(self.users.clone())
      }
    }

    impl Default for LocalTpcDataStore {
       fn default() -> LocalTpcDataStore {
         LocalTpcDataStore { users : vec![] }
       }
    }

    fn test_user() -> User {
       User { id: String::from("")
            , student_number: String::from("s3723315")
            , first_name: String::from("Sam")
            , last_name: String::from("Nolan")
            , preferred_name: String::from("")
            , preferred_email: String::from("")
            , discord_id: None
            , point_record: None
            , show_user: true
            }
    }

    #[test]
    fn test_serialise() {
        let user = test_user();
        assert_eq!(user, serde_dynamodb::from_hashmap(serde_dynamodb::to_hashmap(&user).unwrap()).unwrap());
    }

    async fn add_user(user: User, datastore: &mut impl TpcDatastore) {
        let request = lambda_http::http::Request::builder().uri(String::from("https://6ebcvdz1y3.execute-api.us-east-1.amazonaws.com/register"))
                               .body(Body::Text(serde_json::to_string(&user).unwrap())).unwrap();
                       
        main_handler(datastore, request, Context::default()).await.unwrap();
    }

    async fn list_users(datastore: &mut impl TpcDatastore) -> Vec<User> {
        let list_request = lambda_http::http::Request::builder().uri(String::from("https://6ebcvdz1y3.execute-api.us-east-1.amazonaws.com/members"))
                               .body(Body::Empty).unwrap();

        let list_result = main_handler(datastore, list_request, Context::default()).await.unwrap();
        match list_result.body() {
            Body::Text(response_text) =>  {
                let users : Vec<User> = serde_json::from_str(&response_text).unwrap();
                users
            }
            _ => {
                assert!(false);
                vec![]
            }
        }
    }

    #[tokio::test]
    async fn test_register() {
        let mut datastore = LocalTpcDataStore::default();
        let user = test_user();
        add_user(user, &mut datastore).await;
        assert_eq!(list_users(&mut datastore).await.len(), 1)
    }

    #[tokio::test]
    async fn test_register_no_show() {
        let mut datastore = LocalTpcDataStore::default();
        let mut user = test_user();
        user.show_user = false;
        add_user(user, &mut datastore).await;

        assert_eq!(list_users(&mut datastore).await, vec![])
    }
}

