
use crate::params::Params;
use serde::Deserialize;

/// A request type encapsulating `http-types::Request`.  
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
    /// Create a new instance of `Req`.  
    pub fn new(req: http_types::Request, body: Option<Body>, params: Params) -> Self {
        Self { req, body, params }
    }

    /// Access the body of the request.  
    pub fn body(&self) -> Option<&Body> {
        self.body.as_ref()
    }

    /// Access the parameters of the path.  
    pub fn params(&self) -> &Params {
        &self.params
    }
}
