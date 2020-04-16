use crate::{context::Context, params::Params};

pub struct State {
    req: http_types::Request,
    params: Params,
    context: Context,
}
