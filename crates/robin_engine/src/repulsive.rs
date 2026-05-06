//! Repulsive objects used by actor-vs-actor anti-collision.
//!
//! These are the "personal space" markers each actor contributes to
//! the anti-collision system — when another actor is about to step
//! onto them, `PositionInterface::update_position_anti_collision`
//! (see [`crate::position_interface`]) deviates the moving actor
//! around them.
//!
//! The heavy deviation math already lives in [`crate::rhline`] as pure
//! functions — the structs here are just data holders that wire the
//! right members into those functions.

use serde::{Deserialize, Serialize};

use crate::geo2d::{self, Point2D, Vec2D};
use crate::rhline;

/// Repulsive point — a circular (or wedge-limited) zone of influence
/// centred on `position`.
///
/// `radius`, `action_radius`, `force_a`, `force_b` come from
/// `SetForce` — see [`rhline::repulsive_set_force`].  The stored
/// `action_radius` **includes** `radius`
/// (`action_radius = radius + action_radius_input`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RepulsivePoint {
    pub position: Point2D,
    pub radius: f32,
    /// Total action radius: `radius + action_radius_input`.
    pub action_radius: f32,
    pub force_a: f32,
    pub force_b: f32,

    // Action field. Default: total circle.
    pub is_total: bool,
    pub is_concave: bool,
    pub limit_left: Vec2D,
    pub limit_right: Vec2D,
}

impl RepulsivePoint {
    /// Build a `RepulsivePoint` with the given `SetForce` parameters.
    /// `radius` is the inner (hard) radius; `action_radius_input` is the
    /// soft falloff distance beyond it.
    pub fn new(position: Point2D, radius: f32, action_radius_input: f32) -> Self {
        let (ar, r, fa, fb) = rhline::repulsive_set_force(radius, action_radius_input);
        Self {
            position,
            radius: r,
            action_radius: ar,
            force_a: fa,
            force_b: fb,
            is_total: true,
            is_concave: false,
            limit_left: geo2d::pt(0.0, 0.0),
            limit_right: geo2d::pt(0.0, 0.0),
        }
    }

    /// Restrict the action field to an angular wedge.
    pub fn set_action_field(&mut self, limit_left: Vec2D, limit_right: Vec2D) {
        self.is_total = false;
        self.limit_left = limit_left;
        self.limit_right = limit_right;
        self.is_concave = geo2d::cross(limit_left, limit_right) < 0.0;
    }

    /// Returns `Some(distance_destination)` if the future position
    /// `destination` is close enough to warrant a deviation check.
    pub fn is_deviating(&self, destination: Point2D) -> Option<f32> {
        let rel = geo2d::pt(
            destination.x - self.position.x,
            destination.y - self.position.y,
        );
        let distance = geo2d::length(rel);
        if distance > self.action_radius {
            return None;
        }
        if !self.is_total {
            let left = geo2d::cross(self.limit_left, rel);
            let right = geo2d::cross(self.limit_right, rel);
            if self.is_concave {
                if left < 0.0 && right >= 0.0 {
                    return None;
                }
            } else if left < 0.0 || right >= 0.0 {
                return None;
            }
        }
        Some(distance)
    }

    /// Compute the deviated movement around this point.  Returns
    /// `Some(new_movement)` when the actor should deviate; `None`
    /// means "too far" (continue straight).
    pub fn compute_deviation(
        &self,
        movement: Vec2D,
        origin: Point2D,
        movement_mag: f32,
        distance_destination: f32,
        actor_radius: f32,
    ) -> Option<Vec2D> {
        let r = rhline::repulsive_point_compute_deviation(
            rhline::Vec2::new(movement.x, movement.y),
            rhline::Vec2::new(origin.x, origin.y),
            movement_mag,
            distance_destination,
            actor_radius,
            rhline::Vec2::new(self.position.x, self.position.y),
            self.radius,
            // The pure-math helper expects the *input* action radius
            // (the falloff distance beyond the inner radius), which is
            // `action_radius - radius` since our stored action_radius
            // already includes the inner radius.
            self.action_radius - self.radius,
            self.force_a,
            self.force_b,
        )?;
        Some(geo2d::pt(r.x, r.y))
    }
}

/// Repulsive line segment — a directed line with an outward normal
/// and a repulsion zone extending perpendicular to it.
///
/// `action_radius` **includes** `radius` (same convention as
/// [`RepulsivePoint`]).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RepulsiveLine {
    pub a: Point2D,
    pub b: Point2D,
    pub normal: Vec2D,
    pub vector: Vec2D,
    pub radius: f32,
    /// Total action radius: `radius + action_radius_input`.
    pub action_radius: f32,
    pub force_a: f32,
    pub force_b: f32,
    /// `POINT_TOTAL` flag — when false, only positive-normal side deflects.
    pub is_total: bool,
    /// True when the segment is an "area" (two-sided repulsion); selects
    /// the direct-sense normal.
    pub is_area: bool,
}

impl RepulsiveLine {
    /// Build a `RepulsiveLine` from endpoints and `SetForce` parameters.
    pub fn new(a: Point2D, b: Point2D, radius: f32, action_radius_input: f32) -> Self {
        let (ar, r, fa, fb) = rhline::repulsive_set_force(radius, action_radius_input);
        let dx = b.x - a.x;
        let dy = b.y - a.y;
        let len = (dx * dx + dy * dy).sqrt();
        let (vx, vy) = if len > 0.0 {
            (dx / len, dy / len)
        } else {
            (0.0, 0.0)
        };
        // Default: non-area line uses `get_normal(false)` = (y, -x).
        let nx = vy;
        let ny = -vx;
        Self {
            a,
            b,
            normal: geo2d::pt(nx, ny),
            vector: geo2d::pt(vx, vy),
            radius: r,
            action_radius: ar,
            force_a: fa,
            force_b: fb,
            is_total: true,
            is_area: false,
        }
    }

    /// Flip the `is_area` flag and recompute the normal accordingly.
    pub fn set_area(&mut self, is_area: bool) {
        self.is_area = is_area;
        if is_area {
            // `get_normal()` default = true → (-y, x)
            self.normal = geo2d::pt(-self.vector.y, self.vector.x);
        } else {
            self.normal = geo2d::pt(self.vector.y, -self.vector.x);
        }
    }

    /// True when the `destination` lies between the segment endpoints
    /// along the segment's projection axis.  Delegates to
    /// [`geo2d::point_in_segment_slab`].
    fn is_between(&self, destination: Point2D) -> bool {
        geo2d::point_in_segment_slab(destination, geo2d::Segment2D::new(self.a, self.b))
    }

    /// Returns `Some(signed_distance_destination)` if the future
    /// position `destination` is close enough to warrant a deviation
    /// check.
    pub fn is_deviating(&self, destination: Point2D) -> Option<f32> {
        let rel = geo2d::pt(destination.x - self.a.x, destination.y - self.a.y);
        let distance = rel.x * self.normal.x + rel.y * self.normal.y;
        if !self.is_total && distance < 0.0 {
            return None;
        }
        if distance.abs() < self.action_radius && self.is_between(destination) {
            Some(distance)
        } else {
            None
        }
    }

    /// Compute the deviated movement around this line.
    pub fn compute_deviation(
        &self,
        movement: Vec2D,
        origin: Point2D,
        movement_mag: f32,
        distance_destination: f32,
        actor_radius: f32,
    ) -> Option<Vec2D> {
        let r = rhline::repulsive_line_compute_deviation(
            rhline::Vec2::new(movement.x, movement.y),
            rhline::Vec2::new(origin.x, origin.y),
            movement_mag,
            distance_destination,
            actor_radius,
            self.radius,
            self.action_radius - self.radius,
            self.force_a,
            self.force_b,
            rhline::Vec2::new(self.normal.x, self.normal.y),
            rhline::Vec2::new(self.vector.x, self.vector.y),
            rhline::Vec2::new(self.a.x, self.a.y),
        )?;
        Some(geo2d::pt(r.x, r.y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_is_deviating_total() {
        let p = RepulsivePoint::new(geo2d::pt(0.0, 0.0), 4.0, 12.0);
        // Inside action radius (16 total) → Some
        assert!(p.is_deviating(geo2d::pt(5.0, 0.0)).is_some());
        // Outside action radius → None
        assert!(p.is_deviating(geo2d::pt(50.0, 0.0)).is_none());
    }

    #[test]
    fn point_new_stores_total_action_radius() {
        let p = RepulsivePoint::new(geo2d::pt(0.0, 0.0), 4.0, 12.0);
        assert!((p.radius - 4.0).abs() < 1e-6);
        assert!((p.action_radius - 16.0).abs() < 1e-6);
    }

    #[test]
    fn line_is_deviating_between() {
        let l = RepulsiveLine::new(geo2d::pt(0.0, 0.0), geo2d::pt(10.0, 0.0), 2.0, 5.0);
        // Point near midpoint, on +normal side → Some
        assert!(l.is_deviating(geo2d::pt(5.0, 3.0)).is_some());
        // Point past the segment endpoints → None
        assert!(l.is_deviating(geo2d::pt(20.0, 3.0)).is_none());
    }
}
