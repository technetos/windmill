use crate::params::Params;
use serde::Deserialize;

/// Additional features provided through `enzyme` can be accessed through methods and trait
/// implementations on `Req`.  
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
    pub fn new(req: http_types::Request, body: Option<Body>, params: Params) -> Self {
        Self { req, body, params }
    }

    pub fn body(&self) -> Option<&Body> {
        self.body.as_ref()
    }

    pub fn params(&self) -> &Params {
        &self.params
    }
}
