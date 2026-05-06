//! Read-only loader for the original binary save game format.
//!
//! The original game serialized campaign state as raw little-endian
//! field dumps. This module reads that format and produces Rust
//! `Campaign` structs.
//!
//! **Write support is intentionally omitted** — new saves use serde JSON.
//!
//! ## Save file layout (version 48)
//!
//! ```text
//! Header:
//!   [4]  "RHSG" magic
//!   u32  header version
//!   u32  mission ID
//!   u32  file version (used for conditional fields)
//! Campaign data:
//!   [16] MD5("RHCampaign")
//!   bool reservists_are_back        (if version >= 28)
//!   i32  values[27]
//!   i8   ares
//!   Container<Mission>
//!   Container<MissionPointer>       (accessible)
//!   Container<MissionPointer>       (pending accessible)
//!   AllCharacters
//!   Container<PCDescriptionPtr>     (gang)
//!   Container<PCDescriptionPtr>     (reservists)
//!   Container<PCDescriptionPtr>     (mission team)
//!   Container<SectorProduction>
//!   Container<ObjectType>           (collected relics)
//!   Container<WideString>           (peasant names)
//!   MissionPointer × 4              (last, current, next, blazon)
//!   LastPlayedMissions              (if version >= 30)
//!   LastPseudoMissionStatus         (if version >= 41)
//! ```

use std::io::Read;
use std::path::Path;

use anyhow::{Context, Result, bail};
use byteorder::{LittleEndian, ReadBytesExt};
use md5_crate::{Digest, Md5};

use crate::campaign::{Campaign, NUMBER_OF_VALUES, PcDescription};
use crate::mission::{Mission, MissionStatus};
use crate::pc_status::{HumanStatus, PcStatus, Skill};
use crate::sector_production::SectorProduction;

// ─── Constants ──────────────────────────────────────────────────

const SAVE_MAGIC: &[u8; 4] = b"RHSG";
const SAVE_VERSION: u32 = 48;

// ─── Header ─────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SaveHeader {
    pub mark: [u8; 4],
    pub header_version: u32,
    pub mission_id: u32,
    pub file_version: u32,
}

// ─── Binary reader ──────────────────────────────────────────────

/// Wraps a `Read` stream and provides helpers for reading the legacy
/// binary save format (little-endian, sequential field dumps).
struct BinaryReader<R: Read> {
    inner: R,
    version: u32,
}

impl<R: Read> BinaryReader<R> {
    fn new(inner: R) -> Self {
        BinaryReader { inner, version: 0 }
    }

    // ── Primitive readers ───────────────────────────────────────

    fn read_u8(&mut self) -> Result<u8> {
        self.inner.read_u8().context("read u8")
    }

    fn read_bool(&mut self) -> Result<bool> {
        // bool is one byte in this format.
        Ok(self.read_u8()? != 0)
    }

    fn read_i8(&mut self) -> Result<i8> {
        self.inner.read_i8().context("read i8")
    }

    fn read_u16(&mut self) -> Result<u16> {
        self.inner.read_u16::<LittleEndian>().context("read u16")
    }

    fn read_i16(&mut self) -> Result<i16> {
        self.inner.read_i16::<LittleEndian>().context("read i16")
    }

    fn read_u32(&mut self) -> Result<u32> {
        self.inner.read_u32::<LittleEndian>().context("read u32")
    }

    fn read_i32(&mut self) -> Result<i32> {
        self.inner.read_i32::<LittleEndian>().context("read i32")
    }

    fn read_f32(&mut self) -> Result<f32> {
        self.inner.read_f32::<LittleEndian>().context("read f32")
    }

    fn skip(&mut self, n: usize) -> Result<()> {
        let mut buf = vec![0u8; n];
        self.inner.read_exact(&mut buf).context("skip bytes")?;
        Ok(())
    }

    // ── MD5 stream validation ──────────────────────────────────

    fn validate_stream(&mut self, name: &str) -> Result<()> {
        let expected = Md5::digest(name.as_bytes());
        let mut actual = [0u8; 16];
        self.inner
            .read_exact(&mut actual)
            .with_context(|| format!("read MD5 fingerprint for {name}"))?;
        if actual != expected.as_slice() {
            bail!(
                "MD5 validation failed for \"{name}\": expected {:02x?}, got {:02x?}",
                expected.as_slice(),
                actual
            );
        }
        Ok(())
    }

    // ── String reader ──────────────────────────────────────────

    /// Read a wide string: u16 length + `length` × u16 code units.
    /// Converts from UCS-2 / UTF-16LE to Rust UTF-8.
    fn read_wide_string(&mut self) -> Result<String> {
        let len = self.read_u16()? as usize;
        let mut code_units = Vec::with_capacity(len);
        for _ in 0..len {
            code_units.push(self.read_u16()?);
        }
        String::from_utf16(&code_units).context("invalid UTF-16 in wide string")
    }

    // ── Container reader ───────────────────────────────────────

    /// Read a u32 count, then `count` items using the provided closure.
    fn read_container<T, F>(&mut self, mut read_item: F) -> Result<Vec<T>>
    where
        F: FnMut(&mut Self) -> Result<T>,
    {
        let count = self.read_u32()? as usize;
        let mut items = Vec::with_capacity(count);
        for i in 0..count {
            items.push(read_item(self).with_context(|| format!("container item {i}/{count}"))?);
        }
        Ok(items)
    }

    // ── Profile pointer readers ─────────────────────────────────

    /// Read a mission profile pointer index (u32). Returns None if 0xFFFFFFFF.
    fn read_profile_index(&mut self) -> Result<Option<u32>> {
        let idx = self.read_u32()?;
        if idx == u32::MAX {
            Ok(None)
        } else {
            Ok(Some(idx))
        }
    }

    /// Read a mission pointer (u16 index into missions vec). Returns None if 0xFFFF.
    fn read_mission_pointer(&mut self) -> Result<Option<usize>> {
        let idx = self.read_u16()?;
        if idx == u16::MAX {
            Ok(None)
        } else {
            Ok(Some(idx as usize))
        }
    }

    /// Read a PC description pointer (u32 index into characters vec).
    fn read_pc_description_pointer(&mut self) -> Result<usize> {
        Ok(self.read_u32()? as usize)
    }

    // ── Struct readers ──────────────────────────────────────────

    fn read_header(&mut self) -> Result<SaveHeader> {
        let mut mark = [0u8; 4];
        self.inner
            .read_exact(&mut mark)
            .context("read save header magic")?;
        if &mark != SAVE_MAGIC {
            bail!(
                "invalid save file magic: expected {:?}, got {:?}",
                SAVE_MAGIC,
                mark
            );
        }

        let header_version = self.read_u32().context("read header version")?;
        let mission_id = self.read_u32().context("read mission ID")?;

        // File version — drives conditional field reading below.
        let file_version = self.read_u32().context("read file version")?;
        self.version = file_version;

        if header_version != SAVE_VERSION {
            tracing::warn!(
                "save header version {header_version} != expected {SAVE_VERSION}, \
                 attempting to load anyway"
            );
        }

        Ok(SaveHeader {
            mark,
            header_version,
            mission_id,
            file_version,
        })
    }

    fn read_human_status(&mut self) -> Result<HumanStatus> {
        self.validate_stream("RHHumanStatus")?;

        // SKILL_NUMBER == 2: hand_to_hand, bow
        // Each skill: capacity (u32) then experience (u32)
        let hth_capacity = self.read_u32()?;
        let hth_experience = self.read_u32()?;
        let bow_capacity = self.read_u32()?;
        let bow_experience = self.read_u32()?;

        Ok(HumanStatus {
            hand_to_hand: Skill {
                capacity: hth_capacity,
                experience: hth_experience,
            },
            bow: Skill {
                capacity: bow_capacity,
                experience: bow_experience,
            },
        })
    }

    fn read_pc_status(&mut self) -> Result<PcStatus> {
        // Base class first.
        let human_status = self.read_human_status()?;

        self.validate_stream("RHPCStatus")?;

        let life_points = self.read_i16()?;
        let in_coma = self.read_bool()?;
        let num_ales = self.read_u16()?;
        let num_apples = self.read_u16()?;
        let num_arrows = self.read_u16()?;
        let num_nets = self.read_u16()?;
        let num_plants = self.read_u16()?;
        let num_purses = self.read_u16()?;
        let num_rations = self.read_u16()?; // Stoeckel rations
        let num_stones = self.read_u16()?;
        let num_wasp_nests = self.read_u16()?;
        let beam_me_index_in_sherwood = self.read_i16()?;
        let name = self.read_wide_string()?;

        Ok(PcStatus {
            human_status,
            life_points,
            in_coma,
            num_ales,
            num_arrows,
            num_apples,
            num_rations,
            num_stones,
            num_wasp_nests,
            num_nets,
            num_plants,
            num_purses,
            name,
            name_override: None,
            beam_me_index_in_sherwood,
        })
    }

    fn read_pc_description(&mut self) -> Result<PcDescription> {
        let status = self.read_pc_status()?;
        let character_profile_idx = self.read_profile_index()?;
        let instanced = self.read_bool()?;

        Ok(PcDescription {
            character_profile_idx: character_profile_idx
                .map(robin_engine::profiles::CharacterProfileIdx),
            instanced,
            status,
        })
    }

    fn read_mission(&mut self) -> Result<Mission> {
        self.validate_stream("RHMission")?;

        let age = self.read_u16()?;
        let blazon_price = self.read_u16()?;
        let status_raw = self.read_u32()?;
        let status = match status_raw {
            1 => MissionStatus::Won,
            2 => MissionStatus::Lost,
            _ => MissionStatus::Available,
        };

        // Skip 2 × u16 padding.
        self.skip(4)?;

        let profile_idx = self.read_profile_index()?;

        Ok(Mission {
            age,
            blazon_price,
            status,
            profile_idx,
            ares_state_override: None,
        })
    }

    fn read_sector_production(&mut self) -> Result<SectorProduction> {
        self.validate_stream("RHSectorProduction")?;

        let prod_type_raw = self.read_u32()?;
        let prod_type = match prod_type_raw {
            0 => crate::sector_production::Type::MakeArrow,
            1 => crate::sector_production::Type::MakePurse,
            2 => crate::sector_production::Type::MakeStone,
            3 => crate::sector_production::Type::MakeApple,
            4 => crate::sector_production::Type::MakeAle,
            5 => crate::sector_production::Type::MakeLamblegg,
            6 => crate::sector_production::Type::MakePlant,
            7 => crate::sector_production::Type::MakeNet,
            8 => crate::sector_production::Type::MakeWaspNest,
            9 => crate::sector_production::Type::TrainBow,
            10 => crate::sector_production::Type::TrainHandToHand,
            11 => crate::sector_production::Type::Heal,
            12 => crate::sector_production::Type::Relic,
            _ => crate::sector_production::Type::Unknown,
        };

        let speed = self.read_u16()?;
        let amount = self.read_u16()?;
        let produced_amount = self.read_u16()?;
        let max_amount_reached = self.read_bool()?;

        // Read occupants container
        let version = self.version;
        let occupants = self.read_container(|r| {
            let pc_description_idx = r.read_pc_description_pointer()?;
            // 2D point: 2 × f32.
            let x = r.read_f32()?;
            let y = r.read_f32()?;

            let obstacle = if version >= 47 {
                r.read_u16()?
            } else {
                if version >= 46 {
                    // v46: dummy padding we skip.
                    let _dummy = r.read_u16()?;
                }
                0xFFFF
            };

            Ok(crate::sector_production::Occupant {
                pc_description_idx,
                x,
                y,
                obstacle,
            })
        })?;

        Ok(SectorProduction {
            prod_type,
            speed,
            production_points: Vec::new(), // not stored in save — comes from level scripts
            occupants,
            amount,
            produced_amount,
            max_amount_reached,
        })
    }

    fn read_campaign(&mut self) -> Result<Campaign> {
        self.validate_stream("RHCampaign")?;

        // reservists-are-back flag (version >= 28)
        let reservists_are_back = if self.version >= 28 {
            self.read_bool()?
        } else {
            false
        };

        // 27 × i32 campaign values.
        let mut values = [0i32; NUMBER_OF_VALUES];
        for v in &mut values {
            *v = self.read_i32()?;
        }

        // ARES — i8
        let ares = self.read_i8()?;

        // ── Containers ──

        let missions = self.read_container(|r| r.read_mission())?;

        // Accessible missions — mission pointer indices (u16 each).
        let accessible_mission_indices = self.read_container(|r| {
            r.read_mission_pointer()
                .map(|opt| opt.unwrap_or(usize::MAX))
        })?;

        let pending_accessible_mission_indices = self.read_container(|r| {
            r.read_mission_pointer()
                .map(|opt| opt.unwrap_or(usize::MAX))
        })?;

        // All characters: u32 count + count × PcDescription.
        let characters = self.read_container(|r| r.read_pc_description())?;

        // Gang — PC description pointer indices (u32 each).
        let gang_indices = self.read_container(|r| r.read_pc_description_pointer())?;

        let reservist_indices = self.read_container(|r| r.read_pc_description_pointer())?;

        let mission_team_indices = self.read_container(|r| r.read_pc_description_pointer())?;

        let production_sectors = self.read_container(|r| r.read_sector_production())?;

        // Collected relics — u32 enum values.
        let collected_relics = self.read_container(|r| r.read_u32())?;

        // Peasant names — wide strings.
        let peasant_names = self.read_container(|r| r.read_wide_string())?;

        // ── Mission pointers ──

        let last_mission_idx = self.read_mission_pointer()?;
        let current_mission_idx = self.read_mission_pointer()?;
        let next_mission_idx = self.read_mission_pointer()?;
        let blazon_mission_idx = self.read_mission_pointer()?;

        // Last played missions (version >= 30)
        let last_played_mission_indices = if self.version >= 30 {
            let count = self.read_u32()? as usize;
            let mut indices = Vec::with_capacity(count);
            for _ in 0..count {
                if let Some(idx) = self.read_mission_pointer()? {
                    indices.push(idx);
                }
            }
            indices
        } else {
            Vec::new()
        };

        // Last pseudo mission status (version >= 41)
        let (last_pseudo_mission_status, last_pseudo_mission_id) = if self.version >= 41 {
            let status_raw = self.read_u32()?;
            let status = match status_raw {
                1 => MissionStatus::Won,
                2 => MissionStatus::Lost,
                _ => MissionStatus::Available,
            };
            let id = if self.version >= 48 {
                self.read_u32()?
            } else {
                0
            };
            (status, id)
        } else {
            (MissionStatus::Available, 0)
        };

        Ok(Campaign {
            values,
            ares,
            missions,
            accessible_mission_indices,
            pending_accessible_mission_indices,
            last_mission_idx,
            current_mission_idx,
            next_mission_idx,
            blazon_mission_idx,
            last_played_mission_indices,
            last_pseudo_mission_status,
            last_pseudo_mission_id,
            characters,
            gang_indices,
            reservist_indices,
            mission_team_indices,
            peasant_names,
            reservists_are_back,
            collected_relics,
            production_sectors,
            pre_mission_snapshot: None,
        })
    }
}

// ─── Public API ─────────────────────────────────────────────────

/// Result of loading a legacy binary save file.
#[derive(Debug, Clone)]
pub struct LegacySave {
    pub header: SaveHeader,
    pub campaign: Campaign,
}

/// Load a legacy binary save file from the given path.
///
/// Returns the save header (with mission ID) and the deserialized campaign
/// state. Only the campaign-level save format is supported. In-mission
/// quicksaves that embed engine state will load the campaign portion only.
pub fn load_legacy_save(path: &Path) -> Result<LegacySave> {
    let data =
        std::fs::read(path).with_context(|| format!("read legacy save: {}", path.display()))?;
    load_legacy_save_from_bytes(&data)
}

/// Load a legacy save from an in-memory byte slice.
pub fn load_legacy_save_from_bytes(data: &[u8]) -> Result<LegacySave> {
    let mut reader = BinaryReader::new(data);
    let header = reader.read_header().context("failed to read save header")?;
    let campaign = reader
        .read_campaign()
        .context("failed to read campaign data")?;
    Ok(LegacySave { header, campaign })
}

// ─── Tests ──────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal valid legacy save file in memory.
    fn build_test_save(version: u32, mission_id: u32) -> Vec<u8> {
        use byteorder::{LittleEndian, WriteBytesExt};

        let mut buf = Vec::new();

        // ── Header ──
        buf.extend_from_slice(b"RHSG");
        buf.write_u32::<LittleEndian>(version).unwrap();
        buf.write_u32::<LittleEndian>(mission_id).unwrap();
        buf.write_u32::<LittleEndian>(version).unwrap(); // file version

        // ── Campaign ──

        buf.extend_from_slice(&Md5::digest(b"RHCampaign"));

        // reservists-are-back flag (version >= 28)
        if version >= 28 {
            buf.push(1); // true
        }

        // 27 × i32 campaign values — all zeros.
        for _ in 0..NUMBER_OF_VALUES {
            buf.write_i32::<LittleEndian>(0).unwrap();
        }

        // ARES
        buf.write_i8(5).unwrap();

        // ── Missions container (1 mission) ──
        buf.write_u32::<LittleEndian>(1).unwrap(); // count
        {
            buf.extend_from_slice(&Md5::digest(b"RHMission"));
            buf.write_u16::<LittleEndian>(3).unwrap(); // age
            buf.write_u16::<LittleEndian>(10).unwrap(); // blazon_price
            buf.write_u32::<LittleEndian>(0).unwrap(); // status = Available
            buf.write_u16::<LittleEndian>(0).unwrap(); // skip padding
            buf.write_u16::<LittleEndian>(0).unwrap(); // skip padding
            buf.write_u32::<LittleEndian>(0).unwrap(); // profile index
        }

        // ── Accessible missions (empty) ──
        buf.write_u32::<LittleEndian>(0).unwrap();

        // ── Pending accessible missions (empty) ──
        buf.write_u32::<LittleEndian>(0).unwrap();

        // ── Characters container (1 character) ──
        buf.write_u32::<LittleEndian>(1).unwrap(); // count
        {
            // Human status block.
            buf.extend_from_slice(&Md5::digest(b"RHHumanStatus"));
            buf.write_u32::<LittleEndian>(50).unwrap(); // hth capacity
            buf.write_u32::<LittleEndian>(10).unwrap(); // hth experience
            buf.write_u32::<LittleEndian>(75).unwrap(); // bow capacity
            buf.write_u32::<LittleEndian>(20).unwrap(); // bow experience

            // PC status block.
            buf.extend_from_slice(&Md5::digest(b"RHPCStatus"));
            buf.write_i16::<LittleEndian>(80).unwrap(); // life_points
            buf.push(0); // in_coma = false
            buf.write_u16::<LittleEndian>(3).unwrap(); // ales
            buf.write_u16::<LittleEndian>(2).unwrap(); // apples
            buf.write_u16::<LittleEndian>(15).unwrap(); // arrows
            buf.write_u16::<LittleEndian>(1).unwrap(); // nets
            buf.write_u16::<LittleEndian>(4).unwrap(); // plants
            buf.write_u16::<LittleEndian>(5).unwrap(); // purses
            buf.write_u16::<LittleEndian>(7).unwrap(); // rations (Stoeckel)
            buf.write_u16::<LittleEndian>(6).unwrap(); // stones
            buf.write_u16::<LittleEndian>(2).unwrap(); // wasp nests
            buf.write_i16::<LittleEndian>(-1).unwrap(); // beam_me_index

            // Wide string name: "Robin"
            let name: &[u16] = &[0x52, 0x6F, 0x62, 0x69, 0x6E]; // "Robin" as UCS-2
            buf.write_u16::<LittleEndian>(name.len() as u16).unwrap();
            for &ch in name {
                buf.write_u16::<LittleEndian>(ch).unwrap();
            }

            // Character profile index
            buf.write_u32::<LittleEndian>(0).unwrap();

            // bInstanced
            buf.push(1); // true
        }

        // ── Gang (1 entry) ──
        buf.write_u32::<LittleEndian>(1).unwrap();
        buf.write_u32::<LittleEndian>(0).unwrap(); // character index 0

        // ── Reservists (empty) ──
        buf.write_u32::<LittleEndian>(0).unwrap();

        // ── Mission team (empty) ──
        buf.write_u32::<LittleEndian>(0).unwrap();

        // ── Production sectors (1 sector with 0 occupants) ──
        buf.write_u32::<LittleEndian>(1).unwrap();
        {
            buf.extend_from_slice(&Md5::digest(b"RHSectorProduction"));
            buf.write_u32::<LittleEndian>(0).unwrap(); // type = MakeArrow
            buf.write_u16::<LittleEndian>(100).unwrap(); // speed
            buf.write_u16::<LittleEndian>(50).unwrap(); // amount
            buf.write_u16::<LittleEndian>(25).unwrap(); // produced_amount
            buf.push(0); // max_amount_reached = false
            buf.write_u32::<LittleEndian>(0).unwrap(); // 0 occupants
        }

        // ── Collected relics (empty) ──
        buf.write_u32::<LittleEndian>(0).unwrap();

        // ── Peasant names (1 name) ──
        buf.write_u32::<LittleEndian>(1).unwrap();
        {
            let name: &[u16] = &[0x48, 0x61, 0x6E, 0x73]; // "Hans"
            buf.write_u16::<LittleEndian>(name.len() as u16).unwrap();
            for &ch in name {
                buf.write_u16::<LittleEndian>(ch).unwrap();
            }
        }

        // ── Mission pointers ──
        buf.write_u16::<LittleEndian>(0xFFFF).unwrap(); // last = None
        buf.write_u16::<LittleEndian>(0).unwrap(); // current = 0
        buf.write_u16::<LittleEndian>(0xFFFF).unwrap(); // next = None
        buf.write_u16::<LittleEndian>(0xFFFF).unwrap(); // blazon = None

        // ── Last played missions (version >= 30) ──
        if version >= 30 {
            buf.write_u32::<LittleEndian>(1).unwrap(); // count
            buf.write_u16::<LittleEndian>(0).unwrap(); // mission index 0
        }

        // ── Last pseudo mission status (version >= 41) ──
        if version >= 41 {
            buf.write_u32::<LittleEndian>(0).unwrap(); // Available
            if version >= 48 {
                buf.write_u32::<LittleEndian>(42).unwrap(); // mission ID
            }
        }

        buf
    }

    #[test]
    fn load_minimal_save_v48() {
        let data = build_test_save(SAVE_VERSION, 7);
        let result = load_legacy_save_from_bytes(&data).unwrap();

        // Header
        assert_eq!(&result.header.mark, SAVE_MAGIC);
        assert_eq!(result.header.mission_id, 7);
        assert_eq!(result.header.file_version, SAVE_VERSION);

        // Campaign basics
        let c = &result.campaign;
        assert!(c.reservists_are_back);
        assert_eq!(c.ares, 5);
        assert_eq!(c.values, [0i32; NUMBER_OF_VALUES]);

        // Missions
        assert_eq!(c.missions.len(), 1);
        assert_eq!(c.missions[0].age, 3);
        assert_eq!(c.missions[0].blazon_price, 10);
        assert_eq!(c.missions[0].status, MissionStatus::Available);
        assert_eq!(c.missions[0].profile_idx, Some(0));

        // Characters
        assert_eq!(c.characters.len(), 1);
        let pc = &c.characters[0];
        assert_eq!(
            pc.character_profile_idx,
            Some(robin_engine::profiles::CharacterProfileIdx(0))
        );
        assert!(pc.instanced);
        assert_eq!(pc.status.life_points, 80);
        assert!(!pc.status.in_coma);
        assert_eq!(pc.status.num_ales, 3);
        assert_eq!(pc.status.num_apples, 2);
        assert_eq!(pc.status.num_arrows, 15);
        assert_eq!(pc.status.num_nets, 1);
        assert_eq!(pc.status.num_plants, 4);
        assert_eq!(pc.status.num_purses, 5);
        assert_eq!(pc.status.num_rations, 7);
        assert_eq!(pc.status.num_stones, 6);
        assert_eq!(pc.status.num_wasp_nests, 2);
        assert_eq!(pc.status.beam_me_index_in_sherwood, -1);
        assert_eq!(pc.status.name, "Robin");
        assert_eq!(pc.status.human_status.hand_to_hand.capacity, 50);
        assert_eq!(pc.status.human_status.hand_to_hand.experience, 10);
        assert_eq!(pc.status.human_status.bow.capacity, 75);
        assert_eq!(pc.status.human_status.bow.experience, 20);

        // Gang
        assert_eq!(c.gang_indices, vec![0]);
        assert!(c.reservist_indices.is_empty());
        assert!(c.mission_team_indices.is_empty());

        // Peasant names
        assert_eq!(c.peasant_names, vec!["Hans"]);

        // Mission pointers
        assert_eq!(c.last_mission_idx, None);
        assert_eq!(c.current_mission_idx, Some(0));
        assert_eq!(c.next_mission_idx, None);
        assert_eq!(c.blazon_mission_idx, None);

        // Last played missions
        assert_eq!(c.last_played_mission_indices, vec![0]);

        // Pseudo mission
        assert_eq!(c.last_pseudo_mission_status, MissionStatus::Available);
        assert_eq!(c.last_pseudo_mission_id, 42);
    }

    #[test]
    fn bad_magic_rejected() {
        let mut data = build_test_save(SAVE_VERSION, 0);
        data[0] = b'X'; // corrupt magic
        let err = load_legacy_save_from_bytes(&data).unwrap_err();
        assert!(
            format!("{err:?}").contains("magic"),
            "error should mention magic: {err:?}"
        );
    }

    #[test]
    fn bad_md5_rejected() {
        let mut data = build_test_save(SAVE_VERSION, 0);
        // Corrupt the RHCampaign MD5 fingerprint (starts at offset 16)
        data[16] ^= 0xFF;
        let err = load_legacy_save_from_bytes(&data).unwrap_err();
        assert!(
            format!("{err:?}").contains("MD5"),
            "error should mention MD5: {err:?}"
        );
    }

    #[test]
    fn load_with_production_occupants() {
        use byteorder::WriteBytesExt;

        let mut buf = Vec::new();
        let version: u32 = 48;

        // Header
        buf.extend_from_slice(b"RHSG");
        buf.write_u32::<LittleEndian>(version).unwrap();
        buf.write_u32::<LittleEndian>(1).unwrap(); // mission_id
        buf.write_u32::<LittleEndian>(version).unwrap();

        // Campaign
        buf.extend_from_slice(&Md5::digest(b"RHCampaign"));
        buf.push(0); // reservists_are_back = false
        for _ in 0..NUMBER_OF_VALUES {
            buf.write_i32::<LittleEndian>(0).unwrap();
        }
        buf.write_i8(0).unwrap(); // ares

        // 0 missions, 0 accessible, 0 pending, 0 characters
        for _ in 0..4 {
            buf.write_u32::<LittleEndian>(0).unwrap();
        }

        // Gang, reservists, mission team — all empty
        for _ in 0..3 {
            buf.write_u32::<LittleEndian>(0).unwrap();
        }

        // 1 production sector with 2 occupants
        buf.write_u32::<LittleEndian>(1).unwrap();
        {
            buf.extend_from_slice(&Md5::digest(b"RHSectorProduction"));
            buf.write_u32::<LittleEndian>(4).unwrap(); // MakeAle
            buf.write_u16::<LittleEndian>(10).unwrap(); // speed
            buf.write_u16::<LittleEndian>(20).unwrap(); // amount
            buf.write_u16::<LittleEndian>(5).unwrap(); // produced
            buf.push(1); // max_reached = true

            // 2 occupants
            buf.write_u32::<LittleEndian>(2).unwrap();
            for i in 0..2u32 {
                buf.write_u32::<LittleEndian>(i).unwrap(); // char idx
                buf.write_f32::<LittleEndian>(100.0 + i as f32).unwrap(); // x
                buf.write_f32::<LittleEndian>(200.0 + i as f32).unwrap(); // y
                buf.write_u16::<LittleEndian>(0xFFFF).unwrap(); // obstacle (v>=47)
            }
        }

        // Collected relics — empty
        buf.write_u32::<LittleEndian>(0).unwrap();
        // Peasant names — empty
        buf.write_u32::<LittleEndian>(0).unwrap();

        // Mission pointers — all None
        for _ in 0..4 {
            buf.write_u16::<LittleEndian>(0xFFFF).unwrap();
        }

        // Last played (v>=30)
        buf.write_u32::<LittleEndian>(0).unwrap();

        // Pseudo mission status (v>=41)
        buf.write_u32::<LittleEndian>(0).unwrap(); // Available
        buf.write_u32::<LittleEndian>(0).unwrap(); // id (v>=48)

        let result = load_legacy_save_from_bytes(&buf).unwrap();
        // Production sectors are read but not stored in Campaign — we just
        // verify the read didn't fail and the rest of the data is correct.
        assert!(!result.campaign.reservists_are_back);
        assert!(result.campaign.missions.is_empty());
    }

    #[test]
    fn wide_string_unicode() {
        // Test that non-ASCII wide strings decode correctly
        let mut buf = Vec::new();
        // "Müller" in UCS-2: M ü l l e r
        let name: &[u16] = &[0x4D, 0xFC, 0x6C, 0x6C, 0x65, 0x72];
        buf.extend_from_slice(&(name.len() as u16).to_le_bytes());
        for &ch in name {
            buf.extend_from_slice(&ch.to_le_bytes());
        }

        let mut reader = BinaryReader::new(buf.as_slice());
        reader.version = SAVE_VERSION;
        let s = reader.read_wide_string().unwrap();
        assert_eq!(s, "Müller");
    }
}
