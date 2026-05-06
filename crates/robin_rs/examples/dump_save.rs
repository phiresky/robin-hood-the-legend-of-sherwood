//! Ad-hoc inspector for legacy save files.
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let path: PathBuf = std::env::args()
        .nth(1)
        .expect("usage: dump_save <path>")
        .into();
    let mut data = std::fs::read(&path)?;
    println!("file: {} ({} bytes)", path.display(), data.len());
    println!("first 16 bytes: {:02x?}", &data[..16.min(data.len())]);

    // If magic looks byte-reversed ("GSHR"), reverse first 4 bytes.
    if &data[..4] == b"GSHR" {
        println!("magic is reversed — reversing first 4 bytes to RHSG");
        data[..4].reverse();
    }

    match robin_rs::legacy_save::load_legacy_save_from_bytes(&data) {
        Ok(save) => {
            let c = &save.campaign;
            println!(
                "header: version={} mission_id={} file_version={}",
                save.header.header_version, save.header.mission_id, save.header.file_version
            );
            println!(
                "ares={} reservists_are_back={}",
                c.ares, c.reservists_are_back
            );
            println!(
                "missions={} characters={} gang={} reservists={} team={} sectors={} relics={} peasants={}",
                c.missions.len(),
                c.characters.len(),
                c.gang_indices.len(),
                c.reservist_indices.len(),
                c.mission_team_indices.len(),
                c.production_sectors.len(),
                c.collected_relics.len(),
                c.peasant_names.len()
            );
            println!(
                "last_mission_idx={:?} current={:?} next={:?} blazon={:?}",
                c.last_mission_idx, c.current_mission_idx, c.next_mission_idx, c.blazon_mission_idx
            );
            println!("last_played={:?}", c.last_played_mission_indices);
            println!(
                "LAST PSEUDO MISSION: status={:?} id={}",
                c.last_pseudo_mission_status, c.last_pseudo_mission_id
            );
        }
        Err(e) => println!("ERROR: {e:#}"),
    }
    Ok(())
}
