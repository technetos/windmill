pub async fn read_body(req: &mut http_types::Request) -> Vec<u8> {
    use async_std::prelude::*;

    let has_body = req
        .header(&http_types::headers::CONTENT_LENGTH)
        .map(|header_values| header_values.first().map(|value| value.as_str() != "0"))
        .flatten()
        .unwrap_or_else(|| false);

    let mut body = vec![];
    if has_body {
        let _ = req.read_to_end(&mut body).await;
    }
    body
}
