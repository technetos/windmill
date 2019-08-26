#![feature(async_closure)]

mod endpoint;
mod error;

use error::WebError;
use serde_json::json;

#[test]
fn test() {
    use crate::endpoint::Endpoint;

    mod messages {
        use serde::{Deserialize, Serialize};

        #[derive(Deserialize)]
        pub struct TestRequest {
            pub user_name: String,
        }

        #[derive(Serialize)]
        pub struct TestResponse;

        #[derive(Deserialize)]
        pub struct LogoutRequest {
            pub access_token: String,
        }

        #[derive(Serialize)]
        pub struct LogoutResponse {
            pub success: bool,
        }
    }

    struct TestEndpoint;

    use messages::{LogoutRequest, LogoutResponse, TestRequest, TestResponse};

    impl TestEndpoint {
        pub async fn get_token(req: TestRequest) -> Result<TestResponse, WebError> {
            Err(WebError {
                code: http::StatusCode::UNAUTHORIZED,
                msg: json!("User is unauthorized to access this resource"),
            })
        }

        pub async fn logout(req: LogoutRequest) -> Result<LogoutResponse, WebError> {
            Ok(LogoutResponse { success: true })
        }
    }

    let test_service = Endpoint::new(TestEndpoint::get_token);
    let logout_service = Endpoint::new(TestEndpoint::logout);

    let test_req = http_service::Request::new("foo".into());
}
