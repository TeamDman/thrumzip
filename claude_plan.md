# Claude Takeover Plan for meta-takeout CLI

## Example Analysis Summary

### perceptual_equality.rs (~25 lines pseudocode)
```
1. Define domain types PathToZip, PathInsideZip with holda derive
2. Scan multiple directories for .zip files
3. For each ZIP: extract entry list with metadata (name, compressed_size)
4. Build HashMap<PathInsideZip, Vec<RawEntryInfo>> for duplicate detection
5. Filter to entries with >1 occurrence and image extensions
6. For each duplicate image set:
   - Spawn parallel tasks to extract bytes from each ZIP
   - Decode image using image crate
   - Compute perceptual hash using img_hash with Gradient algorithm
   - Compare all pairs using Hamming distance
7. If all instances within similarity threshold:
   - Report as SIMILAR, identify smallest file
8. If any pair exceeds threshold:
   - Report as MISMATCH with details
9. Use tracing for progress logging throughout
```

### extract_files.rs (~25 lines pseudocode)
```
1. Scan directories for ZIP files
2. Build entry_map: HashMap<PathInsideZip, HashSet<PathToZip>>
3. Create fzf choices from entry_map for user selection
4. Use cloud_terrastodon_user_input::pick_many for interactive selection
5. Create output directory structure with numbered folders
6. For each selected entry:
   - Create subfolder with format "{:04}_{filename}"
   - Extract from all ZIPs containing the entry
   - Save each variant as "{:04}/{filename}" 
   - Generate provenance.txt mapping variants to source ZIPs
7. Use async file operations throughout
```

### check_duplicate_equality.rs (~25 lines pseudocode)
```
1. Collect ZIP files from multiple directories
2. Build entry_map: HashMap<PathInsideZip, Vec<PathToZip>>
3. Filter to entries appearing in multiple ZIPs
4. For each duplicate entry:
   - Extract bytes from each ZIP containing it
   - Compute DefaultHasher hash of raw bytes
   - Compare hashes across all instances
5. Report MISMATCH if hashes differ, identical otherwise
6. Note: Comments indicate this approach is "too slow"
```

### uom_bytes_eta.rs (~25 lines pseudocode)
```
1. Generate mock batches of files with random sizes
2. Use uom crate for type-safe byte measurements
3. Simulate processing with sleep proportional to file size
4. Track progress metrics: processed files/bytes, elapsed time
5. Calculate real-time throughput (files/sec, bytes/sec)
6. Estimate ETA based on remaining bytes and current rate
7. Use humansize for byte formatting, humantime for duration
8. Print progress updates every ~1 second with:
   - Batch progress, files/sec, bytes/sec, remaining, ETA
```

### crc32.rs (~25 lines pseudocode)
```
1. Scan directories for ZIP files with modification times
2. Spawn parallel tasks per ZIP file
3. For each ZIP:
   - Build local map of entry_name -> (crc32, modified_time, size)
   - Collect entry names for random sampling
4. Merge all local maps into global entry_map
5. Randomly sample entries for CRC32 validation
6. For sampled entries:
   - Extract bytes and compute actual CRC32
   - Compare with ZIP's stored CRC32
   - Report mismatches (indicates corruption)
```

## Current State Analysis

### Implemented ✅
- **CLI Structure**: `clap` with derive, `Command` struct, `GlobalArgs` with `--debug`/`--non-interactive`
- **Tracing**: Conditional debug/release logging via `init_tracing`
- **Configuration**: `AppConfig` with `eye_config::PersistableState` for destination/sources/similarity
- **Config Init**: Interactive setup command using `cloud_terrastodon_user_input`
- **Sync Scaffold**: Basic command structure with ZIP scanning and entry mapping
- **Part 1 Implementation**: CRC-based deduplication with parallel extraction in `sync_utils::sync_part1`
- **Domain Types**: `PathToZip`, `PathInsideZip` with `holda` derives
- **Error Handling**: Consistent `eyre::Result` usage with `wrap_err` context

### Current Part 1 Implementation Status
The existing `sync_part1` function:
- ✅ Takes map of ZIP -> paths to extract
- ✅ Spawns parallel extraction tasks
- ✅ Checks if destination file exists (skips if present)
- ✅ Creates parent directories as needed
- ✅ Extracts files using `rc_zip_tokio`
- ✅ Logs progress with remaining count
- ⚠️  **ISSUE**: Progress tracking is basic (just countdown, no ETA/throughput)

### Missing Implementation ❌
- **Part 2**: Perceptual image deduplication
- **Part 3**: Non-image duplicate export with ZIP-name directories
- **Part 4**: Full validation phase with CRC re-verification
- **Enhanced Progress**: ETA, throughput metrics, rich progress display
- **sync_types.rs**: Proper `RawInfo` usage (currently duplicated in sync.rs)

## Gap Analysis: Current vs Target CLI Experience

### Target Experience (from instructions)
```
Part 1: Of the 35000 uniquely named files, 13245 files (37.8%) are the same across all zips (crc32)
Part 2: Of the 35000 uniquely named files, 21000 files (62.1%) are images we will check perceptual similarity for
Part 3: Of the 35000 uniquely named files, 755 (2.2%) are documents with no diff support, each copy will be exported
Part 4: Validation

===
part 2
===
Found 3 files with the same name: "media\other\23967524_1338605629584566_3934457482159587328_n_17870308819197916.jpg"
Found the files in the following zip files:
- C:\Users\TeamD\OneDrive\Documents\Backups\meta\instagram-teamdman-2024-06-18-DveFYE6C.zip
- C:\Users\TeamD\OneDrive\Documents\Backups\meta\instagram-teamdman-2024-06-19-Ab12EECC.zip
Uncompressed size: min=22.7 kb, max=33.8 kb
Computing hashes for each image...
Images are perceptually similar ✅ (dist min=0, max=0.3, mean=0.1)
Syncing smallest file to disk (22.7 kb)
```

### Current Gap
- ❌ No partition statistics reporting (37.8% same CRC, 62.1% images, etc.)
- ❌ No Part 2 image deduplication with similarity checking
- ❌ No detailed per-file analysis with size ranges and similarity metrics
- ❌ No Part 3 multi-variant export for non-image duplicates
- ❌ No Part 4 validation with CRC re-checking
- ❌ No rich progress with throughput/ETA (currently just basic countdown)

## Implementation Plan

### Phase 1: Refactor and Fix Current Code
**Target: 2-3 hours**

1. **Fix sync_types.rs usage**
   - Move `RawInfo` from `sync.rs` to `sync_types.rs` 
   - Update imports in `sync.rs` to use `sync_types::RawInfo`
   - Add `sync_types` to `mod.rs`

2. **Enhance Part 1 Progress Display**
   - Add dependency: `humansize`, `humantime` (already in Cargo.toml)
   - Create `progress_tracker.rs` with:
     - `ProgressTracker` struct tracking files/bytes processed
     - Real-time throughput calculation (files/sec, bytes/sec)
     - ETA estimation based on remaining work
     - Formatted progress messages matching target output
   - Update `sync_part1` to use `ProgressTracker`

3. **Add Partition Statistics**
   - Create `partition_analyzer.rs` with:
     - `analyze_partitions()` function taking `entry_map`
     - Return `PartitionStats` with counts for part1/part2/part3
     - Image extension detection logic
   - Update sync.rs to call analyzer and log statistics

### Phase 2: Implement Part 2 (Image Deduplication)
**Target: 4-6 hours**

1. **Create Image Processing Module**
   - `image_dedup.rs` with:
     - `ImageInstance` struct (zip_path, hash, size)
     - `is_image_extension()` helper
     - `compute_perceptual_hash()` using `img_hash` crate
     - `analyze_image_group()` for similarity checking

2. **Implement sync_part2**
   - In `sync_utils.rs` add `sync_part2()` function:
     - Take entry_map filtered to multi-occurrence images
     - For each image path: extract all variants, compute hashes
     - Check pairwise similarity using Hamming distance
     - If similar: extract smallest file
     - If dissimilar: log mismatch details
     - Use `ProgressTracker` for throughput display

3. **Update main sync workflow**
   - After Part 1, partition remaining entries into images vs non-images
   - Call `sync_part2` with image entries
   - Log detailed per-file analysis matching target output

### Phase 3: Implement Part 3 (Non-Image Export)
**Target: 2-3 hours**

1. **Create Multi-Variant Export**
   - `multi_export.rs` with:
     - `export_all_variants()` function
     - Create subdirectories named after source ZIP files
     - Extract each variant to `dest/path/zipname.zip/filename`
     - Track and report existing vs new files

2. **Implement sync_part3**
   - In `sync_utils.rs` add `sync_part3()` function
   - Take non-image entries with multiple occurrences
   - Use multi-variant export logic
   - Progress tracking with file counts

### Phase 4: Implement Part 4 (Validation)
**Target: 2-3 hours**

1. **Create Validation Module**
   - `validation.rs` with:
     - `validate_extracted_files()` function
     - Re-read extracted files, compute CRC32
     - Compare against expected CRC from ZIP metadata
     - Report validation success/failure statistics

2. **Implement sync_part4**
   - In `sync_utils.rs` add `sync_part4()` function
   - Scan destination directory for all extracted files
   - Parallel CRC validation with progress tracking
   - Generate final summary statistics

### Phase 5: Integration and Polish
**Target: 2-3 hours**

1. **Complete Main Sync Flow**
   - Update `sync.rs` to call all 4 parts in sequence
   - Add proper error handling and recovery
   - Ensure idempotent operation (skip existing files)

2. **Enhanced Logging and UX**
   - Rich progress display with ETA and throughput
   - Color-coded success/warning/error messages
   - Summary statistics at end of each phase

3. **Testing and Validation**
   - Test with actual Facebook/Instagram exports
   - Verify all file types are handled correctly
   - Confirm space savings and deduplication accuracy

## Technical Implementation Details

### Key Dependencies (Already Available)
- `img_hash`: Perceptual hashing with configurable algorithms
- `image`: Image decoding for hash computation
- `crc32fast`: Fast CRC32 computation for validation
- `humansize`/`humantime`: Progress display formatting
- `tokio`: Async/parallel processing throughout

### Progress Tracking Architecture
```rust
struct ProgressTracker {
    start_time: Instant,
    total_files: usize,
    total_bytes: u64,
    processed_files: usize,
    processed_bytes: u64,
    last_update: Instant,
}

impl ProgressTracker {
    fn update(&mut self, files: usize, bytes: u64) -> Option<String> {
        // Return formatted progress string if enough time elapsed
    }
}
```

### Similarity Threshold Handling
- Use `cfg.similarity` from configuration (0.0 to 1.0)
- Convert to Hamming distance: `max_distance = (1.0 - similarity) * hash_bits`
- For 64-bit hash with 99% similarity: `max_distance = 0.64 ≈ 1`

### Error Recovery Strategy
- Individual file failures should not stop entire sync
- Log errors but continue processing remaining files
- Final summary should include error counts
- Partial completion should be resumable on next run

## Success Criteria

1. **Functional Requirements**
   - ✅ All 4 sync phases implemented and working
   - ✅ Accurate image similarity detection
   - ✅ Proper multi-variant export for non-images
   - ✅ CRC validation catching any corruption
   - ✅ Idempotent operation (safe to re-run)

2. **UX Requirements**
   - ✅ Rich progress display with ETA/throughput
   - ✅ Detailed per-file analysis output
   - ✅ Clear phase separation and statistics
   - ✅ Informative error messages and recovery

3. **Performance Requirements**
   - ✅ Parallel processing for all I/O operations
   - ✅ Efficient memory usage (streaming where possible)
   - ✅ Reasonable throughput (target ~2-10 files/sec based on example)

This plan provides a clear roadmap to transform the current Part 1 implementation into a complete, production-ready CLI tool matching the detailed requirements in the instructions.