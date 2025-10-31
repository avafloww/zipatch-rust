//! Example: Parse and display information about a patch file
use zipatch::{Platform, ZiPatchConfig, ZiPatchFile};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <patch_file>", args[0]);
        std::process::exit(1);
    }

    let patch_path = &args[1];
    println!("Opening patch file: {}", patch_path);

    let mut patch = ZiPatchFile::from_path(patch_path)?;
    let header = patch.header();

    println!("\n=== Patch Header ===");
    println!("Version: {}", header.version);
    println!("Patch Type: {}", header.patch_type);
    println!("Entry Files: {}", header.entry_files);
    if let Some(counts) = &header.command_counts {
        println!("Total Commands: {}", counts.total_commands);
        println!("  Add Directories: {}", counts.add_directories);
        println!("  Delete Directories: {}", counts.delete_directories);
        println!("  SQPK Add: {}", counts.sqpk_add_commands);
        println!("  SQPK Delete: {}", counts.sqpk_delete_commands);
        println!("  SQPK Expand: {}", counts.sqpk_expand_commands);
        println!("  SQPK Header: {}", counts.sqpk_header_commands);
        println!("  SQPK File: {}", counts.sqpk_file_commands);
    }

    // Create a dummy config for analyzing changes
    let config = ZiPatchConfig::builder(".")
        .platform(Platform::Win32)
        .build();

    println!("\n=== Analyzing Changes ===");
    let changes = patch.calculate_changed_files(&config)?;
    println!("Added files: {}", changes.added.len());
    println!("Modified files: {}", changes.modified.len());
    println!("Deleted files: {}", changes.deleted.len());

    if !changes.added.is_empty() {
        println!("\nAdded:");
        for file in changes.added.iter().take(10) {
            println!("  + {}", file);
        }
        if changes.added.len() > 10 {
            println!("  ... and {} more", changes.added.len() - 10);
        }
    }

    println!("\n=== Chunk Summary ===");
    let actual_counts = patch.calculate_actual_counts()?;
    println!("Total chunks: {}", actual_counts.total_commands);

    println!("\n✓ Patch file parsed successfully!");

    Ok(())
}
