#![feature(async_closure)]

mod endpoint;
mod error;

pub mod prelude {
    pub type WebResult<T> = Result<T, WebError>;

    pub use super::endpoint::Endpoint;
    pub use super::error::WebError;
}

#[test]
fn test() {
    mod messages {
        use serde::{Deserialize, Serialize};

        #[derive(Deserialize)]
        pub struct LoginRequest {
            pub user_name: String,
            pub password: String,
        }

        #[derive(Serialize)]
        pub struct LoginResponse;

        #[derive(Deserialize)]
        pub struct LogoutRequest {
            pub access_token: String,
        }

        #[derive(Serialize)]
        pub struct LogoutResponse {
            pub success: bool,
        }
    }

    mod policy {
        use crate::WebResult;
        use http::request::Parts;

        #[derive(Debug)]
        pub struct Policy {
            pub token: String,
        }

        impl Policy {
            pub async fn new(parts: Parts) -> WebResult<Policy> {
                let token = String::from("foo");

                Ok(Policy { token })
            }
        }
    }

    use crate::endpoint::Endpoint;
    use serde_json::json;

    struct TestEndpoint;

    use messages::{LoginRequest, LoginResponse, LogoutRequest, LogoutResponse};
    use policy::Policy;

    impl TestEndpoint {
        pub async fn get_token(req: LoginRequest) -> WebResult<LoginResponse> {
            Ok(LoginResponse)
        }

        pub async fn logout(policy: Policy, req: LogoutRequest) -> WebResult<LogoutResponse> {
            if &policy.token != "foo" {
                return Err(WebError {
                    code: http::StatusCode::UNAUTHORIZED,
                    msg: json!("User is unauthorized to access this resource"),
                });
            }

            Ok(LogoutResponse { success: true })
        }
    }

    let login_service = Endpoint::new(TestEndpoint::get_token);
    let logout_service = Endpoint::new_with_policy(TestEndpoint::logout, Policy::new);

    let test_req = http_service::Request::new("foo".into());
}
