use futures::future::{Future, FutureExt};
use http_service::{Request, Response};
use crate::endpoint::AsyncResponse;
use std::{error::Error, pin::Pin};

pub(crate) type RouteFn = Box<Fn(Request) -> AsyncResponse>;

pub struct Route {
    pub path: String,
    pub static_segments: Vec<bool>,
    pub dynamic_segments: Vec<bool>,
    pub handler: RouteFn,
}

impl Route {
    pub fn new(path: impl Into<String>, handler: RouteFn) -> Self {
        let path = path.into();

        let (static_segments, dynamic_segments) = parse_segments(&path);

        Self {
            path,
            static_segments,
            dynamic_segments,
            handler,
        }
    }
}

fn parse_segments(path: &str) -> (Vec<bool>, Vec<bool>) {
    let mut static_segments = vec![];
    let mut dynamic_segments = vec![];

    let _ = path.split("/").map(|s| {
        if s.starts_with(":") {
            dynamic_segments.push(true);
            static_segments.push(false);
        } else {
            dynamic_segments.push(false);
            static_segments.push(true);
        }
    });

    (static_segments, dynamic_segments)
}

pub fn router(routes: Vec<Route>) -> impl Fn(Request) -> AsyncResponse {
    let state_machine = routes.into_iter().fold(NFA, |nfa, route| {
        let mut index = 0;
        route.path.split("/").enumerate().for_each(|(i, segment)| {
            index = i;

            if route.static_segments[i] {
                nfa.insert_static_segment(i, segment);
            } else if route.dynamic_segments[i] {
                nfa.insert_dynamic_segment(i, segment[1..]);
            }
        });

        nfa.set_final_state(index, route.handler);
    });

    move |req: Request| {
        let fut = async move {

        };

        fut.boxed()
    }
}

struct State {
    next_states: Vec<usize>,
    handler: Option<RouteFn>,
    segment: String,
}

struct NFA {
    static_segment: Vec<bool>,
    dynamic_segment: Vec<bool>,
    states: Vec<State>,
}
