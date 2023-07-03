fn collect_alloc_stats() -> anyhow::Result<GlobalStats> {
    tikv_jemalloc_ctl::epoch::advance()?;

    let mut opts = tikv_jemalloc_ctl::stats_print::Options::default();
    opts.json_format = true;
    opts.skip_per_arena = true;
    opts.skip_mutex_statistics = true;

    let mut buf = vec![];
    tikv_jemalloc_ctl::stats_print::stats_print(&mut buf, opts)?;

    Ok(serde_json::from_slice(&buf[..])?)
}

use {serde::Deserialize, tracing::log::warn};

#[derive(Debug, Deserialize)]
struct TotalStats {
    allocated: u64,
    active: u64,
    metadata: u64,
    resident: u64,
    mapped: u64,
    retained: u64,
}

#[derive(Debug, Deserialize)]
struct BinStats {
    nmalloc: u64,
    ndalloc: u64,
    nrequests: u64,
}

#[derive(Debug, Deserialize)]
struct MergedArenaStats {
    bins: Vec<BinStats>,
}

#[derive(Debug, Deserialize)]
struct ArenaStats {
    merged: MergedArenaStats,
}

#[derive(Debug, Deserialize)]
struct BinConstants {
    size: u64,
}

#[derive(Debug, Deserialize)]
struct ArenaConstants {
    bin: Vec<BinConstants>,
}

#[derive(Debug, Deserialize)]
struct Jemalloc {
    stats: TotalStats,

    #[serde(rename = "stats.arenas")]
    stats_arenas: ArenaStats,

    arenas: ArenaConstants,
}

#[derive(Debug, Deserialize)]
struct GlobalStats {
    jemalloc: Jemalloc,
}

pub struct AllocMetrics {
    allocated: opentelemetry::metrics::ValueRecorder<u64>,
    active: opentelemetry::metrics::ValueRecorder<u64>,
    metadata: opentelemetry::metrics::ValueRecorder<u64>,
    resident: opentelemetry::metrics::ValueRecorder<u64>,
    mapped: opentelemetry::metrics::ValueRecorder<u64>,
    retained: opentelemetry::metrics::ValueRecorder<u64>,
    bin: AllocBinMetrics,
}

struct AllocBinMetrics {
    nmalloc: opentelemetry::metrics::ValueRecorder<u64>,
    ndalloc: opentelemetry::metrics::ValueRecorder<u64>,
    nrequests: opentelemetry::metrics::ValueRecorder<u64>,
}

impl AllocBinMetrics {
    pub fn new(meter: &opentelemetry::metrics::Meter) -> Self {
        let nmalloc = meter
            .u64_value_recorder("jemalloc_memory_bin_nmalloc")
            .with_description(
                "Cumulative number of times a bin region of the corresponding size class was \
                 allocated from the arena, whether to fill the relevant tcache if opt.tcache is \
                 enabled, or to directly satisfy an allocation request otherwise.",
            )
            .init();
        let ndalloc = meter
            .u64_value_recorder("jemalloc_memory_bin_ndalloc")
            .with_description(
                "Cumulative number of times a bin region of the corresponding size class was \
                 returned to the arena, whether to flush the relevant tcache if opt.tcache is \
                 enabled, or to directly deallocate an allocation otherwise.",
            )
            .init();
        let nrequests = meter
            .u64_value_recorder("jemalloc_memory_bin_nrequests")
            .with_description(
                "Cumulative number of allocation requests satisfied by bin regions of the \
                 corresponding size class.",
            )
            .init();

        Self {
            nmalloc,
            ndalloc,
            nrequests,
        }
    }

    pub fn collect_alloc_stats(&self, stats: &GlobalStats) {
        let bin_const = stats.jemalloc.arenas.bin.iter();
        let bin_stats = stats.jemalloc.stats_arenas.merged.bins.iter();

        for (bin_const, bin_stats) in bin_const.zip(bin_stats) {
            let tags = [opentelemetry::KeyValue::new(
                "bin_size",
                bin_const.size.try_into().unwrap_or(0i64),
            )];

            self.nmalloc.record(bin_stats.nmalloc, &tags);
            self.ndalloc.record(bin_stats.ndalloc, &tags);
            self.nrequests.record(bin_stats.nrequests, &tags);
        }
    }
}

impl AllocMetrics {
    pub fn new(meter: &opentelemetry::metrics::Meter) -> Self {
        let allocated = meter
            .u64_value_recorder("jemalloc_memory_allocated")
            .with_description(
                "Total number of bytes allocated by the application. This corresponds to \
                 `stats.allocated` in jemalloc's API.",
            )
            .init();
        let active = meter
            .u64_value_recorder("jemalloc_memory_active")
            .with_description(
                "Total number of bytes in active pages allocated by the application. This \
                 corresponds to `stats.active` in jemalloc's API.",
            )
            .init();
        let metadata = meter
            .u64_value_recorder("jemalloc_memory_metadata")
            .with_description(
                "Total number of bytes dedicated to `jemalloc` metadata. This corresponds to \
                 `stats.metadata` in jemalloc's API.",
            )
            .init();
        let resident = meter
            .u64_value_recorder("jemalloc_memory_resident")
            .with_description(
                "Total number of bytes in physically resident data pages mapped by the allocator. \
                 This corresponds to `stats.resident` in jemalloc's API.",
            )
            .init();
        let mapped = meter
            .u64_value_recorder("jemalloc_memory_mapped")
            .with_description(
                "Total number of bytes in active extents mapped by the allocator. This \
                 corresponds to `stats.mapped` in jemalloc's API.",
            )
            .init();
        let retained = meter
            .u64_value_recorder("jemalloc_memory_retained")
            .with_description(
                "Total number of bytes in virtual memory mappings that were retained rather than \
                 being returned to the operating system via e.g. `munmap(2)`. This corresponds to \
                 `stats.retained` in jemalloc's API.",
            )
            .init();

        let bin = AllocBinMetrics::new(meter);

        Self {
            allocated,
            active,
            metadata,
            resident,
            mapped,
            retained,
            bin,
        }
    }

    pub fn collect_alloc_stats(&self) {
        let Ok(stats) = collect_alloc_stats() else {
            warn!("Failed to collect jemalloc stats.");
            return;
        };

        let mem_stats = &stats.jemalloc.stats;

        self.allocated.record(mem_stats.allocated, &[]);
        self.active.record(mem_stats.active, &[]);
        self.metadata.record(mem_stats.metadata, &[]);
        self.resident.record(mem_stats.resident, &[]);
        self.mapped.record(mem_stats.mapped, &[]);
        self.retained.record(mem_stats.retained, &[]);

        self.bin.collect_alloc_stats(&stats);
    }
}
