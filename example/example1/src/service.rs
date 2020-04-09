use http_types::{headers::HeaderName, mime, Mime, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use windmill::*;

pub async fn service<Body, Res>(
    mut req: http_types::Request,
    params: Params,
    endpoint: impl Endpoint<Body, Res>,
) -> Result<http_types::Response, Error>
where
    Body: for<'de> Deserialize<'de>,
    Res: Serialize,
{
    let body = serde_json::from_slice(&read_body(&mut req).await).unwrap_or_else(|_| None);
    let res_body = endpoint.call(Req::new(req, body, params)).await?;

    let bytes = serde_json::to_vec(&res_body).map_err(|e| Error {
        code: StatusCode::InternalServerError,
        msg: json!(&format!("{}", e)),
    })?;

    let mut res = response(StatusCode::Ok, mime::JSON);
    res.set_body(bytes);
    Ok(res)
}

pub async fn auth_service<Body, Res>(
    req: http_types::Request,
    params: Params,
    endpoint: impl Endpoint<Body, Res>,
) -> Result<http_types::Response, Error>
where
    Body: for<'de> Deserialize<'de>,
    Res: Serialize,
{
    use std::str::FromStr;
    let header_name = HeaderName::from_str("authorization").map_err(|header_name| Error {
        code: StatusCode::InternalServerError,
        msg: json!("bad header name"),
    })?;

    let header = req.header(&header_name).ok_or_else(|| Error {
        code: StatusCode::BadRequest,
        msg: json!("authorization required"),
    })?;

    service(req, params, endpoint).await
}

fn response(code: StatusCode, mime: Mime) -> http_types::Response {
    let mut res = http_types::Response::new(code);
    let _ = res.set_content_type(mime);
    res
}
