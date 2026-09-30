//! Spring, drag and throw motion a host or a consumer's own component drives (design/06-INTERACTIONS.md).

pub use ds_motion::{
    drag::fraction_along,
    drag_return::{DragReturn, use_drag_return},
    projection::Throw,
    spring::{Ratio, SpringPhase},
    spring_point::{
        PointThrow, Release, SpringPointMotion, use_spring_point, use_spring_point_motion,
    },
    spring_spec::{SpringResponse, SpringSpec},
    timeline::spring::PxPerUnit,
    use_spring::use_spring,
    velocity::{Velocity, VelocityMeter},
};
