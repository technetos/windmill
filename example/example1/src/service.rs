use enzyme::*;
use http_types::{mime, Mime, StatusCode};
use serde::{Deserialize, Serialize};

pub async fn service<Body, Res>(
    mut req: http_types::Request,
    params: Params,
    endpoint: impl Endpoint<Body, Res>,
) -> http_types::Response
where
    Body: for<'de> Deserialize<'de>,
    Res: Serialize,
{
    let body = serde_json::from_slice(&read_body(&mut req).await).unwrap_or_else(|_| None);

    let maybe_bytes = endpoint
        .call(Req::new(req, body, params))
        .await
        .map(|body| serde_json::to_vec(&body));

    match maybe_bytes {
        Ok(Ok(bytes)) => {
            let mut res = response(StatusCode::Ok, mime::JSON);
            res.set_body(bytes);
            res
        }
        Ok(Err(e)) => {
            let mut res = response(StatusCode::InternalServerError, mime::JSON);
            res.set_body(serde_json::to_vec(&format!("{}", e)).unwrap());
            res
        }
        Err(e) => {
            let mut res = response(e.code(), mime::JSON);
            res.set_body(serde_json::to_vec(e.msg()).unwrap());
            res
        }
    }
}

pub async fn auth_service<Body, Res>(
    req: http_types::Request,
    params: Params,
    endpoint: impl Endpoint<Body, Res>,
) -> http_types::Response
where
    Body: for<'de> Deserialize<'de>,
    Res: Serialize,
{
    use std::str::FromStr;
    if req.header(&http_types::headers::HeaderName::from_str("authorization").unwrap()).is_none() {
        return http_types::Response::new(StatusCode::BadRequest);
    }
    service(req, params, endpoint).await    
}

fn response(code: StatusCode, mime: Mime) -> http_types::Response {
    let mut res = http_types::Response::new(code);
    let _ = res.set_content_type(mime);
    res
}
