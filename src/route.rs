use crate::params::Params;
use std::{pin::Pin, future::Future};

pub(crate) type ResponseFuture = Pin<Box<dyn Future<Output = http_types::Response> + Send + Sync>>;
type RouteFn = Box<dyn Fn(http_types::Request, Params) -> ResponseFuture + Send + Sync>;

/// A route constructed using the [`route!`](macro.route.html) macro.  
pub struct Route {
    pub static_segments: Vec<StaticSegment>,
    pub dynamic_segments: Vec<DynamicSegment>,
    pub handler: Option<RouteFn>,
}

#[doc(hidden)]
pub struct StaticSegment {
    pub value: &'static str,
    pub position: usize,
}

#[doc(hidden)]
pub struct DynamicSegment {
    pub name: &'static str,
    pub position: usize,
}

pub(crate) struct RawSegment<'s> {
    pub(crate) value: &'s str,
    position: usize,
}

pub(crate) struct RawRoute<'s> {
    pub raw_segments: Vec<RawSegment<'s>>,
}

impl<'s> RawRoute<'s> {
    pub(crate) fn from_path(path: &'s str) -> Self {
        Self {
            raw_segments: path
                .split("/")
                .skip(1)
                .enumerate()
                .map(|(i, segment)| RawSegment {
                    value: segment,
                    position: i,
                })
                .collect(),
        }
    }
}

impl<'s> PartialEq<RawSegment<'s>> for StaticSegment {
    fn eq(&self, other: &RawSegment) -> bool {
        self.position == other.position && self.value == other.value
    }
}

impl<'s> PartialEq<RawSegment<'s>> for DynamicSegment {
    fn eq(&self, other: &RawSegment) -> bool {
        self.position == other.position
    }
}

impl<'s> PartialEq<StaticSegment> for RawSegment<'s> {
    fn eq(&self, other: &StaticSegment) -> bool {
        other == self
    }
}

impl<'s> PartialEq<DynamicSegment> for RawSegment<'s> {
    fn eq(&self, other: &DynamicSegment) -> bool {
        other == self
    }
}
