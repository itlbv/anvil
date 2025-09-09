use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::io::{BufReader, BufWriter};
use std::path::Path; // for flush()

use bincode::config::standard;
use bincode::serde::{decode_from_std_read, encode_into_std_write};

#[derive(Serialize, Deserialize, Clone)]
pub struct RunMeta {
    pub sim_hz: u32,
    pub seed: u64,
    pub version: u32, // bump if you change event schema
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PropsDelta {
    #[serde(with = "crate::entity_serde::opt")]
    pub selected_entity: Option<hecs::Entity>,
    pub draw_map_grid: Option<bool>,
    pub quit: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TickEvents {
    pub tick: u64,
    pub props: Option<PropsDelta>,
    pub commands: Vec<crate::entity_commands::EntityCommand>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Trailer {
    pub end_tick: u64,
    pub final_world_hash: u64,
}

pub struct Recorder {
    cfg: bincode::config::Configuration,
    w: BufWriter<File>,
}

impl Recorder {
    // <- Use a generic P: AsRef<Path>
    pub fn new<P: AsRef<Path>>(path: P, meta: RunMeta) -> Result<Self> {
        let p = path.as_ref();
        let f = File::create(p)?;
        let mut w = BufWriter::new(f);
        let cfg = standard();
        encode_into_std_write(&meta, &mut w, cfg)?;
        Ok(Self { cfg, w })
    }

    pub fn push(&mut self, ev: &TickEvents) -> Result<()> {
        encode_into_std_write(ev, &mut self.w, self.cfg)?;
        Ok(())
    }

    pub fn finish(mut self, trailer: &Trailer) -> Result<()> {
        encode_into_std_write(trailer, &mut self.w, self.cfg)?;
        self.w.flush()?; // <- ensure bytes hit disk
        Ok(())
    }
}

pub struct Player {
    cfg: bincode::config::Configuration,
    r: BufReader<File>,
    pub meta: RunMeta,
    pub trailer: Trailer,
    peek: Option<TickEvents>,
}

impl Player {
    // <- same generic signature here
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let cfg = standard();
        let p = path.as_ref();

        // Reader #1: normal streaming reader (meta consumed)
        let f1 = File::open(p)?;
        let mut r1 = BufReader::new(f1);
        let meta: RunMeta = decode_from_std_read(&mut r1, cfg)?;

        // Reader #2: scan to preload trailer (skip all TickEvents until decode fails, then read Trailer)
        let f2 = File::open(p)?;
        let mut r2 = BufReader::new(f2);
        let _meta2: RunMeta = decode_from_std_read(&mut r2, cfg)?;
        loop {
            match decode_from_std_read::<TickEvents, _, _>(&mut r2, cfg) {
                Ok(_ev) => continue,
                Err(_) => break, // trailer next
            }
        }

        let trailer: Trailer = match decode_from_std_read(&mut r2, cfg) {
            Ok(t) => t,
            Err(e) => {
                return Err(anyhow::anyhow!(
            "Trace is missing a Trailer (file truncated or recorded with an older binary). \
             Re-record with the current build. Underlying error: {e}"
        ));
            }
        };

        Ok(Self {
            cfg,
            r: r1,
            meta,
            trailer,
            peek: None,
        })
    }

    /// Non-blocking: return events exactly for `tick`. Stops as soon as a different tick is seen.
    pub fn next_for_tick(&mut self, tick: u64) -> Result<Vec<TickEvents>> {
        let mut out = Vec::new();

        if let Some(ev) = self.peek.take() {
            if ev.tick == tick {
                out.push(ev);
            } else {
                self.peek = Some(ev);
                return Ok(out);
            }
        }

        const MAX_READS_PER_CALL: usize = 64;
        for _ in 0..MAX_READS_PER_CALL {
            match decode_from_std_read::<TickEvents, _, _>(&mut self.r, self.cfg) {
                Ok(ev) if ev.tick == tick => out.push(ev),
                Ok(ev) => {
                    self.peek = Some(ev);
                    break;
                }
                Err(_) => break, // EOF or trailer boundary
            }
        }

        Ok(out)
    }

    /// If you still want it; returns the preloaded trailer.
    pub fn read_trailer(self) -> Trailer {
        self.trailer
    }
}
