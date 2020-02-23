
use crate::params::Params;
use serde::Deserialize;

/// A request type encapsulating `http-types::Request`, url parameters and the deserialized body.   
pub struct Req<Body>
where
    Body: for<'de> Deserialize<'de> + 'static + Send,
{
    req: http_types::Request,
    body: Option<Body>,
    params: Params,
}

impl<Body> std::ops::Deref for Req<Body>
where
    Body: for<'de> Deserialize<'de> + 'static + Send,
{
    type Target = http_types::Request;

    fn deref(&self) -> &Self::Target {
        &self.req
    }
}

impl<Body> std::ops::DerefMut for Req<Body>
where
    Body: for<'de> Deserialize<'de> + 'static + Send,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.req
    }
}

impl<Body> Req<Body>
where
    Body: for<'de> Deserialize<'de> + 'static + Send,
{
    pub(crate) fn new(req: http_types::Request, body: Option<Body>, params: Params) -> Self {
        Self { req, body, params }
    }

    /// Access the body of the request.  
    /// ```
    /// let body = req.body().ok_or_else(|| Error {
    ///     code: StatusCode::BadRequest,
    ///     msg: json!("body required"),
    /// })?;
    /// ```
    pub fn body(&self) -> Option<&Body> {
        self.body.as_ref()
    }

    /// Access the parameters of the path.  
    /// ```
    /// let param = req.params().get("foo").ok_or_else(|| Error {
    ///   code: StatusCode::InternalServerError,
    ///   msg: json!("param foo does not exist"),
    /// })?;
    /// ```
    pub fn params(&self) -> &Params {
        &self.params
    }
}
