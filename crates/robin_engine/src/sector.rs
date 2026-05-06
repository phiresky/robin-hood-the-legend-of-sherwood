//! Sector system — map regions with type flags, geometry, and specialized behavior.
//!
//! Uses composition: [`Sector`] holds the common polygon + metadata, and
//! [`SectorKind`] carries variant-specific data.

use bitflags::bitflags;
use serde::{Deserialize, Serialize};

use crate::gate::ActorAuthInfo;
use crate::geo2d::{BBox2D, Point2D};
use crate::sector_production;
use crate::sound_cache::Material;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// BuildingIdx — nominal newtype
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Index into the engine's building table.  Wraps [`nonmax::NonMaxU16`]
/// so `Option<BuildingIdx>` is 2 bytes via niche optimization.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    robin_state_hash_derive::StateHash,
)]
pub struct BuildingIdx(pub nonmax::NonMaxU16);

impl BuildingIdx {
    #[inline]
    pub fn new(v: u16) -> Option<Self> {
        nonmax::NonMaxU16::new(v).map(Self)
    }
    #[inline]
    pub fn get(self) -> u16 {
        self.0.get()
    }
}
impl From<BuildingIdx> for u16 {
    #[inline]
    fn from(i: BuildingIdx) -> u16 {
        i.0.get()
    }
}
impl From<BuildingIdx> for u32 {
    #[inline]
    fn from(i: BuildingIdx) -> u32 {
        u32::from(i.0.get())
    }
}
impl From<BuildingIdx> for usize {
    #[inline]
    fn from(i: BuildingIdx) -> usize {
        usize::from(i.0.get())
    }
}
impl std::fmt::Display for BuildingIdx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.get().fmt(f)
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// SectorNumber — nominal newtype
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Canonical sector identifier.
///
/// Stored as signed i16 on sectors and reinterpreted as u16 on door
/// records; both refer to the same id space.  Negative values are used
/// as "invalid" sentinels (`sector == -1` ⇒ "not found"), so we keep the
/// i16 representation and surface it with `From<SectorNumber> for u16` via
/// `as u16` (sign-preserving reinterpretation at door-comparison sites).
///
/// Distinct from [`crate::fast_find_grid::SectorIndex`] which is the
/// slot index into `FastFindGrid::level::sectors`.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    robin_state_hash_derive::StateHash,
)]
pub struct SectorNumber(pub i16);

impl SectorNumber {
    /// Wrap an i16 as a sector number.
    #[inline]
    pub const fn new(v: i16) -> Self {
        Self(v)
    }
    /// Raw signed value.
    #[inline]
    pub const fn get(self) -> i16 {
        self.0
    }
    /// True when the value is a valid (non-negative) id.  Negative
    /// sector numbers (especially `-1`) are used as "invalid" sentinels.
    #[inline]
    pub const fn is_valid(self) -> bool {
        self.0 >= 0
    }
}
impl From<i16> for SectorNumber {
    #[inline]
    fn from(v: i16) -> Self {
        Self(v)
    }
}
impl From<SectorNumber> for i16 {
    #[inline]
    fn from(n: SectorNumber) -> i16 {
        n.0
    }
}
impl From<SectorNumber> for u16 {
    #[inline]
    fn from(n: SectorNumber) -> u16 {
        // Sign-preserving reinterpretation at door-comparison sites
        // (`d.sector_in == sector_number as u16`).
        n.0 as u16
    }
}
impl PartialEq<i16> for SectorNumber {
    #[inline]
    fn eq(&self, other: &i16) -> bool {
        self.0 == *other
    }
}
impl PartialEq<SectorNumber> for i16 {
    #[inline]
    fn eq(&self, other: &SectorNumber) -> bool {
        *self == other.0
    }
}
impl PartialEq<u16> for SectorNumber {
    #[inline]
    fn eq(&self, other: &u16) -> bool {
        // Compare as bit patterns (door-site convention).
        (self.0 as u16) == *other
    }
}
impl PartialEq<SectorNumber> for u16 {
    #[inline]
    fn eq(&self, other: &SectorNumber) -> bool {
        *self == (other.0 as u16)
    }
}
// Handy for tests / i32 literals.  Compares by widening to i32 (the
// SectorNumber fits losslessly).
impl PartialEq<i32> for SectorNumber {
    #[inline]
    fn eq(&self, other: &i32) -> bool {
        i32::from(self.0) == *other
    }
}
impl PartialEq<SectorNumber> for i32 {
    #[inline]
    fn eq(&self, other: &SectorNumber) -> bool {
        *self == i32::from(other.0)
    }
}
// Increment by i16 literal — useful for the level-loading pass that
// walks motion areas and doors sequentially.
impl std::ops::AddAssign<i16> for SectorNumber {
    #[inline]
    fn add_assign(&mut self, rhs: i16) {
        self.0 += rhs;
    }
}
impl std::ops::Add<i16> for SectorNumber {
    type Output = SectorNumber;
    #[inline]
    fn add(self, rhs: i16) -> Self {
        Self(self.0 + rhs)
    }
}
impl std::fmt::Display for SectorNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// SectorType bitflags
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct SectorType: u32 {
        const AREA       = 1;
        const MOTION     = 2;
        const PATCH      = 4;
        const SCRIPT     = 8;
        const SOUND      = 16;
        const PLANE      = 32;
        const MOUSE      = 64;
        const CROSS      = 128;
        const APPLY      = 256;
        const LIFT       = 512;
        const ASSOCIATED = 1024;
        const SURROUND   = 2048;
        const DOOR       = 4096;
        const BUILDING   = 8192;
        const SHADOW     = 16384;
        const JUMP       = 32768;
        const RAILROAD   = 65536;
        const APEX       = 131072;
    }
}

impl SectorType {
    pub fn is_area(self) -> bool {
        self.contains(Self::AREA)
    }
    /// An obstacle is any sector without the AREA flag.
    pub fn is_obstacle(self) -> bool {
        !self.contains(Self::AREA)
    }
    pub fn is_motion(self) -> bool {
        self.contains(Self::MOTION)
    }
    pub fn is_plane(self) -> bool {
        self.contains(Self::PLANE)
    }
    pub fn is_sound(self) -> bool {
        self.contains(Self::SOUND)
    }
    pub fn is_script(self) -> bool {
        self.contains(Self::SCRIPT)
    }
    pub fn is_patch(self) -> bool {
        self.contains(Self::PATCH)
    }
    pub fn is_mouse(self) -> bool {
        self.contains(Self::MOUSE)
    }
    pub fn is_cross(self) -> bool {
        self.contains(Self::CROSS)
    }
    pub fn is_apply(self) -> bool {
        self.contains(Self::APPLY)
    }
    pub fn is_lift(self) -> bool {
        self.contains(Self::LIFT)
    }
    pub fn is_building(self) -> bool {
        self.contains(Self::BUILDING)
    }
    pub fn is_associated(self) -> bool {
        self.contains(Self::ASSOCIATED)
    }
    pub fn is_door(self) -> bool {
        self.contains(Self::DOOR)
    }
    pub fn is_shadow(self) -> bool {
        self.contains(Self::SHADOW)
    }
    pub fn is_railroad(self) -> bool {
        self.contains(Self::RAILROAD)
    }
    pub fn is_apex(self) -> bool {
        self.contains(Self::APEX)
    }
    pub fn is_jump(self) -> bool {
        self.contains(Self::JUMP)
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Small enums
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Lift sub-type.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    Default,
    robin_state_hash_derive::StateHash,
)]
pub enum LiftType {
    #[default]
    Normal,
    Stairs,
    Ladder,
    Wall,
}

impl LiftType {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::Normal,
            1 => Self::Stairs,
            2 => Self::Ladder,
            3 => Self::Wall,
            _ => panic!("unknown lift type: {v}"),
        }
    }

    /// Wall and ladder lifts restrict who can traverse them.
    pub fn is_wall_or_ladder(self) -> bool {
        matches!(self, Self::Wall | Self::Ladder)
    }

    /// Lift translation for an actor in upright posture moving inside this
    /// lift sector.  Upright posture has upwards == downwards, so the
    /// upwards mapping is used unconditionally.
    pub fn translate_upright_action(
        self,
        action: crate::order::OrderType,
    ) -> crate::order::OrderType {
        // Upright posture has upwards == downwards; pick the upwards mapping.
        self.translate_climb_action(action, false)
    }

    /// Lift translation for an actor on a ladder or wall (`Posture::OnLadder`
    /// / `Posture::OnWall`).  When `going_down` is true the actor is moving
    /// in the `low - high` direction and gets the downwards animation;
    /// otherwise the upwards animation.  Determined by dot-producting the
    /// ladder vector with the movement vector to pick downwards vs upwards.
    ///
    /// Stairs / Normal lift types reach this only via the upright path
    /// (their up/down mappings are identical).
    pub fn translate_climb_action(
        self,
        action: crate::order::OrderType,
        going_down: bool,
    ) -> crate::order::OrderType {
        use crate::order::OrderType as A;
        match self {
            LiftType::Normal => {
                // Normal: up/down both return the same.
                if action == A::RunningUpright {
                    A::RunningStairs
                } else {
                    A::WalkingUpright
                }
            }
            LiftType::Stairs => {
                if action == A::RunningUpright {
                    action
                } else {
                    A::WalkingStairs
                }
            }
            LiftType::Ladder => {
                if going_down {
                    match action {
                        A::RunningUpright => A::ClimbingLadderDownFast,
                        A::WalkingAlerted => A::ClimbingLadderDownAlerted,
                        _ => A::ClimbingLadderDown,
                    }
                } else {
                    match action {
                        A::RunningUpright => A::ClimbingLadderUpFast,
                        A::WalkingAlerted => A::ClimbingLadderUpAlerted,
                        _ => A::ClimbingLadderUp,
                    }
                }
            }
            LiftType::Wall => {
                if going_down {
                    if action == A::RunningUpright {
                        A::ClimbingWallDownFast
                    } else {
                        A::ClimbingWallDown
                    }
                } else if action == A::RunningUpright {
                    A::ClimbingWallUpFast
                } else {
                    A::ClimbingWallUp
                }
            }
        }
    }
}

/// Classification of NPC occupants in a building sector.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, robin_state_hash_derive::StateHash,
)]
pub enum OccupantKind {
    Villains,
    Civilians,
    Empty,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Sector — common base data
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// A sector — a polygonal region of the game map.
///
/// All sector subtypes share these fields; variant-specific data lives in
/// [`SectorKind`].
#[derive(Debug, Clone, Serialize, Deserialize, robin_state_hash_derive::StateHash)]
pub struct Sector {
    // ── Polygon geometry loaded from proto stream ──
    /// The polygon vertices defining this sector's boundary.
    pub points: Vec<Point2D>,

    /// Axis-aligned bounding box of the polygon.
    pub bounding_box: BBox2D,

    // ── Type and identification (loaded from proto) ──
    /// Bitflag combination of sector type properties.
    pub sector_type: SectorType,

    /// Layer index this sector belongs to (`None` = unset).
    pub layer: Option<u16>,

    /// Global sector number (auto-incremented at construction).
    /// Stored in the legacy signed representation used by serialized
    /// sector state; grid-facing code wraps it in [`SectorNumber`] at
    /// ingestion so invalid sentinels stay out of pathing lookups.
    pub sector_number: i16,

    /// Obstacle number for pathfinding.
    pub obstacle_number: i16,

    /// Runtime state tracking counter.
    pub state_id: u32,

    // ── Active state ──
    // Serialized for door sectors; others are always `true`.
    pub active: bool,

    // ── Variant-specific data ──
    pub kind: SectorKind,
}

impl Sector {
    /// Create a new sector with the given type flags and no variant data.
    pub fn new(sector_type: SectorType) -> Self {
        Self {
            points: Vec::new(),
            bounding_box: BBox2D::new(),
            sector_type,
            layer: None,
            sector_number: 0,
            obstacle_number: 0,
            state_id: 0,
            active: true,
            kind: SectorKind::Generic,
        }
    }

    /// Create a sector with the specified kind and default type flags for that kind.
    pub fn with_kind(kind: SectorKind) -> Self {
        let sector_type = kind.default_sector_type();
        Self {
            points: Vec::new(),
            bounding_box: BBox2D::new(),
            sector_type,
            layer: None,
            sector_number: 0,
            obstacle_number: 0,
            state_id: 0,
            active: true,
            kind,
        }
    }

    // ── Type queries ──

    pub fn is_active(&self) -> bool {
        self.active
    }
    pub fn set_active(&mut self, state: bool) {
        self.active = state;
    }
    pub fn get_type(&self) -> SectorType {
        self.sector_type
    }
    pub fn is_of_type(&self, t: SectorType) -> bool {
        self.sector_type.contains(t)
    }
    pub fn is_area(&self) -> bool {
        self.sector_type.is_area()
    }
    pub fn is_obstacle(&self) -> bool {
        self.sector_type.is_obstacle()
    }
    pub fn is_motion(&self) -> bool {
        self.sector_type.is_motion()
    }
    pub fn is_building(&self) -> bool {
        self.sector_type.is_building()
    }
    pub fn is_lift(&self) -> bool {
        self.sector_type.is_lift()
    }
    pub fn is_door(&self) -> bool {
        self.sector_type.is_door()
    }
    pub fn is_shadow(&self) -> bool {
        self.sector_type.is_shadow()
    }

    // ── Geometry ──

    pub fn get_layer(&self) -> Option<u16> {
        self.layer
    }
    pub fn set_layer(&mut self, layer: Option<u16>) {
        self.layer = layer;
    }
    pub fn get_sector_number(&self) -> i16 {
        self.sector_number
    }
    pub fn set_sector_number(&mut self, n: i16) {
        self.sector_number = n;
    }
    pub fn num_points(&self) -> usize {
        self.points.len()
    }
    pub fn get_point(&self, index: usize) -> Point2D {
        self.points[index]
    }
    pub fn get_bounding_box(&self) -> &BBox2D {
        &self.bounding_box
    }

    /// Approximate area from bounding box.
    pub fn bbox_area(&self) -> f32 {
        if self.bounding_box.is_somewhere() {
            self.bounding_box.width() * self.bounding_box.height()
        } else {
            0.0
        }
    }

    /// Add a point to the polygon and expand the bounding box.
    pub fn add_point(&mut self, point: Point2D) {
        self.points.push(point);
        self.bounding_box.expand_point(point);
    }

    /// Ray-casting point-in-polygon test against this sector's vertices.
    /// Returns `false` for degenerate polygons (<3 points).
    pub fn polygon_contains(&self, p: Point2D) -> bool {
        let n = self.points.len();
        if n < 3 {
            return false;
        }
        let mut inside = false;
        let mut j = n - 1;
        for i in 0..n {
            let vi = self.points[i];
            let vj = self.points[j];
            if (vi.y > p.y) != (vj.y > p.y) {
                let x_intersect = (vj.x - vi.x) * (p.y - vi.y) / (vj.y - vi.y) + vi.x;
                if p.x < x_intersect {
                    inside = !inside;
                }
            }
            j = i;
        }
        inside
    }

    // ── Type flag manipulation ──

    pub fn set_type_flag(&mut self, flag: SectorType, state: bool) {
        if state {
            self.sector_type |= flag;
        } else {
            self.sector_type -= flag;
        }
    }

    pub fn add_type(&mut self, flag: SectorType) {
        self.sector_type |= flag;
    }

    // ── Kind access helpers ──

    pub fn as_motion_area(&self) -> Option<&MotionAreaData> {
        match &self.kind {
            SectorKind::MotionArea(d) => Some(d),
            SectorKind::Lift(d) => Some(&d.motion),
            SectorKind::Building(d) => Some(&d.motion),
            _ => None,
        }
    }

    pub fn as_motion_area_mut(&mut self) -> Option<&mut MotionAreaData> {
        match &mut self.kind {
            SectorKind::MotionArea(d) => Some(d),
            SectorKind::Lift(d) => Some(&mut d.motion),
            SectorKind::Building(d) => Some(&mut d.motion),
            _ => None,
        }
    }

    pub fn as_lift(&self) -> Option<&LiftData> {
        match &self.kind {
            SectorKind::Lift(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_lift_mut(&mut self) -> Option<&mut LiftData> {
        match &mut self.kind {
            SectorKind::Lift(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_building(&self) -> Option<&BuildingData> {
        match &self.kind {
            SectorKind::Building(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_building_mut(&mut self) -> Option<&mut BuildingData> {
        match &mut self.kind {
            SectorKind::Building(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_script(&self) -> Option<&ScriptSectorData> {
        match &self.kind {
            SectorKind::Script(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_script_mut(&mut self) -> Option<&mut ScriptSectorData> {
        match &mut self.kind {
            SectorKind::Script(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_archery(&self) -> Option<&ArcheryData> {
        match &self.kind {
            SectorKind::Archery(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_archery_mut(&mut self) -> Option<&mut ArcheryData> {
        match &mut self.kind {
            SectorKind::Archery(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_shadow(&self) -> Option<&ShadowData> {
        match &self.kind {
            SectorKind::Shadow(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_material(&self) -> Option<&MaterialData> {
        match &self.kind {
            SectorKind::Material(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_door_sector(&self) -> Option<&DoorSectorData> {
        match &self.kind {
            SectorKind::DoorSector(d) => Some(d),
            _ => None,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// SectorKind — variant-specific data
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Variant-specific sector data.
#[derive(Debug, Clone, Serialize, Deserialize, robin_state_hash_derive::StateHash)]
pub enum SectorKind {
    /// A plain sector with no specialized data.
    Generic,

    /// Walkable motion area.
    MotionArea(MotionAreaData),

    /// Lift / stairs / ladder / wall.
    Lift(Box<LiftData>),

    /// Building interior with occupant tracking.
    Building(Box<BuildingData>),

    /// Height plane for 3D projection.
    Plane(PlaneData),

    /// Patch sector linking to a terrain patch.
    Patch(PatchData),

    /// Associated sector — clickable overlay linked to another sector.
    Associated(AssociatedData),

    /// Door sector — clickable area around a door.
    DoorSector(DoorSectorData),

    /// Material/sound sector for footstep sounds and projectile bounces.
    Material(MaterialData),

    /// Script sector with trigger zone logic.
    Script(Box<ScriptSectorData>),

    /// Archery sector with guard patrol waypoints.
    Archery(Box<ArcheryData>),

    /// Shadow sector for ambient shadow rendering.
    Shadow(ShadowData),
}

impl SectorKind {
    /// Returns the default `SectorType` flags for this variant.
    pub fn default_sector_type(&self) -> SectorType {
        match self {
            Self::Generic => SectorType::empty(),
            Self::MotionArea(_) => SectorType::MOTION | SectorType::AREA,
            Self::Lift(_) => SectorType::MOTION | SectorType::AREA | SectorType::LIFT,
            Self::Building(_) => SectorType::MOTION | SectorType::AREA | SectorType::BUILDING,
            Self::Plane(_) => SectorType::PLANE,
            Self::Patch(_) => SectorType::PATCH,
            Self::Associated(_) => SectorType::ASSOCIATED,
            Self::DoorSector(_) => SectorType::DOOR | SectorType::MOUSE,
            Self::Material(_) => SectorType::SOUND | SectorType::CROSS,
            Self::Script(_) => SectorType::CROSS | SectorType::SCRIPT,
            Self::Archery(_) => SectorType::SURROUND,
            Self::Shadow(_) => SectorType::SHADOW,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// MotionAreaData
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Shared data for all walkable motion areas (base for Lift and Building too).
///
/// Gates, gate directions, projection areas, and jump lines are loaded from
/// the level proto stream and stored as indices into the global gate /
/// obstacle / jump-line tables.
#[derive(Debug, Clone, Default, Serialize, Deserialize, robin_state_hash_derive::StateHash)]
pub struct MotionAreaData {
    /// Index into the fast-find grid's area table.
    pub area_index: u16,

    /// Gate indices into the global gate table.
    pub gate_indices: Vec<crate::gate::DoorIndex>,

    /// For each gate, `true` if the gate's "in" direction points into this sector.
    pub gate_directions: Vec<bool>,

    /// Sight obstacle indices for projection areas.
    pub projection_area_indices: Vec<crate::sight_obstacle::SightObstacleIndex>,

    /// Jump line indices belonging to this sector.
    pub jump_line_indices: Vec<crate::jump_line::JumpLineIndex>,

    /// Whether all movement in this sector must be crouched.
    /// WARNING: Only set during level initialisation, not at runtime.
    pub force_crouched: bool,
}

impl MotionAreaData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_gate(&mut self, gate_index: crate::gate::DoorIndex, direction: bool) {
        self.gate_indices.push(gate_index);
        self.gate_directions.push(direction);
    }

    pub fn num_gates(&self) -> usize {
        self.gate_indices.len()
    }

    pub fn get_gate(&self, index: usize) -> (crate::gate::DoorIndex, bool) {
        (self.gate_indices[index], self.gate_directions[index])
    }

    pub fn add_projection_area(
        &mut self,
        obstacle_index: crate::sight_obstacle::SightObstacleIndex,
    ) {
        self.projection_area_indices.push(obstacle_index);
    }

    pub fn add_jump_line(&mut self, jump_line_index: crate::jump_line::JumpLineIndex) {
        self.jump_line_indices.push(jump_line_index);
    }

    pub fn num_jump_lines(&self) -> usize {
        self.jump_line_indices.len()
    }

    pub fn is_forcing_crouched(&self) -> bool {
        self.force_crouched
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// LiftData
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Lift / stairs / ladder / wall connecting different height layers.
#[derive(Debug, Clone, Serialize, Deserialize, robin_state_hash_derive::StateHash)]
pub struct LiftData {
    /// Inherited motion area data.
    pub motion: MotionAreaData,

    /// Sub-type of this lift.
    pub lift_type: LiftType,

    // ── Serialized (save-game state) ──
    /// Number of actors currently on the lift.
    pub occupants: u16,
    /// Number of PC actors on the lift.
    pub occupants_pc: u16,
    /// Whether a character is currently going upwards.
    pub occupied_upwards: bool,
    /// Whether a character is currently going downwards.
    pub occupied_downwards: bool,
    /// Cooldown timer preventing re-entry after use.
    pub wait_time: u32,

    // ── Loaded from proto ──
    /// Facing direction for the lift (0..15, ladder only).
    pub direction: i16,

    /// Bottom lift-side exit point, mirroring legacy implementation `RHSectorLift::GetLowExitPoint`.
    pub low_exit_point: Option<Point2D>,
    /// Top lift-side exit point, mirroring legacy implementation `RHSectorLift::GetHighExitPoint`.
    pub high_exit_point: Option<Point2D>,

    /// Ancillary index of the lowest door in the global gate table.
    pub lowest_door_index: Option<u32>,
    /// Ancillary index of the highest door in the global gate table.
    pub highest_door_index: Option<u32>,
}

impl Default for LiftData {
    fn default() -> Self {
        Self {
            motion: MotionAreaData::default(),
            lift_type: LiftType::Normal,
            occupants: 0,
            occupants_pc: 0,
            occupied_upwards: false,
            occupied_downwards: false,
            wait_time: 0,
            direction: 0,
            low_exit_point: None,
            high_exit_point: None,
            lowest_door_index: None,
            highest_door_index: None,
        }
    }
}

impl LiftData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_occupied(&self) -> bool {
        self.occupants != 0
    }

    pub fn is_occupied_by_pc(&self) -> bool {
        self.occupants_pc != 0
    }

    /// Check if downward traversal is currently allowed.
    /// Decrements the wait timer as a side effect.
    pub fn is_authorized_downwards(&mut self) -> bool {
        if self.wait_time != 0 {
            self.wait_time -= 1;
            return false;
        }
        self.occupants == 0 || self.occupied_downwards
    }

    /// Check if upward traversal is currently allowed.
    /// Decrements the wait timer as a side effect.
    pub fn is_authorized_upwards(&mut self) -> bool {
        if self.wait_time != 0 {
            self.wait_time -= 1;
            return false;
        }
        self.occupants == 0 || self.occupied_upwards
    }

    /// Mark an actor as entering the lift going downwards.
    ///
    /// `is_pc` should be `true` if the actor is a player character.
    pub fn set_occupied_downwards(&mut self, is_pc: bool, entering: bool) {
        if entering {
            self.occupants += 1;
            if is_pc {
                self.occupants_pc += 1;
            }
            self.occupied_downwards = true;
            self.wait_time = 100;
        } else {
            self.occupants = self.occupants.saturating_sub(1);
            if is_pc {
                self.occupants_pc = self.occupants_pc.saturating_sub(1);
            }
            if self.occupants == 0 {
                self.wait_time = 0;
                self.occupied_downwards = false;
                self.occupied_upwards = false;
            }
        }
    }

    /// Mark an actor as entering the lift going upwards.
    ///
    /// `is_pc` should be `true` if the actor is a player character.
    pub fn set_occupied_upwards(&mut self, is_pc: bool, entering: bool) {
        if entering {
            self.occupants += 1;
            if is_pc {
                self.occupants_pc += 1;
            }
            self.occupied_upwards = true;
            self.wait_time = 80;
        } else {
            self.occupants = self.occupants.saturating_sub(1);
            if is_pc {
                self.occupants_pc = self.occupants_pc.saturating_sub(1);
            }
            if self.occupants == 0 {
                self.wait_time = 0;
                self.occupied_downwards = false;
                self.occupied_upwards = false;
            }
        }
    }

    /// Check whether an actor is authorized to use this lift.
    ///
    /// - **Wall**: only PCs with the climb action.
    /// - **Ladder**: humans who are not civilians.
    /// - **Stairs**: all humans + objects (no animals ship in the game).
    /// - **Normal**: everyone.
    pub fn is_actor_authorized(&self, actor: &ActorAuthInfo) -> bool {
        match self.lift_type {
            LiftType::Wall => actor.kind.is_pc() && actor.has_climb,
            LiftType::Ladder => actor.kind.is_human() && !actor.kind.is_civilian(),
            LiftType::Stairs => true,
            LiftType::Normal => true,
        }
    }

    /// Pick the animation to use when traversing this lift upwards with
    /// the given base movement action.
    pub fn get_action_upwards(&self, action: crate::order::OrderType) -> crate::order::OrderType {
        use crate::order::OrderType as A;
        match self.lift_type {
            LiftType::Normal => {
                if action == A::RunningUpright {
                    A::RunningStairs
                } else {
                    A::WalkingUpright
                }
            }
            LiftType::Stairs => {
                if action == A::RunningUpright {
                    action
                } else {
                    A::WalkingStairs
                }
            }
            LiftType::Ladder => match action {
                A::RunningUpright => A::ClimbingLadderUpFast,
                A::WalkingAlerted => A::ClimbingLadderUpAlerted,
                _ => A::ClimbingLadderUp,
            },
            LiftType::Wall => {
                if action == A::RunningUpright {
                    A::ClimbingWallUpFast
                } else {
                    A::ClimbingWallUp
                }
            }
        }
    }

    /// Pick the animation to use when traversing this lift downwards with
    /// the given base movement action.
    pub fn get_action_downwards(&self, action: crate::order::OrderType) -> crate::order::OrderType {
        use crate::order::OrderType as A;
        match self.lift_type {
            LiftType::Normal => {
                if action == A::RunningUpright {
                    A::RunningStairs
                } else {
                    A::WalkingUpright
                }
            }
            LiftType::Stairs => {
                if action == A::RunningUpright {
                    action
                } else {
                    A::WalkingStairs
                }
            }
            LiftType::Ladder => match action {
                A::RunningUpright => A::ClimbingLadderDownFast,
                A::WalkingAlerted => A::ClimbingLadderDownAlerted,
                _ => A::ClimbingLadderDown,
            },
            LiftType::Wall => {
                if action == A::RunningUpright {
                    A::ClimbingWallDownFast
                } else {
                    A::ClimbingWallDown
                }
            }
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// BuildingData
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Building interior with occupant tracking.
#[derive(Debug, Clone, Default, Serialize, Deserialize, robin_state_hash_derive::StateHash)]
pub struct BuildingData {
    /// Inherited motion area data.
    pub motion: MotionAreaData,

    /// Maximum number of occupants allowed (loaded from proto, `None` = unlimited).
    pub max_occupants: Option<u16>,

    // ── Serialized (save-game state) ──
    /// Entities currently inside the building.
    pub occupant_indices: Vec<crate::entity_id::EntityId>,

    /// Whether this building has an arrow reserve the player can collect.
    pub arrow_reserve: bool,
}

impl BuildingData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_authorized(&self) -> bool {
        match self.max_occupants {
            None => true,
            Some(limit) => (self.occupant_indices.len() as u16) < limit,
        }
    }

    pub fn num_occupants(&self) -> usize {
        self.occupant_indices.len()
    }

    pub fn enter(&mut self, element_index: crate::entity_id::EntityId) {
        debug_assert!(
            !self.occupant_indices.contains(&element_index),
            "actor already in building"
        );
        self.occupant_indices.push(element_index);
    }

    pub fn leave(&mut self, element_index: crate::entity_id::EntityId) {
        let pos = self
            .occupant_indices
            .iter()
            .position(|&i| i == element_index)
            .expect("actor not in building");
        self.occupant_indices.remove(pos);
    }

    /// Classify the NPC occupants (villains vs civilians).
    ///
    /// `is_npc_alive_soldier` and `is_npc_alive_civilian` are closures that
    /// query actor properties by element index.
    pub fn get_kind_of_occupants<F, G>(
        &self,
        is_npc_alive_soldier: F,
        is_npc_alive_civilian: G,
    ) -> (OccupantKind, u16)
    where
        F: Fn(crate::entity_id::EntityId) -> bool,
        G: Fn(crate::entity_id::EntityId) -> bool,
    {
        let mut villains: u16 = 0;
        let mut civilians: u16 = 0;

        for &idx in &self.occupant_indices {
            if is_npc_alive_soldier(idx) {
                villains += 1;
            } else if is_npc_alive_civilian(idx) {
                civilians += 1;
            }
        }

        if villains != 0 {
            (OccupantKind::Villains, villains)
        } else if civilians != 0 {
            (OccupantKind::Civilians, civilians)
        } else {
            (OccupantKind::Empty, 0)
        }
    }

    pub fn has_arrow_reserve(&self) -> bool {
        self.arrow_reserve
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// PlaneData
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Height plane for 3D sight projection.
#[derive(Debug, Clone, Default, Serialize, Deserialize, robin_state_hash_derive::StateHash)]
pub struct PlaneData {
    /// Index of the associated sight obstacle.
    pub sight_obstacle_index: Option<crate::sight_obstacle::SightObstacleIndex>,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// PatchData
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Sector linked to a terrain patch (for dynamic terrain changes).
#[derive(Debug, Clone, Default, Serialize, Deserialize, robin_state_hash_derive::StateHash)]
pub struct PatchData {
    /// Index of the associated patch.
    pub patch_index: Option<crate::patch::PatchIndex>,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// AssociatedData
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Clickable overlay sector linked to another sector (e.g. lift mouse zones).
#[derive(Debug, Clone, Default, Serialize, Deserialize, robin_state_hash_derive::StateHash)]
pub struct AssociatedData {
    /// Index of the associated sector.
    pub associated_sector_index: Option<crate::fast_find_grid::SectorIndex>,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// DoorSectorData
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Clickable polygon around a door (for mouse interaction).
#[derive(Debug, Clone, Default, Serialize, Deserialize, robin_state_hash_derive::StateHash)]
pub struct DoorSectorData {
    /// Index of the associated door in the global gate table.
    pub door_index: Option<u32>,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// MaterialData
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Material / sound sector — determines footstep sounds and projectile bounce
/// properties.
#[derive(Debug, Clone, Serialize, Deserialize, robin_state_hash_derive::StateHash)]
pub struct MaterialData {
    /// Material type for footstep sounds.
    pub material: Material,

    /// Vertical bounce factor for projectiles.
    pub bounce_vertical: f32,

    /// Horizontal bounce factor for projectiles.
    pub bounce_horizontal: f32,

    /// Probability (0..255) of playing a footstep sound.
    pub sound_probability: u8,
}

impl Default for MaterialData {
    fn default() -> Self {
        Self {
            material: Material::Ground,
            bounce_vertical: 0.0,
            bounce_horizontal: 0.0,
            sound_probability: 0,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// ScriptSectorData
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Script-triggered zone sector with enter/leave events.
#[derive(Debug, Clone, Serialize, Deserialize, robin_state_hash_derive::StateHash)]
pub struct ScriptSectorData {
    /// Sector index within the fast-find grid.  `None` until the
    /// owning sector is registered with the grid.
    pub sector_index: Option<crate::fast_find_grid::SectorIndex>,

    /// Whether this sector has a script class bound to it.
    pub script_associated: bool,

    /// Script class name (if `script_associated`).
    pub script_class_name: Option<String>,

    /// Production type for production sectors (UNKNOWN if not a production sector).
    pub production_sector_type: sector_production::Type,

    /// Maximum throwing apex height (only valid when sector_type contains APEX).
    pub max_throwing_apex_height: f32,

    /// `true` once `DefineFlatTrajectoryZone` has converted this script
    /// sector into an apex sector.  The conversion drops the SCRIPT|CROSS
    /// bits and rewrites the type to APEX, so the engine then stops
    /// scanning the sector for occupant transitions.  Our overlay can only
    /// OR bits, so we track the conversion explicitly and skip the zone
    /// in `tick_zone_occupants`.
    pub transformed_to_apex: bool,

    // ── Serialized (save-game state) ──
    /// Entities currently inside this script zone.
    pub occupant_indices: Vec<crate::entity_id::EntityId>,
}

impl Default for ScriptSectorData {
    fn default() -> Self {
        Self {
            sector_index: None,
            script_associated: false,
            script_class_name: None,
            production_sector_type: sector_production::Type::Unknown,
            max_throwing_apex_height: 0.0,
            transformed_to_apex: false,
            occupant_indices: Vec::new(),
        }
    }
}

impl ScriptSectorData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_inside(&self, element_index: crate::entity_id::EntityId) -> bool {
        self.occupant_indices.contains(&element_index)
    }

    pub fn any_occupant(&self) -> Option<crate::entity_id::EntityId> {
        self.occupant_indices.first().copied()
    }

    pub fn enter(&mut self, element_index: crate::entity_id::EntityId) {
        if self.occupant_indices.contains(&element_index) {
            tracing::warn!(
                "actor {element_index} entering script sector {:?} twice",
                self.sector_index
            );
            return;
        }
        // Insert at the front.
        self.occupant_indices.insert(0, element_index);
    }

    pub fn leave(&mut self, element_index: crate::entity_id::EntityId) {
        if let Some(pos) = self
            .occupant_indices
            .iter()
            .position(|&i| i == element_index)
        {
            self.occupant_indices.remove(pos);
        } else {
            tracing::warn!(
                "actor {element_index} leaving script sector {:?} it hadn't entered",
                self.sector_index
            );
        }
    }

    pub fn remove_all_occupants(&mut self) {
        self.occupant_indices.clear();
    }

    pub fn num_occupants(&self) -> usize {
        self.occupant_indices.len()
    }

    pub fn get_occupant(&self, index: usize) -> crate::entity_id::EntityId {
        self.occupant_indices[index]
    }

    /// Transform this script sector into an apex sector.
    /// The sector_type on the parent `Sector` must also be updated to `APEX`.
    pub fn transform_into_apex(&mut self, apex_height: f32) {
        assert!(
            !self.script_associated,
            "cannot convert a script-associated sector to apex"
        );
        self.max_throwing_apex_height = apex_height;
        // Sector is no longer a script zone — see field doc.
        self.transformed_to_apex = true;
        // Drop any cached occupants; the occupant list isn't maintained
        // once the sector becomes APEX.
        self.occupant_indices.clear();
    }

    pub fn get_apex_height(&self) -> f32 {
        self.max_throwing_apex_height
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// ArcheryPoint / ArcheryData
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Index of a waypoint within an archery sector's `points` Vec.
///
/// Plain `u16` newtype with `Default = 0` (the first waypoint is a
/// valid index).  Sentinel fields like `index_first_shooting_point`
/// use `Option<ArcheryPointIdx>` with `None` instead of `u16::MAX`.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    robin_state_hash_derive::StateHash,
)]
pub struct ArcheryPointIdx(pub u16);

impl From<ArcheryPointIdx> for u16 {
    #[inline]
    fn from(i: ArcheryPointIdx) -> u16 {
        i.0
    }
}
impl From<ArcheryPointIdx> for u32 {
    #[inline]
    fn from(i: ArcheryPointIdx) -> u32 {
        u32::from(i.0)
    }
}
impl From<ArcheryPointIdx> for usize {
    #[inline]
    fn from(i: ArcheryPointIdx) -> usize {
        usize::from(i.0)
    }
}
impl From<u16> for ArcheryPointIdx {
    #[inline]
    fn from(v: u16) -> Self {
        Self(v)
    }
}
impl std::fmt::Display for ArcheryPointIdx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// A waypoint on an archery patrol path.
///
/// Each point has a position, an optional shooting direction, and can be
/// occupied by a single guard (soldier element).
#[derive(Debug, Clone, Serialize, Deserialize, robin_state_hash_derive::StateHash)]
pub struct ArcheryPoint {
    /// Map position of this waypoint.
    pub position: Point2D,

    /// Whether this is a shooting point (guards stop here to fire).
    pub is_shooting_point: bool,

    /// Layer this point is on.
    pub layer: u16,

    /// Facing direction (0..15) for shooting points.
    pub direction: u16,

    /// Sector index this point is associated with.
    pub sector_index: Option<crate::fast_find_grid::SectorIndex>,

    /// Entity currently occupying this point.  `None` means the point
    /// is free.
    pub owner_element_index: Option<crate::entity_id::EntityId>,
}

impl Default for ArcheryPoint {
    fn default() -> Self {
        Self {
            position: Point2D { x: 0.0, y: 0.0 },
            is_shooting_point: false,
            layer: 0,
            direction: 0,
            sector_index: None,
            owner_element_index: None,
        }
    }
}

impl ArcheryPoint {
    /// Assign a guard to this point.
    pub fn occupy(&mut self, element_index: crate::entity_id::EntityId) {
        assert!(
            self.owner_element_index.is_none(),
            "archery point already occupied by {:?}",
            self.owner_element_index
        );
        self.owner_element_index = Some(element_index);
    }

    /// Release the guard from this point.
    pub fn release(&mut self) {
        assert!(
            self.owner_element_index.is_some(),
            "archery point is not occupied"
        );
        self.owner_element_index = None;
    }

    /// Whether a guard is currently stationed at this point.
    pub fn is_occupied(&self) -> bool {
        self.owner_element_index.is_some()
    }

    /// Get the entity of the occupying guard, if any.
    pub fn owner(&self) -> Option<crate::entity_id::EntityId> {
        self.owner_element_index
    }
}

/// Archery / guard patrol sector with waypoints and shooting positions.
#[derive(Debug, Clone, Default, Serialize, Deserialize, robin_state_hash_derive::StateHash)]
pub struct ArcheryData {
    /// Waypoints along the patrol path (at least 3).
    pub waypoints: Vec<ArcheryPoint>,

    /// Index of the sector this archery zone is associated with.
    pub associated_sector_index: Option<crate::fast_find_grid::SectorIndex>,

    /// Index of the first shooting point in `waypoints`.  `None` when
    /// the sector has no shooting points.
    pub index_first_shooting_point: Option<ArcheryPointIdx>,

    /// Index of the last shooting point in `waypoints`.  `None` when
    /// the sector has no shooting points.
    pub index_last_shooting_point: Option<ArcheryPointIdx>,

    /// Total number of shooting points.
    pub num_shooting_points: u16,

    // ── Serialized ──
    /// Number of guards currently assigned to this archery sector.
    pub num_owners: u16,
}

impl ArcheryData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn num_waypoints(&self) -> usize {
        self.waypoints.len()
    }

    pub fn get_waypoint(&self, index: usize) -> &ArcheryPoint {
        &self.waypoints[index]
    }

    pub fn is_full(&self) -> bool {
        self.num_owners >= self.num_shooting_points
    }

    pub fn increment_owner_counter(&mut self) {
        assert!(!self.is_full(), "archery sector is full");
        self.num_owners += 1;
    }

    pub fn decrement_owner_counter(&mut self) {
        assert!(self.num_owners > 0, "archery sector has no owners");
        self.num_owners -= 1;
    }

    /// Check if a position at the given layer is inside this archery zone.
    ///
    /// Layer check, then polygon containment.
    pub fn is_inside(&self, position: Point2D, layer: u16, parent_sector: &Sector) -> bool {
        if parent_sector.layer != Some(layer) {
            return false;
        }
        parent_sector.polygon_contains(position)
    }

    /// Find the first unoccupied shooting point, returning its index.
    pub fn find_free_shooting_point(&self) -> Option<usize> {
        for (i, wp) in self.waypoints.iter().enumerate() {
            if wp.is_shooting_point && !wp.is_occupied() {
                return Some(i);
            }
        }
        None
    }

    /// Assign a guard to a specific waypoint.
    pub fn occupy_point(&mut self, point_index: usize, element_index: crate::entity_id::EntityId) {
        self.waypoints[point_index].occupy(element_index);
    }

    /// Release a guard from a specific waypoint.
    pub fn release_point(&mut self, point_index: usize) {
        self.waypoints[point_index].release();
    }

    /// Find the waypoint occupied by the given element and release it.
    /// Returns the index of the released point, or `None` if not found.
    pub fn release_by_owner(&mut self, element_index: crate::entity_id::EntityId) -> Option<usize> {
        for (i, wp) in self.waypoints.iter_mut().enumerate() {
            if wp.owner_element_index == Some(element_index) {
                wp.owner_element_index = None;
                return Some(i);
            }
        }
        None
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// ShadowData
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Shadow sector with computed barycentre and average radius.
#[derive(Debug, Clone, Serialize, Deserialize, robin_state_hash_derive::StateHash)]
pub struct ShadowData {
    /// 2D barycentre (centroid) of the shadow polygon.
    pub barycentre_2d: Point2D,

    /// 3D barycentre — includes Z from the height plane at the centroid.
    pub barycentre_3d_x: f32,
    pub barycentre_3d_y: f32,
    pub barycentre_3d_z: f32,

    /// Average distance from centroid to vertices.
    pub radius: f32,
}

impl Default for ShadowData {
    fn default() -> Self {
        Self {
            barycentre_2d: Point2D { x: 0.0, y: 0.0 },
            barycentre_3d_x: 0.0,
            barycentre_3d_y: 0.0,
            barycentre_3d_z: 0.0,
            radius: 0.0,
        }
    }
}

impl ShadowData {
    /// Compute the 2D barycentre and average radius from the parent sector's
    /// polygon points.
    ///
    /// Note: the 3D barycentre requires height plane lookup and must be
    /// computed separately (see `initialize_3d`).
    pub fn initialize_2d(&mut self, points: &[Point2D]) {
        assert!(!points.is_empty(), "shadow sector has no points");
        let n = points.len() as f32;

        let mut cx = 0.0_f32;
        let mut cy = 0.0_f32;
        for p in points {
            cx += p.x;
            cy += p.y;
        }
        let inv_n = 1.0 / n;
        cx *= inv_n;
        cy *= inv_n;
        self.barycentre_2d = Point2D { x: cx, y: cy };

        let mut total_dist = 0.0_f32;
        for p in points {
            let dx = p.x - cx;
            let dy = p.y - cy;
            total_dist += (dx * dx + dy * dy).sqrt();
        }
        self.radius = total_dist * inv_n;
    }

    /// Compute the 3D barycentre given the top plane of the plane sector
    /// that contains the shadow's 2D barycentre.  `top_plane_points` comes
    /// from the `SightObstacle` of the enclosing `SectorKind::Plane`; the
    /// caller is responsible for looking it up via the fast-find grid.
    /// Pass `None` when no plane sector encloses the barycentre — we fall
    /// back to a flat `z=0`.
    ///
    /// The 2D and 3D passes are split so the engine can drive the plane
    /// lookup.
    pub fn initialize_3d(&mut self, top_plane_points: Option<&[[f32; 3]; 3]>) {
        let bx = self.barycentre_2d.x;
        let by = self.barycentre_2d.y;
        match top_plane_points {
            None => {
                self.barycentre_3d_x = bx;
                self.barycentre_3d_y = by;
                self.barycentre_3d_z = 0.0;
            }
            Some(points) => {
                // Derive the plane coefficients from the 3 top-plane
                // points.  The plane is `z = A*x + B*y + D`, so from
                // the normal (nx, ny, nz) = v1 × v2:
                //   A = -nx/nz, B = -ny/nz,
                //   D = p0z + (nx*p0x + ny*p0y)/nz.
                let [p0, p1, p2] = *points;
                let v1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
                let v2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
                let nx = v1[1] * v2[2] - v1[2] * v2[1];
                let ny = v1[2] * v2[0] - v1[0] * v2[2];
                let nz = v1[0] * v2[1] - v1[1] * v2[0];
                if nz.abs() < 1e-9 {
                    // Degenerate plane — fall back to flat.
                    self.barycentre_3d_x = bx;
                    self.barycentre_3d_y = by;
                    self.barycentre_3d_z = 0.0;
                    return;
                }
                let fa = -nx / nz;
                let fb = -ny / nz;
                let fd = p0[2] + (nx * p0[0] + ny * p0[1]) / nz;
                let denom = 1.0 - fb;
                let z = if denom.abs() < 1e-9 {
                    0.0
                } else {
                    0.1 + (fb * by + fa * bx + fd) / denom
                };
                self.barycentre_3d_z = z;
                // Iso projection: screen-space Y is world Y + Z.
                self.barycentre_3d_y = by + z;
                self.barycentre_3d_x = bx;
            }
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Tests
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[cfg(test)]
mod tests {
    use super::*;
    use crate::element::ElementKind;
    use crate::geo2d::pt;

    #[test]
    fn sector_type_bitflags() {
        let t = SectorType::MOTION | SectorType::AREA;
        assert!(t.is_motion());
        assert!(t.is_area());
        assert!(!t.is_obstacle());
        assert!(!t.is_lift());

        let t2 = t | SectorType::LIFT;
        assert!(t2.is_lift());
        assert!(t2.is_motion());
        assert!(t2.is_area());
    }

    #[test]
    fn lift_type_from_u8() {
        assert_eq!(LiftType::from_u8(0), LiftType::Normal);
        assert_eq!(LiftType::from_u8(1), LiftType::Stairs);
        assert_eq!(LiftType::from_u8(2), LiftType::Ladder);
        assert_eq!(LiftType::from_u8(3), LiftType::Wall);

        assert!(LiftType::Wall.is_wall_or_ladder());
        assert!(LiftType::Ladder.is_wall_or_ladder());
        assert!(!LiftType::Stairs.is_wall_or_ladder());
    }

    #[test]
    #[should_panic(expected = "unknown lift type")]
    fn lift_type_from_u8_invalid() {
        LiftType::from_u8(99);
    }

    #[test]
    fn lift_type_translate_upright_action_stairs() {
        use crate::order::OrderType as A;
        // Stairs: walking → walking_stairs, running stays running.
        assert_eq!(
            LiftType::Stairs.translate_upright_action(A::WalkingUpright),
            A::WalkingStairs
        );
        assert_eq!(
            LiftType::Stairs.translate_upright_action(A::RunningUpright),
            A::RunningUpright
        );
    }

    #[test]
    fn lift_type_translate_upright_action_ladder_and_wall() {
        use crate::order::OrderType as A;
        assert_eq!(
            LiftType::Ladder.translate_upright_action(A::WalkingUpright),
            A::ClimbingLadderUp
        );
        assert_eq!(
            LiftType::Ladder.translate_upright_action(A::RunningUpright),
            A::ClimbingLadderUpFast
        );
        assert_eq!(
            LiftType::Ladder.translate_upright_action(A::WalkingAlerted),
            A::ClimbingLadderUpAlerted
        );
        assert_eq!(
            LiftType::Wall.translate_upright_action(A::WalkingUpright),
            A::ClimbingWallUp
        );
        assert_eq!(
            LiftType::Wall.translate_upright_action(A::RunningUpright),
            A::ClimbingWallUpFast
        );
    }

    #[test]
    fn lift_type_translate_upright_action_normal() {
        use crate::order::OrderType as A;
        // Normal lift: walking stays walking, running becomes running_stairs.
        assert_eq!(
            LiftType::Normal.translate_upright_action(A::WalkingUpright),
            A::WalkingUpright
        );
        assert_eq!(
            LiftType::Normal.translate_upright_action(A::RunningUpright),
            A::RunningStairs
        );
    }

    #[test]
    fn lift_type_translate_climb_action_ladder_directional() {
        use crate::order::OrderType as A;
        // Going down a ladder: walking → ClimbingLadderDown,
        // running → ClimbingLadderDownFast, alerted → ClimbingLadderDownAlerted.
        assert_eq!(
            LiftType::Ladder.translate_climb_action(A::WalkingUpright, true),
            A::ClimbingLadderDown
        );
        assert_eq!(
            LiftType::Ladder.translate_climb_action(A::RunningUpright, true),
            A::ClimbingLadderDownFast
        );
        assert_eq!(
            LiftType::Ladder.translate_climb_action(A::WalkingAlerted, true),
            A::ClimbingLadderDownAlerted
        );
        // Going up: same as upright translation.
        assert_eq!(
            LiftType::Ladder.translate_climb_action(A::WalkingUpright, false),
            A::ClimbingLadderUp
        );
        assert_eq!(
            LiftType::Ladder.translate_climb_action(A::RunningUpright, false),
            A::ClimbingLadderUpFast
        );
    }

    #[test]
    fn lift_type_translate_climb_action_wall_directional() {
        use crate::order::OrderType as A;
        assert_eq!(
            LiftType::Wall.translate_climb_action(A::WalkingUpright, true),
            A::ClimbingWallDown
        );
        assert_eq!(
            LiftType::Wall.translate_climb_action(A::RunningUpright, true),
            A::ClimbingWallDownFast
        );
        assert_eq!(
            LiftType::Wall.translate_climb_action(A::WalkingUpright, false),
            A::ClimbingWallUp
        );
        assert_eq!(
            LiftType::Wall.translate_climb_action(A::RunningUpright, false),
            A::ClimbingWallUpFast
        );
    }

    #[test]
    fn lift_type_translate_climb_action_stairs_normal_symmetric() {
        use crate::order::OrderType as A;
        // Stairs / Normal: up == down.  Direction is ignored.
        for going_down in [false, true] {
            assert_eq!(
                LiftType::Stairs.translate_climb_action(A::WalkingUpright, going_down),
                A::WalkingStairs
            );
            assert_eq!(
                LiftType::Stairs.translate_climb_action(A::RunningUpright, going_down),
                A::RunningUpright
            );
            assert_eq!(
                LiftType::Normal.translate_climb_action(A::WalkingUpright, going_down),
                A::WalkingUpright
            );
            assert_eq!(
                LiftType::Normal.translate_climb_action(A::RunningUpright, going_down),
                A::RunningStairs
            );
        }
    }

    #[test]
    fn lift_type_upright_matches_climb_upwards() {
        // `translate_upright_action` must be identical to
        // `translate_climb_action(_, false)` for every type / action,
        // since upwards == downwards for upright posture.
        use crate::order::OrderType as A;
        for lt in [
            LiftType::Normal,
            LiftType::Stairs,
            LiftType::Ladder,
            LiftType::Wall,
        ] {
            for action in [
                A::WalkingUpright,
                A::RunningUpright,
                A::WalkingAlerted,
                A::WalkingCrouched,
            ] {
                assert_eq!(
                    lt.translate_upright_action(action),
                    lt.translate_climb_action(action, false),
                    "upright vs climb-upwards mismatch for {lt:?} {action:?}",
                );
            }
        }
    }

    #[test]
    fn sector_add_point_expands_bbox() {
        let mut sector = Sector::new(SectorType::AREA);
        sector.add_point(pt(10.0, 20.0));
        sector.add_point(pt(50.0, 60.0));
        sector.add_point(pt(30.0, 40.0));

        assert_eq!(sector.num_points(), 3);
        assert!(sector.bounding_box.is_somewhere());
        assert!((sector.bounding_box.x_min() - 10.0).abs() < 1e-6);
        assert!((sector.bounding_box.y_min() - 20.0).abs() < 1e-6);
        assert!((sector.bounding_box.x_max() - 50.0).abs() < 1e-6);
        assert!((sector.bounding_box.y_max() - 60.0).abs() < 1e-6);
    }

    #[test]
    fn sector_with_kind_default_type() {
        let s = Sector::with_kind(SectorKind::Lift(Box::new(LiftData::new())));
        assert!(s.sector_type.is_motion());
        assert!(s.sector_type.is_area());
        assert!(s.sector_type.is_lift());
    }

    #[test]
    fn lift_occupant_tracking() {
        let mut lift = LiftData::new();
        assert!(!lift.is_occupied());

        lift.set_occupied_downwards(true, true);
        assert!(lift.is_occupied());
        assert!(lift.is_occupied_by_pc());
        assert_eq!(lift.occupants, 1);
        assert_eq!(lift.occupants_pc, 1);
        assert!(lift.occupied_downwards);
        assert_eq!(lift.wait_time, 100);

        lift.set_occupied_downwards(true, false);
        assert!(!lift.is_occupied());
        assert_eq!(lift.wait_time, 0);
        assert!(!lift.occupied_downwards);
        assert!(!lift.occupied_upwards);
    }

    #[test]
    fn lift_upward_occupant_tracking() {
        let mut lift = LiftData::new();

        lift.set_occupied_upwards(false, true);
        assert!(lift.is_occupied());
        assert!(!lift.is_occupied_by_pc());
        assert!(lift.occupied_upwards);
        assert_eq!(lift.wait_time, 80);

        lift.set_occupied_upwards(false, false);
        assert!(!lift.is_occupied());
    }

    #[test]
    fn building_enter_leave() {
        use crate::entity_id::EntityId;
        let mut bld = BuildingData::new();
        assert!(bld.is_authorized());
        assert_eq!(bld.num_occupants(), 0);

        bld.enter(EntityId(42));
        bld.enter(EntityId(7));
        assert_eq!(bld.num_occupants(), 2);

        bld.leave(EntityId(42));
        assert_eq!(bld.num_occupants(), 1);
        assert_eq!(bld.occupant_indices[0], EntityId(7));
    }

    #[test]
    fn building_kind_of_occupants() {
        use crate::entity_id::EntityId;
        let mut bld = BuildingData::new();
        bld.enter(EntityId(1));
        bld.enter(EntityId(2));
        bld.enter(EntityId(3));

        // 1 = soldier, 2 = civilian, 3 = soldier
        let (kind, count) = bld.get_kind_of_occupants(
            |id| id == EntityId(1) || id == EntityId(3), // is_npc_alive_soldier
            |id| id == EntityId(2),                      // is_npc_alive_civilian
        );
        assert_eq!(kind, OccupantKind::Villains);
        assert_eq!(count, 2);
    }

    #[test]
    fn script_sector_enter_leave() {
        use crate::entity_id::EntityId;
        let mut script = ScriptSectorData::new();
        script.enter(EntityId(10));
        script.enter(EntityId(20));

        assert!(script.is_inside(EntityId(10)));
        assert!(script.is_inside(EntityId(20)));
        assert!(!script.is_inside(EntityId(30)));
        assert_eq!(script.num_occupants(), 2);

        // Insert at front
        assert_eq!(script.get_occupant(0), EntityId(20));
        assert_eq!(script.get_occupant(1), EntityId(10));

        script.leave(EntityId(10));
        assert!(!script.is_inside(EntityId(10)));
        assert_eq!(script.num_occupants(), 1);
    }

    #[test]
    fn archery_owner_counter() {
        let mut arch = ArcheryData::new();
        arch.num_shooting_points = 2;

        assert!(!arch.is_full());
        arch.increment_owner_counter();
        assert!(!arch.is_full());
        arch.increment_owner_counter();
        assert!(arch.is_full());

        arch.decrement_owner_counter();
        assert!(!arch.is_full());
    }

    #[test]
    fn archery_point_occupy_release() {
        let mut point = ArcheryPoint {
            position: pt(100.0, 200.0),
            is_shooting_point: true,
            ..ArcheryPoint::default()
        };

        assert!(!point.is_occupied());
        assert_eq!(point.owner(), None);

        point.occupy(crate::entity_id::EntityId(42));
        assert!(point.is_occupied());
        assert_eq!(point.owner(), Some(crate::entity_id::EntityId(42)));

        point.release();
        assert!(!point.is_occupied());
    }

    #[test]
    #[should_panic(expected = "already occupied")]
    fn archery_point_double_occupy_panics() {
        let mut point = ArcheryPoint {
            is_shooting_point: true,
            ..ArcheryPoint::default()
        };
        point.occupy(crate::entity_id::EntityId(1));
        point.occupy(crate::entity_id::EntityId(2));
    }

    #[test]
    fn archery_find_free_shooting_point() {
        let mut arch = ArcheryData::new();
        arch.waypoints = vec![
            ArcheryPoint {
                position: pt(0.0, 0.0),
                is_shooting_point: false,
                ..ArcheryPoint::default()
            },
            ArcheryPoint {
                position: pt(10.0, 0.0),
                is_shooting_point: true,
                owner_element_index: Some(crate::entity_id::EntityId(99)),
                ..ArcheryPoint::default()
            },
            ArcheryPoint {
                position: pt(20.0, 0.0),
                is_shooting_point: true,
                ..ArcheryPoint::default()
            },
        ];
        arch.num_shooting_points = 2;

        // First shooting point (index 1) is occupied, second (index 2) is free
        assert_eq!(arch.find_free_shooting_point(), Some(2));

        arch.occupy_point(2, crate::entity_id::EntityId(50));
        assert_eq!(arch.find_free_shooting_point(), None);
    }

    #[test]
    fn archery_release_by_owner() {
        use crate::entity_id::EntityId;
        let mut arch = ArcheryData::new();
        arch.waypoints = vec![
            ArcheryPoint {
                is_shooting_point: true,
                owner_element_index: Some(EntityId(42)),
                ..ArcheryPoint::default()
            },
            ArcheryPoint {
                is_shooting_point: true,
                owner_element_index: Some(EntityId(99)),
                ..ArcheryPoint::default()
            },
        ];

        assert_eq!(arch.release_by_owner(EntityId(99)), Some(1));
        assert!(!arch.waypoints[1].is_occupied());
        assert_eq!(arch.release_by_owner(EntityId(99)), None); // already released
    }

    // -- Lift authorization tests --

    fn lift_actor(kind: ElementKind, has_climb: bool) -> ActorAuthInfo {
        ActorAuthInfo {
            kind,
            pc_auth_bit: 0,
            has_lockpick: false,
            has_climb,
            has_jump: false,
            is_rider: false,
            posture: crate::element::Posture::Upright,
        }
    }

    #[test]
    fn lift_wall_only_pc_with_climb() {
        let mut lift = LiftData::new();
        lift.lift_type = LiftType::Wall;

        let pc_climb = lift_actor(ElementKind::ActorPc, true);
        let pc_no_climb = lift_actor(ElementKind::ActorPc, false);
        let soldier = lift_actor(ElementKind::ActorSoldier, false);

        assert!(lift.is_actor_authorized(&pc_climb));
        assert!(!lift.is_actor_authorized(&pc_no_climb));
        assert!(!lift.is_actor_authorized(&soldier));
    }

    #[test]
    fn lift_ladder_humans_except_civilians() {
        let mut lift = LiftData::new();
        lift.lift_type = LiftType::Ladder;

        let pc = lift_actor(ElementKind::ActorPc, false);
        let soldier = lift_actor(ElementKind::ActorSoldier, false);
        let civilian = lift_actor(ElementKind::ActorCivilian, false);

        assert!(lift.is_actor_authorized(&pc));
        assert!(lift.is_actor_authorized(&soldier));
        assert!(!lift.is_actor_authorized(&civilian));
    }

    #[test]
    fn lift_stairs_allows_humans() {
        let mut lift = LiftData::new();
        lift.lift_type = LiftType::Stairs;

        let pc = lift_actor(ElementKind::ActorPc, false);
        let civilian = lift_actor(ElementKind::ActorCivilian, false);

        assert!(lift.is_actor_authorized(&pc));
        assert!(lift.is_actor_authorized(&civilian));
    }

    #[test]
    fn lift_normal_allows_everyone() {
        let lift = LiftData::new(); // default is Normal
        let pc = lift_actor(ElementKind::ActorPc, false);
        assert!(lift.is_actor_authorized(&pc));
    }

    #[test]
    fn shadow_initialize_2d() {
        let mut shadow = ShadowData::default();
        let points = vec![pt(0.0, 0.0), pt(10.0, 0.0), pt(10.0, 10.0), pt(0.0, 10.0)];
        shadow.initialize_2d(&points);

        assert!((shadow.barycentre_2d.x - 5.0).abs() < 1e-6);
        assert!((shadow.barycentre_2d.y - 5.0).abs() < 1e-6);
        assert!(shadow.radius > 0.0);
    }

    #[test]
    fn sector_serde_roundtrip() {
        let mut sector = Sector::with_kind(SectorKind::Lift(Box::new(LiftData::new())));
        // Set some serialized state
        if let SectorKind::Lift(ref mut lift) = sector.kind {
            lift.occupants = 3;
            lift.occupants_pc = 1;
            lift.occupied_downwards = true;
            lift.wait_time = 50;
        }

        let json = serde_json::to_string(&sector).unwrap();
        let deserialized: Sector = serde_json::from_str(&json).unwrap();

        // All fields survive a full round-trip now that sector serialization
        // covers level data alongside runtime state.
        let lift = deserialized.as_lift().unwrap();
        assert_eq!(lift.occupants, 3);
        assert_eq!(lift.occupants_pc, 1);
        assert!(lift.occupied_downwards);
        assert_eq!(lift.wait_time, 50);
        assert_eq!(lift.lift_type, LiftType::Normal);
        assert_eq!(deserialized.sector_type, sector.sector_type);
        assert_eq!(deserialized.points, sector.points);
    }

    #[test]
    fn building_serde_roundtrip() {
        use crate::entity_id::EntityId;
        let mut bld = BuildingData::new();
        bld.enter(EntityId(1));
        bld.enter(EntityId(2));
        bld.arrow_reserve = true;

        let sector = Sector {
            kind: SectorKind::Building(Box::new(bld)),
            ..Sector::new(SectorType::MOTION | SectorType::AREA | SectorType::BUILDING)
        };

        let json = serde_json::to_string(&sector).unwrap();
        let de: Sector = serde_json::from_str(&json).unwrap();
        let bld2 = de.as_building().unwrap();
        assert_eq!(bld2.occupant_indices, vec![EntityId(1), EntityId(2)]);
        assert!(bld2.arrow_reserve);
    }
}
